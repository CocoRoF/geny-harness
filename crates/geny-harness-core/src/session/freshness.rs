//! Session health evaluation.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Session freshness status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FreshnessStatus {
    /// Normal operation.
    Fresh,
    /// Idle timeout approaching.
    StaleWarn,
    /// Idle for extended period.
    StaleIdle,
    /// Many messages accumulated.
    StaleCompact,
    /// Max age or message count exceeded.
    StaleReset,
}

impl FreshnessStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FreshnessStatus::Fresh => "fresh",
            FreshnessStatus::StaleWarn => "stale_warn",
            FreshnessStatus::StaleIdle => "stale_idle",
            FreshnessStatus::StaleCompact => "stale_compact",
            FreshnessStatus::StaleReset => "stale_reset",
        }
    }

    pub fn should_revive(&self) -> bool {
        matches!(self, FreshnessStatus::StaleIdle)
    }

    pub fn should_compact(&self) -> bool {
        matches!(self, FreshnessStatus::StaleCompact)
    }

    pub fn should_reset(&self) -> bool {
        matches!(self, FreshnessStatus::StaleReset)
    }
}

/// Session freshness thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshnessPolicy {
    pub idle_timeout_secs: i64,
    pub warn_threshold_secs: i64,
    pub compact_message_count: usize,
    pub reset_message_count: usize,
    pub max_age_secs: i64,
}

impl Default for FreshnessPolicy {
    fn default() -> Self {
        Self {
            idle_timeout_secs: 30 * 60,      // 30 minutes
            warn_threshold_secs: 20 * 60,    // 20 minutes
            compact_message_count: 100,
            reset_message_count: 500,
            max_age_secs: 4 * 60 * 60,       // 4 hours
        }
    }
}

impl FreshnessPolicy {
    pub fn evaluate(
        &self,
        created_at: DateTime<Utc>,
        last_active: DateTime<Utc>,
        message_count: usize,
    ) -> FreshnessStatus {
        let now = Utc::now();
        let age = now - created_at;
        let idle = now - last_active;

        // Priority: max_age > reset_message_count > compact_message_count > idle > warn > FRESH
        if age > Duration::seconds(self.max_age_secs) {
            return FreshnessStatus::StaleReset;
        }
        if message_count >= self.reset_message_count {
            return FreshnessStatus::StaleReset;
        }
        if message_count >= self.compact_message_count {
            return FreshnessStatus::StaleCompact;
        }
        if idle > Duration::seconds(self.idle_timeout_secs) {
            return FreshnessStatus::StaleIdle;
        }
        if idle > Duration::seconds(self.warn_threshold_secs) {
            return FreshnessStatus::StaleWarn;
        }
        FreshnessStatus::Fresh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freshness_defaults() {
        let policy = FreshnessPolicy::default();
        let now = Utc::now();
        assert_eq!(policy.evaluate(now, now, 0), FreshnessStatus::Fresh);
    }

    #[test]
    fn test_freshness_stale_reset_message_count() {
        let policy = FreshnessPolicy::default();
        let now = Utc::now();
        assert_eq!(
            policy.evaluate(now, now, 500),
            FreshnessStatus::StaleReset
        );
    }

    #[test]
    fn test_freshness_stale_compact() {
        let policy = FreshnessPolicy::default();
        let now = Utc::now();
        assert_eq!(
            policy.evaluate(now, now, 100),
            FreshnessStatus::StaleCompact
        );
    }

    #[test]
    fn test_freshness_status_properties() {
        assert!(FreshnessStatus::StaleIdle.should_revive());
        assert!(!FreshnessStatus::Fresh.should_revive());
        assert!(FreshnessStatus::StaleCompact.should_compact());
        assert!(FreshnessStatus::StaleReset.should_reset());
    }
}
