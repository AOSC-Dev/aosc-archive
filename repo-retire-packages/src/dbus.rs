use std::time::Duration;

use anyhow::Result;
use log::{error, info};
use zbus::{zvariant::OwnedObjectPath, Connection};

#[derive(Clone)]
pub struct ServiceState<'a> {
    id: String,
    state: String,
    triggered_by: Vec<String>,
    proxy: SystemdUnitProxy<'a>,
}

#[zbus::dbus_proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
trait SystemdManager {
    fn get_unit(&self, name: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[zbus::dbus_proxy(interface = "org.freedesktop.systemd1.Unit", assume_defaults = true)]
trait SystemdUnit {
    fn start(&self, mode: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    fn stop(&self, mode: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    #[dbus_proxy(property)]
    fn sub_state(&self) -> zbus::Result<String>;

    #[dbus_proxy(property)]
    fn id(&self) -> zbus::Result<String>;

    #[dbus_proxy(property)]
    fn triggered_by(&self) -> zbus::Result<Vec<String>>;
}

async fn get_services<S: AsRef<str>>(
    conn: &Connection,
    name: &[S],
) -> Result<Vec<OwnedObjectPath>> {
    let proxy = SystemdManagerProxy::new(conn).await?;
    let mut results = Vec::new();
    for name in name.iter() {
        let path = proxy.get_unit(name.as_ref()).await?;
        results.push(path);
    }

    Ok(results)
}

async fn get_service_status(conn: &Connection, service: OwnedObjectPath) -> Result<ServiceState> {
    let proxy = SystemdUnitProxy::builder(conn)
        .path(service)?
        .build()
        .await?;
    let triggered_by = proxy.triggered_by().await?;
    let state = proxy.sub_state().await?;
    let id = proxy.id().await?;

    Ok(ServiceState {
        id,
        state,
        triggered_by,
        proxy,
    })
}

async fn wait_for_service(proxy: &SystemdUnitProxy<'_>) -> Result<()> {
    loop {
        let state = proxy.sub_state().await?;
        if state == "dead" || state == "waiting" {
            break;
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    Ok(())
}

#[inline]
async fn inhibit_service(service: &ServiceState<'_>) -> zbus::Result<OwnedObjectPath> {
    service.proxy.stop("replace").await
}

pub async fn inhibit_services<'a, S: AsRef<str>>(
    conn: &'a Connection,
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
        inhibit_service(trigger).await?;
    }
    for srv in states {
        info!("Waiting for {} ...", srv.id);
        wait_for_service(&srv.proxy).await?;
    }

    Ok(active_triggers)
}

pub async fn restore_services(services: &[ServiceState<'_>]) -> Result<()> {
    for service in services {
        let proxy = &service.proxy;
        let result = proxy.start("replace").await;
        if let Err(err) = result {
            error!("Failed to start unit {}: {}", service.id, err);
        }
    }

    Ok(())
}
