use anyhow::{bail, Context, Result};
use bytesize::ByteSize;
use log::{error, info};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::atomic::AtomicUsize;
use std::{path::Path, sync::atomic::Ordering};
use tokio::io::AsyncReadExt;

use crate::abbs::update_abbs_database;
use crate::db::{
    determine_retired_kernel_packages, determine_retired_packages, save_new_packages, PackageMeta,
};

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

async fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let mut f = tokio::fs::File::open(path).await?;
    let mut buffer = String::new();
    buffer.reserve(1024);
    f.read_to_string(&mut buffer).await?;

    Ok(toml::from_str(&buffer)?)
}

pub async fn retire_action<P: AsRef<Path>>(
    config_file: P,
    dry_run: bool,
    output: P,
    oot: bool,
    kernel: bool,
    db_path: P,
    abbs_path: P,
) -> Result<()> {
    let config = load_config(config_file).await?;
    info!("Connecting to database ...");
    let pool = PgPool::connect(&config.config.db_pgconn).await?;
    if oot {
        info!("Out-of-tree retirement enabled.");
        info!("Updating the in-tree package database ...");
        update_abbs_database(&pool, &abbs_path).await?;
    }
    info!("Determining what packages to retire ...");
    if !config.config.abbs_sync && oot {
        error!("Invalid configuration: abbs_sync should be enabled in order to correctly retire packages!");
        bail!("Refusing to continue to avoid damaging package pool")
    }
    let mut packages = determine_retired_packages(&pool, oot).await?;

    if kernel {
        let outdated_kernel_packages = determine_retired_kernel_packages(&pool).await?;
        packages.extend(outdated_kernel_packages);
    }

    let total_size = packages.iter().fold(0, |t, x| t + x.size);
    let total_count = packages.len();

    info!(
        "{} packages to retire, {} total",
        total_count,
        ByteSize::b(total_size as u64).to_string_as(true)
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
            ByteSize::b(total_size as u64)
        );
        return Ok(());
    }

    info!("Moving retired packages ...");
    let count = AtomicUsize::new(1);
    let output_path = output.as_ref();
    tokio::fs::create_dir_all(output_path).await?;
    generate_manifest(&packages, db_path.as_ref()).await?;
    // move files
    for package_chunk in packages.chunks(40) {
        let errored =
            chunked_copy_files(&config, package_chunk, &count, total_count, output_path).await;
        if errored {
            bail!("Errors detected, bailing out ...")
        }
    }

    Ok(())
}

async fn generate_manifest(packages: &Vec<PackageMeta>, db_path: &Path) -> Result<()> {
    info!("Generating manifest ...");
    let db_path = db_path.to_owned();
    let packages = packages.clone();
    tokio::task::spawn_blocking(move || save_new_packages(db_path, &packages)).await??;

    Ok(())
}

async fn chunked_copy_files(
    config: &Config,
    packages: &[PackageMeta],
    count: &AtomicUsize,
    total_count: usize,
    output_path: &Path,
) -> bool {
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

    errored
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
        let dest_path = output_path.join(path);
        if tokio::fs::metadata(&dest_path).await.is_ok() {
            info!("Skipping, already copied: {}", filename);
            return Ok(());
        }
        tokio::fs::create_dir_all(&target_dir)
            .await
            .with_context(|| format!("when creating target directory {}", target_dir.display()))?;
        tokio::fs::copy(&original_path, &dest_path)
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
