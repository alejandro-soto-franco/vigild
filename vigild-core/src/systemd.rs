use zbus::{proxy, Connection};
use crate::{error::Result, unit::{ActiveState, UnitStatus}};

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
trait Manager {
    fn list_units(
        &self,
    ) -> zbus::Result<
        Vec<(
            String,
            String,
            String,
            String,
            String,
            String,
            zbus::zvariant::OwnedObjectPath,
            u32,
            String,
            zbus::zvariant::OwnedObjectPath,
        )>,
    >;

    fn get_unit(&self, name: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.systemd1.Unit",
    default_service = "org.freedesktop.systemd1"
)]
trait Unit {
    #[zbus(property)]
    fn after(&self) -> zbus::Result<Vec<String>>;
    #[zbus(property)]
    fn requires(&self) -> zbus::Result<Vec<String>>;
    #[zbus(property)]
    fn wants(&self) -> zbus::Result<Vec<String>>;
}

pub async fn connect_system_bus() -> Result<Connection> {
    Ok(Connection::system().await?)
}

pub async fn query_units(conn: &Connection) -> Result<Vec<UnitStatus>> {
    let manager = ManagerProxy::new(conn).await?;
    let raw = manager.list_units().await?;
    let units = raw
        .into_iter()
        .map(|(name, description, load_state, active_state, sub_state, ..)| UnitStatus {
            name,
            description,
            load_state,
            active: ActiveState::from(active_state),
            sub_state,
        })
        .collect();
    Ok(units)
}

pub struct UnitDeps {
    pub after: Vec<String>,
    pub requires: Vec<String>,
    pub wants: Vec<String>,
}

pub async fn query_deps(conn: &Connection, unit_name: &str) -> Result<UnitDeps> {
    let manager = ManagerProxy::new(conn).await?;
    let path = manager.get_unit(unit_name).await?;
    let unit = UnitProxy::builder(conn)
        .path(path)?
        .build()
        .await?;
    Ok(UnitDeps {
        after: unit.after().await?,
        requires: unit.requires().await?,
        wants: unit.wants().await?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn query_units_returns_nonempty() {
        if std::env::var("SKIP_DBUS_TESTS").is_ok() {
            return;
        }
        let conn = connect_system_bus().await.expect("D-Bus connection");
        let units = query_units(&conn).await.expect("query_units");
        assert!(!units.is_empty(), "systemd must have at least one active unit");
        for u in &units {
            assert!(!u.name.is_empty());
        }
    }

    #[tokio::test]
    async fn query_deps_for_sshd() {
        if std::env::var("SKIP_DBUS_TESTS").is_ok() {
            return;
        }
        let conn = connect_system_bus().await.expect("D-Bus connection");
        let units = query_units(&conn).await.unwrap();
        let ssh = units.iter().find(|u| u.name.contains("ssh"));
        if ssh.is_none() {
            return;
        }
        let deps = query_deps(&conn, &ssh.unwrap().name).await;
        assert!(deps.is_ok());
    }
}
