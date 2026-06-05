use crate::error::{CoreError, Result};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub message: String,
    pub priority: Option<String>,
    pub timestamp: Option<String>,
}

pub async fn tail_journal(unit: &str, n: usize) -> Result<Vec<JournalEntry>> {
    let output = Command::new("journalctl")
        .args(["-u", unit, "-n", &n.to_string(), "-o", "json", "--no-pager"])
        .output()
        .await
        .map_err(|e| CoreError::Journal(format!("journalctl exec failed: {e}")))?;

    if !output.status.success() && output.stdout.is_empty() {
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
            entries.push(JournalEntry {
                message: val["MESSAGE"].as_str().unwrap_or("").to_string(),
                priority: val["PRIORITY"].as_str().map(|s| s.to_string()),
                timestamp: val["__REALTIME_TIMESTAMP"].as_str().map(|s| s.to_string()),
            });
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tail_journal_returns_entries() {
        if std::env::var("SKIP_JOURNAL_TESTS").is_ok() {
            return;
        }
        let entries = tail_journal("systemd-journald.service", 5).await;
        assert!(entries.is_ok(), "journal tail failed: {:?}", entries);
    }

    #[tokio::test]
    async fn tail_journal_unknown_unit_returns_empty() {
        if std::env::var("SKIP_JOURNAL_TESTS").is_ok() {
            return;
        }
        let entries = tail_journal("definitely-does-not-exist-xyz.service", 5)
            .await
            .unwrap();
        assert!(entries.is_empty());
    }
}
