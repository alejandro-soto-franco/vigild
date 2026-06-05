use crate::{error::Result, unit::UnitStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub host: String,
    pub timestamp_unix: u64,
    pub units: Vec<UnitStatus>,
}

pub fn encode_frame(report: &HealthReport) -> Result<Vec<u8>> {
    Ok(bincode::serialize(report)?)
}

pub fn decode_frame(bytes: &[u8]) -> Result<HealthReport> {
    Ok(bincode::deserialize(bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit::{ActiveState, UnitStatus};

    fn sample_report() -> HealthReport {
        HealthReport {
            host: "host-a".into(),
            timestamp_unix: 1_700_000_000,
            units: vec![UnitStatus {
                name: "sshd.service".into(),
                description: "SSH Daemon".into(),
                load_state: "loaded".into(),
                active: ActiveState::Active,
                sub_state: "running".into(),
            }],
        }
    }

    #[test]
    fn round_trip_encode_decode() {
        let report = sample_report();
        let encoded = encode_frame(&report).expect("encode");
        let decoded = decode_frame(&encoded).expect("decode");
        assert_eq!(decoded.host, report.host);
        assert_eq!(decoded.units.len(), 1);
        assert_eq!(decoded.units[0].name, "sshd.service");
    }

    #[test]
    fn decode_garbage_returns_error() {
        let result = decode_frame(b"not valid bincode garbage data !!!!");
        assert!(result.is_err());
    }
}
