use crate::{
    error::Result,
    unit::{ActiveState, UnitStatus},
};
use zbus::{proxy, Connection};

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
trait Manager {
    #[allow(clippy::type_complexity)]
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

    fn load_unit(&self, name: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
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
    #[zbus(property, name = "Id")]
    fn id(&self) -> zbus::Result<String>;
    #[zbus(property, name = "Description")]
    fn description(&self) -> zbus::Result<String>;
    #[zbus(property, name = "LoadState")]
    fn load_state(&self) -> zbus::Result<String>;
    #[zbus(property, name = "ActiveState")]
    fn active_state(&self) -> zbus::Result<String>;
    #[zbus(property, name = "SubState")]
    fn sub_state(&self) -> zbus::Result<String>;
}

pub async fn connect_system_bus() -> Result<Connection> {
    Ok(Connection::system().await?)
}

pub async fn query_units(conn: &Connection) -> Result<Vec<UnitStatus>> {
    let manager = ManagerProxy::new(conn).await?;
    let raw = manager.list_units().await?;
    let units = raw
        .into_iter()
        .map(
            |(name, description, load_state, active_state, sub_state, ..)| UnitStatus {
                name,
                description,
                load_state,
                active: ActiveState::from(active_state),
                sub_state,
            },
        )
        .collect();
    Ok(units)
}

pub async fn query_watched_units(conn: &Connection, watch: &[String]) -> Result<Vec<UnitStatus>> {
    let manager = ManagerProxy::new(conn).await?;
    let mut out = Vec::with_capacity(watch.len());
    for name in watch {
        match manager.load_unit(name).await {
            Ok(path) => {
                let unit = UnitProxy::builder(conn).path(path)?.build().await?;
                let canonical = unit.id().await.unwrap_or_else(|_| name.clone());
                let load_state = unit
                    .load_state()
                    .await
                    .unwrap_or_else(|_| "not-found".into());
                if load_state == "not-found" {
                    out.push(UnitStatus {
                        name: canonical,
                        description: String::new(),
                        load_state,
                        active: ActiveState::from("inactive"),
                        sub_state: String::new(),
                    });
                    continue;
                }
                let description = unit.description().await.unwrap_or_default();
                let active_state = unit
                    .active_state()
                    .await
                    .unwrap_or_else(|_| "inactive".into());
                let sub_state = unit.sub_state().await.unwrap_or_default();
                out.push(UnitStatus {
                    name: canonical,
                    description,
                    load_state,
                    active: ActiveState::from(active_state),
                    sub_state,
                });
            }
            Err(_) => {
                out.push(UnitStatus {
                    name: name.clone(),
                    description: String::new(),
                    load_state: "not-found".into(),
                    active: ActiveState::from("inactive"),
                    sub_state: String::new(),
                });
            }
        }
    }
    Ok(out)
}

pub struct UnitDeps {
    pub after: Vec<String>,
    pub requires: Vec<String>,
    pub wants: Vec<String>,
}

pub async fn query_deps(conn: &Connection, unit_name: &str) -> Result<UnitDeps> {
    let manager = ManagerProxy::new(conn).await?;
    let path = manager.get_unit(unit_name).await?;
    let unit = UnitProxy::builder(conn).path(path)?.build().await?;
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
        assert!(
            !units.is_empty(),
            "systemd must have at least one active unit"
        );
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

    #[tokio::test]
    async fn query_watched_reports_inactive_unit() {
        if std::env::var("SKIP_DBUS_TESTS").is_ok() {
            return;
        }
        let conn = connect_system_bus().await.expect("D-Bus connection");
        let watch = vec!["emergency.service".to_string()];
        let units = query_watched_units(&conn, &watch)
            .await
            .expect("query_watched_units");
        assert_eq!(
            units.len(),
            1,
            "must report watched unit even when inactive"
        );
        assert_eq!(units[0].name, "emergency.service");
        assert!(
            matches!(
                units[0].active,
                ActiveState::Inactive | ActiveState::Other(_)
            ),
            "emergency.service should be inactive, got {:?}",
            units[0].active
        );
    }

    #[tokio::test]
    async fn query_watched_resolves_alias() {
        if std::env::var("SKIP_DBUS_TESTS").is_ok() {
            return;
        }
        let conn = connect_system_bus().await.expect("D-Bus connection");
        let watch = vec!["dbus.service".to_string()];
        let units = query_watched_units(&conn, &watch)
            .await
            .expect("query_watched_units");
        assert_eq!(units.len(), 1, "alias must resolve to one unit");
        assert_ne!(
            units[0].load_state, "not-found",
            "dbus.service alias must resolve to a loaded unit"
        );
        assert!(matches!(units[0].active, ActiveState::Active));
    }

    #[tokio::test]
    async fn query_watched_handles_nonexistent() {
        if std::env::var("SKIP_DBUS_TESTS").is_ok() {
            return;
        }
        let conn = connect_system_bus().await.expect("D-Bus connection");
        let watch = vec!["definitely-not-a-real-unit.service".to_string()];
        let units = query_watched_units(&conn, &watch)
            .await
            .expect("query_watched_units should not error on missing units");
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].load_state, "not-found");
    }
}
