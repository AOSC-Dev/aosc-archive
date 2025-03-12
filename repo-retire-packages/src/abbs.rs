use anyhow::{anyhow, bail, Result};
use log::{error, info, warn};
use sqlx::{prelude::FromRow, Pool, Postgres, Transaction};
use std::{collections::HashMap, fs::read_to_string, path::Path, process::Command};

/// Represents a row in the table public.packages.
#[derive(FromRow, Clone, Debug)]
struct PackageEntry {
    name: String,
    tree: String,
    category: String,
    section: String,
    pkg_section: String,
    directory: String,
    description: String,
}

impl PackageEntry {
    fn from(defines: &Path, rel_path: &Path) -> Result<PackageEntry> {
        assert!(defines.is_file());
        let rel_dir = rel_path.parent().unwrap().parent().unwrap();
        let defines_content = read_to_string(defines)?;
        let mut pkg = PackageEntry {
            name: String::new(),
            tree: "aosc-os-abbs".to_string(),
            category: String::new(),
            section: String::new(),
            pkg_section: String::new(),
            directory: String::new(),
            description: String::new(),
        };
        fn quotes(c: char) -> bool {
            c == '\'' || c == '"'
        }
        for line in defines_content.lines() {
            // Dumb way to parse the defines file!
            // But we don't have to worry since we only need a few things
            // that is easy to get.
            if let Some((k, v)) = line.trim().split_once('=') {
                let field = &mut match k {
                    "PKGNAME" => &mut pkg.name,
                    "PKGSEC" => &mut pkg.pkg_section,
                    "PKGDES" => &mut pkg.description,
                    _ => continue,
                };
                field.push_str(v.trim().trim_matches(quotes));
            }
        }
        let category_dirname = rel_dir.parent().unwrap().file_name().unwrap();
        let (category, section) = category_dirname.to_str().unwrap().split_once('-').unwrap();
        pkg.category.push_str(category);
        pkg.section.push_str(section);
        pkg.directory.push_str(rel_dir.to_str().unwrap());
        Ok(pkg)
    }
}

/// Scans the entire ABBS tree, truncates the `public.packages` table, push back the results.
/// This keeps the table updated.
fn scan_tree(abbs_dir: &dyn AsRef<Path>) -> Result<Vec<PackageEntry>> {
    let mut hash_collection: HashMap<String, PackageEntry> = HashMap::new();
    let walker = walkdir::WalkDir::new(abbs_dir)
        .max_depth(4)
        .same_file_system(true);
    for entry in walker.into_iter() {
        let entry = entry?;
        if !(entry.file_type().is_file() && entry.file_name() == "defines") {
            continue;
        }
        let path = entry.path();
        let rel_path = path.strip_prefix(abbs_dir)?;
        if rel_path.is_absolute() {
            bail!("Unable to resolve the relative path of {}", path.display());
        }
        let pkg = PackageEntry::from(path, rel_path)?;
        if hash_collection.contains_key(&pkg.name) {
            let conflict = hash_collection.get(&pkg.name).unwrap();
            warn!("Conflict detected:");
            warn!("- Package {} registed at {}", &pkg.name, &pkg.directory);
            warn!(
                "- Package {} registed at {}",
                &conflict.name, &conflict.directory
            );
        }
        hash_collection.insert(pkg.name.clone(), pkg.clone());
    }
    let collection: Vec<PackageEntry> = hash_collection.into_values().collect();
    info!(
        "abbs: Done, {} packages in the collection.",
        collection.len()
    );
    Ok(collection)
}

async fn perform_update_transactions(
    pool: &Pool<Postgres>,
    collection: Vec<PackageEntry>,
) -> Result<()> {
    let mut tx = pool.begin().await?;
    // Enclose it into a fn so that we can roll back the transaction
    // before we quit in case of error.
    async fn perform_transaction(
        collection: Vec<PackageEntry>,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<()> {
        info!("Expected to insert {} entries.", collection.len());
        sqlx::query!("TRUNCATE TABLE public.packages")
            .execute(&mut **tx)
            .await?;
        for package in collection {
            sqlx::query!(
                r#"INSERT INTO public.packages (
                    name, tree, category, section, pkg_section, directory, description
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7
                )"#,
                package.name,
                package.tree,
                package.category,
                package.section,
                package.pkg_section,
                package.directory,
                package.description
            )
            .execute(&mut **tx)
            .await?;
        }
        Ok(())
    }

    info!("Performing the transaction ...");
    match perform_transaction(collection, &mut tx).await {
        Ok(_) => {
            info!("Commiting transaction ...");
            tx.commit().await?;
        }
        Err(e) => {
            error!("Database returned an error while performing the transaction:");
            error!("{}", e);
            e.chain().skip(1).for_each(|cause| {
                error!("Caused by:");
                error!("\t{}", cause);
            });
            tx.rollback().await.unwrap_or_else(|_| {
                panic!("Unable to rollback the transaction!");
            });
            bail!("Can not perform the transcations.");
        }
    }
    Ok(())
}

pub async fn update_abbs_database(pool: &Pool<Postgres>, abbs_dir: &dyn AsRef<Path>) -> Result<()> {
    // Update ABBS tree first.
    info!("abbs: Updating ABBS tree ...");
    let mut cmd_git_fetch = Command::new("git");
    let cmd_git_fetch = cmd_git_fetch
        .arg("fetch")
        .arg("origin")
        .current_dir(abbs_dir);
    let mut cmd_git_reset = Command::new("git");
    let cmd_git_reset = cmd_git_reset
        .arg("reset")
        .arg("--hard")
        .arg("remotes/origin/stable")
        .current_dir(abbs_dir);
    let result = cmd_git_fetch.status()?;
    if !result.success() {
        return Err(anyhow!(
            "Failed to run `git fetch', check your Internet connectivity."
        ));
    }
    let result = cmd_git_reset.status()?;
    if !result.success() {
        return Err(anyhow!(
            "Failed to run `git reset --hard origin/stable', check your repository at {}.",
            abbs_dir.as_ref().display()
        ));
    }

    // Scan the ABBS tree.
    info!("abbs: Scanning the ABBS tree for in-tree packages ...");
    let collection = scan_tree(abbs_dir)?;

    info!("Pushing in-tree packages into the database ...");
    perform_update_transactions(pool, collection).await?;
    Ok(())
}

#[test]
fn test_scan_tree() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let abbs_dir = if let Ok(v) = std::env::var("ABBS_DIR") {
        v
    } else {
        "aosc-os-abbs".into()
    };
    let abbs_dir = Path::new(&abbs_dir);
    let abbs_dir = abbs_dir.canonicalize()?;
    let _ = scan_tree(&abbs_dir);
    Ok(())
}

#[tokio::test]
async fn test_update_db() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let abbs_dir = if let Ok(v) = std::env::var("ABBS_DIR") {
        v
    } else {
        "aosc-os-abbs".into()
    };
    let db_url = if let Ok(v) = std::env::var("DB_URL") {
        v
    } else {
        "postgres:///packages".into()
    };
    let abbs_dir = Path::new(&abbs_dir);
    if !abbs_dir.is_dir() {
        bail!(
            "{} is either nonexistant or not a directory.",
            abbs_dir.display()
        );
    }
    let abbs_dir = abbs_dir.canonicalize()?;
    let pool = sqlx::PgPool::connect(&db_url).await?;
    update_abbs_database(&pool, &abbs_dir).await?;
    Ok(())
}
