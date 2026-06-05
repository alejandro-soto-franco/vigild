use std::collections::HashMap;
use vigild_core::peer::HealthReport;

#[derive(Default)]
pub struct SharedState {
    reports: HashMap<String, HealthReport>,
}

impl SharedState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, report: HealthReport) {
        self.reports.insert(report.host.clone(), report);
    }

    #[allow(dead_code)]
    pub fn snapshot(&self) -> Vec<HealthReport> {
        self.reports.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vigild_core::{
        peer::HealthReport,
        unit::{ActiveState, UnitStatus},
    };

    fn make_report(host: &str, unit: &str, active: ActiveState) -> HealthReport {
        HealthReport {
            host: host.into(),
            timestamp_unix: 1_700_000_000,
            units: vec![UnitStatus {
                name: unit.into(),
                description: String::new(),
                load_state: "loaded".into(),
                active,
                sub_state: String::new(),
            }],
        }
    }

    #[test]
    fn merge_two_hosts() {
        let mut state = SharedState::new();
        state.update(make_report("host-a", "sshd.service", ActiveState::Active));
        state.update(make_report("host-b", "sshd.service", ActiveState::Failed));
        let snapshot = state.snapshot();
        assert_eq!(snapshot.len(), 2);
        let a = snapshot.iter().find(|r| r.host == "host-a").unwrap();
        assert!(matches!(a.units[0].active, ActiveState::Active));
    }

    #[test]
    fn update_replaces_stale_report_for_same_host() {
        let mut state = SharedState::new();
        state.update(make_report("host-a", "sshd.service", ActiveState::Failed));
        state.update(make_report("host-a", "sshd.service", ActiveState::Active));
        let snapshot = state.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert!(matches!(snapshot[0].units[0].active, ActiveState::Active));
    }
}
