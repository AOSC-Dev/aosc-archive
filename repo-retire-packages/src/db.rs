use std::path::Path;

use anyhow::{Context, Result};
use log::debug;
use rusqlite::{params, Connection};
use sqlx::{query, query_as, PgPool};

const SQLITE_INIT_SCRIPT: &str = include_str!("../init.sql");

#[derive(Debug, Clone)]
pub struct PackageMeta {
    pub package: String,
    pub sha256: String,
    pub size: i64,
    pub filename: String,
    pub version: String,
    pub architecture: String,
    pub repo: String,
}

pub async fn determine_retired_packages(pool: &PgPool, oot: bool) -> Result<Vec<PackageMeta>> {
    let packages = query_as!(
        PackageMeta,
        r#"SELECT package, sha256, size, filename, version, architecture, repo FROM 
(SELECT *, rank() OVER (PARTITION BY package, repo ORDER BY _vercomp DESC) AS pos FROM pv_packages) 
AS sq WHERE pos > 1"#
    )
    .fetch_all(pool)
    .await?;

    if oot {
        let mut oot_packages = query_as!(
            PackageMeta,
        r#"SELECT DISTINCT pp.package, pp.sha256, pp.size, pp.filename, pp.version, pp.architecture, pp.repo FROM 
pv_packages pp LEFT JOIN packages p ON pp.package = p.name WHERE 
tree IS NULL AND pp.package NOT LIKE '%-dbg' AND pp.package NOT SIMILAR TO '(linux-kernel-|linux\+kernel\+|u-boot)%'
AND pp.repo LIKE '%/stable'"#).fetch_all(pool).await?;
        oot_packages.extend(packages);
        return Ok(oot_packages);
    }

    Ok(packages)
}

pub async fn determine_retired_kernel_packages(pool: &PgPool) -> Result<Vec<PackageMeta>> {
    // Sorry, but I think the easiest way to do it is to use subqueries.
    // Feel free to improve the following query.
    log::info!("Fecthing packages to retire for the mainline kernel ...");
    let mut collection = query_as!(
        PackageMeta,
        r#"
        SELECT DISTINCT
            p1.package, p1.sha256, p1.size, p1.filename, p1.version, p1.architecture, p1.repo
        FROM pv_packages p1
        WHERE
	        p1.package LIKE 'linux-kernel-%'
	        AND p1.package SIMILAR TO 'linux-kernel-[0-9]%'
	        AND NOT EXISTS (
                SELECT 1
                FROM pv_packages p2
                WHERE
                    p2.package = 'linux+kernel'
                    AND substring(p1.version FROM '[0-9]+\.[0-9]+\.[0-9]+') = substring(p2.version FROM '[0-9]+\.[0-9]+\.[0-9]+')
            )
    "#
    )
    .fetch_all(pool)
    .await?;
    // WARNING: we can not hardcode kernel variants.
    let other_variants = query!(
        r#"
        SELECT DISTINCT
            substring(package FROM '(?:linux\+kernel\+)(.*)')
            AS variants
        FROM pv_packages
        WHERE
            package SIMILAR TO 'linux\+kernel\+%';
    "#
    )
    .fetch_all(pool)
    .await?;
    // NOTE: I think it must be done this way, otherwise we have to alter
    // the database (to create a function that can return rows) and call
    // that function in this series of queries.
    // Also due to the naming convention of kernel packages, I can't come
    // up a better idea that the query can be done without loops. Sorry,
    // but this is the best thing I can do for now.
    for variant in other_variants {
        let variant_name = variant.variants.context("Got an empty variant name")?;
        log::info!(
            "Fecthing packages to retire for kernel variant {}",
            &variant_name
        );
        // Let's log available versions for linux+kernel metapackages, just
        // to make sure.
        let available_versions = query!(
            r#"
            SELECT DISTINCT
                substring(version FROM '[0-9]+\.[0-9]+\.[0-9]+')
                AS version
            FROM pv_packages
            WHERE
                package = ('linux+kernel+' || $1)
        "#,
            &variant_name
        )
        .fetch_all(pool)
        .await?;
        log::info!("Available versions for 'linux+kernel+{}' :", &variant_name);
        available_versions
            .iter()
            .for_each(|x| log::info!(" - {}", x.version.clone().unwrap_or_default()));
        let collection_other_variants = query_as!(
            PackageMeta,
            r#"
            SELECT DISTINCT
                p1.package, p1.sha256, p1.size, p1.filename, p1.version, p1.architecture, p1.repo
            FROM pv_packages p1
            WHERE
                p1.package SIMILAR TO ('linux-kernel-' || $1 || '-[0-9]%')
                AND NOT EXISTS (
                    SELECT 1
                    FROM pv_packages p2
                    WHERE
                        p2.package = ('linux+kernel+' || $1)
                        AND substring(p1.version FROM '[0-9]+\.[0-9]+\.[0-9]+') = substring(p2.version FROM '[0-9]+\.[0-9]+\.[0-9]+')
                )
        "#,
            variant_name
        )
        .fetch_all(pool)
        .await?;
        collection.extend(collection_other_variants);
    }
    Ok(collection)
}

pub fn save_new_packages<P: AsRef<Path>>(db_path: P, packages: &[PackageMeta]) -> Result<()> {
    let mut conn = Connection::open(db_path)?;
    conn.execute_batch(SQLITE_INIT_SCRIPT)?;
    let tx = conn.transaction()?;

    for p in packages {
        debug!("INSERT INTO packages (package, sha256, size, filename, version, architecture, repo) VALUES ({}, {}, {}, {}, {}, {}, {})", p.package, p.sha256, p.size, p.filename, p.version, p.architecture, p.repo);
        tx.execute("INSERT INTO packages (package, sha256, size, filename, version, architecture, repo) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", params![p.package, p.sha256, p.size, p.filename, p.version, p.architecture, p.repo]).context(format!("when processing {}", p.filename))?;
    }

    tx.commit()?;

    Ok(())
}

#[tokio::test]
async fn test_kernel_packages_to_retire() -> Result<()> {
    use bytesize::ByteSize;
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let db_url = if let Ok(v) = std::env::var("DB_URL") {
        v
    } else {
        "postgres:///packages".into()
    };

    let pool = sqlx::PgPool::connect(&db_url).await?;
    let collection = determine_retired_kernel_packages(&pool).await?;
    let mut total_retired_size: u64 = 0;
    for package in &collection {
        log::info!(
            "Will retire linux kernel version {} at {}",
            package.version,
            package.filename
        );
        total_retired_size += package.size as u64;
    }
    log::info!(
        "{} packages will be retired, freeing {} of space.",
        collection.len(),
        ByteSize::b(total_retired_size).to_string_as(true)
    );
    Ok(())
}
