use std::path::Path;

use anyhow::{Context, Result};
use log::debug;
use rusqlite::{params, Connection};
use sqlx::{query_as, PgPool};

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
tree IS NULL AND pp.package NOT LIKE '%-dbg' AND pp.package NOT SIMILAR TO '(linux-kernel-|linux\+kernel\+|u-boot)%'"#).fetch_all(pool).await?;
        oot_packages.extend(packages);
        return Ok(oot_packages);
    }

    Ok(packages)
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
