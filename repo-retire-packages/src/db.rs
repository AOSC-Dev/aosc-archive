use anyhow::Result;
use sqlx::{query_as, PgPool};

#[derive(Debug, Clone)]
pub struct PackageMeta {
    pub package: String,
    pub sha256: String,
    pub size: i64,
    pub filename: String,
    pub version: String,
}

pub async fn determine_retired_packages(pool: &PgPool, oot: bool) -> Result<Vec<PackageMeta>> {
    let packages = query_as!(
        PackageMeta,
        r#"SELECT package, sha256, size, filename, version FROM 
(SELECT *, rank() OVER (PARTITION BY package, repo ORDER BY _vercomp DESC) AS pos FROM pv_packages) 
AS sq WHERE pos > 1"#
    )
    .fetch_all(pool)
    .await?;

    if oot {
        let mut oot_packages = query_as!(
            PackageMeta,
        r#"SELECT DISTINCT pp.package, pp.sha256, pp.size, pp.filename, pp.version FROM 
pv_packages pp LEFT JOIN packages p ON pp.package = p.name WHERE 
tree IS NULL AND pp.package NOT LIKE '%-dbg' AND pp.package NOT SIMILAR TO '(linux-kernel-|linux\+kernel\+|u-boot)%'"#).fetch_all(pool).await?;
        oot_packages.extend(packages);
        return Ok(oot_packages);
    }

    Ok(packages)
}
