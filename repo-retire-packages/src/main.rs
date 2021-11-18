use anyhow::Result;
use bytesize::ByteSize;
use log::{error, info};
use serde::Deserialize;
use sqlx::{query_as, PgPool};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

mod cli;

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

async fn determine_retired_packages(pool: &PgPool) -> Result<Vec<PackageMeta>> {
    let packages = query_as!(
        PackageMeta,
        r#"SELECT package, sha256, size, filename FROM 
(SELECT *, rank() OVER (PARTITION BY package, repo ORDER BY _vercomp DESC) AS pos FROM pv_packages) 
AS sq WHERE pos > 1"#
    )
    .fetch_all(pool)
    .await?;

    Ok(packages)
}

async fn retire_action<P: AsRef<Path>>(config_file: P, dry_run: bool, output: P) -> Result<()> {
    let config = load_config(config_file).await?;
    info!("Connecting to database ...");
    let pool = PgPool::connect(&config.config.db_pgconn).await?;
    info!("Determining what packages to retire ...");
    let packages = determine_retired_packages(&pool).await?;
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
        return Ok(());
    }

    info!("Moving retired packages ...");
    let mut count = 1usize;
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
    for p in packages.iter() {
        info!("[{}/{}] Moving {} ... ", count, total_count, p.filename);
        let path = Path::new(&p.filename);
        if let Some(parent) = path.parent() {
            let target_dir = output_path.join(parent);
            let original_path = Path::new(&config.config.path).join(path);
            tokio::fs::create_dir_all(target_dir).await?;
            tokio::fs::copy(&original_path, output_path.join(path)).await?;
            tokio::fs::remove_file(&original_path).await?;
        } else {
            error!("No parent directory: {}", p.filename);
        }
        count += 1;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = cli::build_cli();
    let args = app.get_matches();

    env_logger::init();
    match args.subcommand() {
        ("retire", Some(args)) => {
            retire_action(
                args.value_of("config").unwrap(),
                args.is_present("dry-run"),
                args.value_of("output").unwrap(),
            )
            .await?;
        }
        ("binning", Some(args)) => {
            todo!()
        }
        _ => {
            println!("{}", args.usage());
            std::process::exit(1);
        }
    }

    Ok(())
}
