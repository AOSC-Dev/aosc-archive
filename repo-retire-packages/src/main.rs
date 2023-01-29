use anyhow::Result;
use dbus_tokio::connection;
use log::error;

mod cli;
mod dbus;
mod retire;

use retire::retire_action;

#[tokio::main]
async fn main() -> Result<()> {
    let app = cli::build_cli();
    let args = app.get_matches();

    env_logger::init();
    match args.subcommand() {
        Some(("retire", args)) => {
            let inhibit = args.get_many::<String>("inhibit");
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
                args.get_one::<String>("config").unwrap(),
                args.contains_id("dry-run"),
                args.get_one::<String>("output").unwrap(),
                args.contains_id("out-of-tree"),
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
