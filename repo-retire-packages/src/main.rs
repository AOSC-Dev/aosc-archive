use anyhow::Result;

mod abbs;
mod cli;
mod db;
mod dbus;
mod retire;

use clap::Parser;
use retire::retire_action;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();
    env_logger::init();

    match args {
        cli::Args::Retire(args) => {
            // handle inhibition
            let mut inhibited = None;
            let conn = zbus::Connection::system().await.unwrap();

            if !args.inhibit.is_empty() {
                inhibited = Some(dbus::inhibit_services(&conn, &args.inhibit).await.unwrap());
            }

            retire_action(
                args.config,
                args.dry_run,
                args.output,
                args.out_of_tree,
                args.with_kernel,
                args.database,
                args.abbs_dir,
            )
            .await?;
            // restore services
            if let Some(inhibit) = inhibited {
                dbus::restore_services(&inhibit).await?;
            }
        }
        cli::Args::Binning(_) => todo!(),
    }

    Ok(())
}
