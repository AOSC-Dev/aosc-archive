use anyhow::{bail, Context, Result};
use bytesize::ByteSize;
use dbus_tokio::connection;
use log::{error, info};
use serde::Deserialize;
use sqlx::{query_as, PgPool};
use std::sync::atomic::AtomicUsize;
use std::{path::Path, sync::atomic::Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

mod cli;
mod dbus;

#[derive(Debug, Deserialize)]
struct Config {
    config: GeneralConfig,
}

#[derive(Debug, Deserialize)]
struct GeneralConfig {
    pub db_pgconn: String,
    pub path: String,
    pub abbs_sync: bool,
}

#[derive(Debug, Clone)]
struct PackageMeta {
    package: String,
    sha256: String,
    size: i64,
    filename: String,
}

async fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let mut f = tokio::fs::File::open(path).await?;
    let mut buffer = Vec::new();
    buffer.reserve(1024);
    f.read_to_end(&mut buffer).await?;

    Ok(toml::from_slice(&buffer)?)
}

async fn determine_retired_packages(pool: &PgPool, oot: bool) -> Result<Vec<PackageMeta>> {
    let packages = query_as!(
        PackageMeta,
        r#"SELECT package, sha256, size, filename FROM 
(SELECT *, rank() OVER (PARTITION BY package, repo ORDER BY _vercomp DESC) AS pos FROM pv_packages) 
AS sq WHERE pos > 1"#
    )
    .fetch_all(pool)
    .await?;

    if oot {
        let mut oot_packages = query_as!(
            PackageMeta,
        r#"SELECT DISTINCT pp.package, pp.sha256, pp.size, pp.filename FROM 
pv_packages pp LEFT JOIN packages p ON pp.package = p.name WHERE 
tree IS NULL AND pp.package NOT LIKE '%-dbg' AND pp.package NOT SIMILAR TO '(linux-kernel-|linux\+kernel\+|u-boot)%'"#).fetch_all(pool).await?;
        oot_packages.extend(packages);
        return Ok(oot_packages);
    }

    Ok(packages)
}

async fn retire_action<P: AsRef<Path>>(
    config_file: P,
    dry_run: bool,
    output: P,
    oot: bool,
) -> Result<()> {
    let config = load_config(config_file).await?;
    info!("Connecting to database ...");
    let pool = PgPool::connect(&config.config.db_pgconn).await?;
    info!("Determining what packages to retire ...");
    if !config.config.abbs_sync && oot {
        error!("Invalid configuration: abbs_sync should be enabled in order to correctly retire packages!");
        bail!("Refusing to continue to avoid damaging package pool")
    }
    let packages = determine_retired_packages(&pool, oot).await?;
    let total_size = packages.iter().fold(0, |t, x| t + x.size);
    let total_count = packages.len();

    info!(
        "{} packages to retire, {} total",
        total_count,
        ByteSize::b(total_size as u64).to_string()
    );

    if dry_run {
        info!(
            "The following packages would be moved to `{}`:",
            output.as_ref().display()
        );
        for p in packages.iter() {
            info!("{}: {}", p.package, p.filename);
        }
        info!(
            "[DRY-RUN] {} packages would be retired, {} total",
            total_count,
            ByteSize::b(total_size as u64).to_string()
        );
        return Ok(());
    }

    info!("Moving retired packages ...");
    let count = AtomicUsize::new(1);
    let output_path = output.as_ref();
    tokio::fs::create_dir_all(output_path).await?;
    // generate the manifest
    info!("Generating manifest ...");
    let manifest = packages.iter().fold(String::new(), |t, x| {
        t + &x.sha256 + " " + &x.filename + "\n"
    });
    let mut f = tokio::fs::File::create(output_path.join("backup_label")).await?;
    f.write_all(manifest.as_bytes()).await?;
    // move files
    let mut tasks = Vec::new();
    let original_path = Path::new(&config.config.path);
    for p in packages.iter() {
        tasks.push(backup_package(
            &count,
            total_count,
            &p.filename,
            output_path,
            original_path,
        ));
    }
    info!("Moving files ...");
    let mut errored = false;
    for r in futures::future::join_all(tasks).await {
        if let Err(e) = r {
            errored = true;
            error!("Error occurred while moving files: {:?}", e);
        }
    }
    if errored {
        bail!("Errors detected, bailing out ...")
    }

    Ok(())
}

async fn backup_package(
    count: &AtomicUsize,
    total_count: usize,
    filename: &str,
    output_path: &Path,
    original_path: &Path,
) -> Result<()> {
    info!(
        "[{}/{}] Moving {} ... ",
        count.fetch_add(1, Ordering::SeqCst),
        total_count,
        filename
    );
    let path = Path::new(filename);
    if let Some(parent) = path.parent() {
        let target_dir = output_path.join(parent);
        let original_path = original_path.join(path);
        tokio::fs::create_dir_all(&target_dir)
            .await
            .with_context(|| format!(
                "when creating target directory {}",
                target_dir.display()
            ))?;
        tokio::fs::copy(&original_path, output_path.join(path))
            .await
            .with_context(|| format!("when copying {}", original_path.display()))?;
        tokio::fs::remove_file(&original_path)
            .await
            .with_context(|| format!("when deleting {}", original_path.display()))?;
        info!("Successfully moved {}", filename);
    } else {
        error!("No parent directory: {}", filename);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = cli::build_cli();
    let args = app.get_matches();

    env_logger::init();
    match args.subcommand() {
        Some(("retire", args)) => {
            let inhibit = args.values_of("inhibit");
            let (resource, conn) = connection::new_system_sync()?;
            let _handle = tokio::spawn(async {
                let err = resource.await;
                error!("Lost connection to D-Bus: {}", err);
            });
            // handle inhibition
            let mut inhibited = None;
            if let Some(inhibit) = inhibit {
                inhibited = Some(
                    dbus::inhibit_services(conn.as_ref(), &inhibit.collect::<Vec<_>>())
                        .await
                        .unwrap(),
                );
            }
            retire_action(
                args.value_of("config").unwrap(),
                args.is_present("dry-run"),
                args.value_of("output").unwrap(),
                args.is_present("out-of-tree"),
            )
            .await?;
            // restore services
            if let Some(inhibit) = inhibited {
                dbus::restore_services(conn.as_ref(), &inhibit).await?;
            }
        }
        Some(("binning", _args)) => {
            todo!()
        }
        _ => {
            std::process::exit(1);
        }
    }

    Ok(())
}
