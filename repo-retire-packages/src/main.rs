use anyhow::Result;

mod cli;
mod dbus;
mod retire;
mod db;

use retire::retire_action;

#[tokio::main]
async fn main() -> Result<()> {
    let app = cli::build_cli();
    let args = app.get_matches();

    env_logger::init();
    match args.subcommand() {
        Some(("retire", args)) => {
            let inhibit = args.get_many::<String>("inhibit");
            let conn = zbus::Connection::system().await.unwrap();
            // handle inhibition
            let mut inhibited = None;
            if let Some(inhibit) = inhibit {
                inhibited = Some(
                    dbus::inhibit_services(&conn, &inhibit.collect::<Vec<_>>())
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
                dbus::restore_services(&inhibit).await?;
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
