use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActiveState {
    Active,
    Inactive,
    Activating,
    Deactivating,
    Failed,
    Other(String),
}

impl From<&str> for ActiveState {
    fn from(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "inactive" => Self::Inactive,
            "activating" => Self::Activating,
            "deactivating" => Self::Deactivating,
            "failed" => Self::Failed,
            other => Self::Other(other.to_string()),
        }
    }
}

impl From<String> for ActiveState {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitStatus {
    pub name: String,
    pub description: String,
    pub load_state: String,
    pub active: ActiveState,
    pub sub_state: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_state_from_str_active() {
        let s = ActiveState::from("active");
        assert!(matches!(s, ActiveState::Active));
    }

    #[test]
    fn active_state_from_str_unknown() {
        let s = ActiveState::from("reloading");
        assert!(matches!(s, ActiveState::Other(ref x) if x == "reloading"));
    }

    #[test]
    fn unit_status_display() {
        let u = UnitStatus {
            name: "sshd.service".into(),
            description: "OpenSSH Daemon".into(),
            load_state: "loaded".into(),
            active: ActiveState::Active,
            sub_state: "running".into(),
        };
        assert_eq!(u.name, "sshd.service");
        assert!(matches!(u.active, ActiveState::Active));
    }
}
