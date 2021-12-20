use std::time::Duration;

use anyhow::{anyhow, Result};
use dbus::nonblock::{stdintf::org_freedesktop_dbus::Properties, Proxy, SyncConnection};
use dbus::Path;
use log::{error, info};

#[derive(Clone)]
pub struct ServiceState<'a> {
    id: String,
    state: String,
    triggered_by: Vec<String>,
    path: Path<'a>,
}

async fn get_services<'a, S: AsRef<str>>(
    conn: &'a SyncConnection,
    name: &[S],
) -> Result<Vec<Path<'a>>> {
    let proxy = Proxy::new(
        "org.freedesktop.systemd1",
        "/org/freedesktop/systemd1",
        Duration::from_secs(2),
        conn,
    );
    let mut results = Vec::new();
    for name in name.iter() {
        let (path,): (Path,) = proxy
            .method_call(
                "org.freedesktop.systemd1.Manager",
                "GetUnit",
                (name.as_ref(),),
            )
            .await?;
        results.push(path);
    }

    Ok(results)
}

async fn get_service_status<'a>(
    conn: &'a SyncConnection,
    service: Path<'a>,
) -> Result<ServiceState<'a>> {
    let proxy = Proxy::new(
        "org.freedesktop.systemd1",
        &service,
        Duration::from_secs(2),
        conn,
    );
    let props = proxy.get_all("org.freedesktop.systemd1.Unit").await?;
    let triggered_by = props.get("TriggeredBy").and_then(|x| x.0.as_iter());
    let state = props
        .get("SubState")
        .and_then(|x| x.0.as_str())
        .ok_or_else(|| anyhow!("SubState is missing"))?;
    let id = props
        .get("Id")
        .and_then(|x| x.0.as_str())
        .ok_or_else(|| anyhow!("Id is missing"))?;
    let triggered_by_string = if let Some(triggered_by) = triggered_by {
        triggered_by
            .filter_map(|x| x.as_str().and_then(|s| Some(s.to_string())))
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    Ok(ServiceState {
        id: id.to_string(),
        state: state.to_string(),
        triggered_by: triggered_by_string,
        path: service,
    })
}

async fn wait_for_service<'a>(conn: &'a SyncConnection, service: Path<'a>) -> Result<()> {
    let proxy = Proxy::new(
        "org.freedesktop.systemd1",
        &service,
        Duration::from_secs(2),
        conn,
    );
    loop {
        let (state,): (String,) = proxy
            .get("org.freedesktop.systemd1.Unit", "SubState")
            .await?;
        if state == "dead" || state == "waiting" {
            break;
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    Ok(())
}

async fn inhibit_service<'a>(conn: &'a SyncConnection, service: &ServiceState<'a>) -> Result<()> {
    let proxy = Proxy::new(
        "org.freedesktop.systemd1",
        &service.path,
        Duration::from_secs(2),
        conn,
    );

    proxy
        .method_call("org.freedesktop.systemd1.Unit", "Stop", ("replace",))
        .await?;

    Ok(())
}

pub async fn inhibit_services<'a, S: AsRef<str>>(
    conn: &'a SyncConnection,
    services: &[S],
) -> Result<Vec<ServiceState<'a>>> {
    let services = get_services(conn, services).await?;
    let mut states = Vec::new();
    let mut triggers: Vec<String> = Vec::new();
    for service in services {
        states.push(get_service_status(conn, service).await?);
    }
    for state in states.clone() {
        triggers.extend(state.triggered_by);
    }
    let triggers = get_services(conn, &triggers).await?;
    let mut active_triggers = Vec::new();
    // find all the active triggers
    for trigger in triggers {
        let trigger = get_service_status(conn, trigger).await?;
        if trigger.state == "running" || trigger.state == "waiting" {
            active_triggers.push(trigger);
        }
    }
    for trigger in active_triggers.iter() {
        info!("Inhibiting trigger {} ...", trigger.id);
        inhibit_service(conn, trigger).await?;
    }
    for srv in states {
        info!("Waiting for {} ...", srv.id);
        wait_for_service(conn, srv.path).await?;
    }

    Ok(active_triggers)
}

pub async fn restore_services<'a>(
    conn: &SyncConnection,
    services: &[ServiceState<'a>],
) -> Result<()> {
    for service in services {
        let proxy = Proxy::new(
            "org.freedesktop.systemd1",
            &service.path,
            Duration::from_secs(2),
            conn,
        );
        let result: Result<(Path,), dbus::Error> = proxy
            .method_call("org.freedesktop.systemd1.Unit", "Start", ("replace",))
            .await;
        if let Err(err) = result {
            error!("Failed to start unit {}: {}", service.id, err);
        }
    }

    Ok(())
}
