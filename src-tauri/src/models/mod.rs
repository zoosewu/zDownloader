// Models module - data structures owned by the shell (download data lives in zoo-core)
pub mod settings;

/// Wire contract: every enum's serialized value must match test-fixtures/api-samples.json,
/// which the frontend test suite checks against the TS types as well.
#[cfg(test)]
mod enum_contract_tests {
    use super::settings::{SpeedLimitMode, Theme};
    use serde::Serialize;
    use zoo_core::proxy_pool::ProxyStatus;
    use zoo_core::TaskState;

    fn fixture(name: &str) -> Vec<String> {
        let raw = include_str!("../../../test-fixtures/api-samples.json");
        let json: serde_json::Value = serde_json::from_str(raw).unwrap();
        serde_json::from_value(json["enums"][name].clone()).unwrap()
    }

    fn wire<T: Serialize>(values: &[T]) -> Vec<String> {
        values
            .iter()
            .map(|v| serde_json::to_value(v).unwrap().as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn enums_match_shared_fixture() {
        use TaskState::*;
        assert_eq!(
            wire(&[Scheduled, Queued, Running, Paused, Completed, Failed, Cancelled]),
            fixture("taskState")
        );
        assert_eq!(wire(&[Theme::Light, Theme::Dark]), fixture("theme"));
        assert_eq!(
            wire(&[SpeedLimitMode::Unlimited, SpeedLimitMode::Fixed]),
            fixture("speedLimitMode")
        );
        assert_eq!(
            wire(&[ProxyStatus::Unknown, ProxyStatus::Verifying, ProxyStatus::Active,
                   ProxyStatus::Failed, ProxyStatus::Blacklisted]),
            fixture("proxyStatus")
        );
    }
}
