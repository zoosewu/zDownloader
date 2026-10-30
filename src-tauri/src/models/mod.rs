// Models module - data structures for the application
pub mod download_segment;
pub mod download_task;
pub mod proxy_info;
pub mod settings;

pub use download_segment::*;
pub use download_task::*;
pub use proxy_info::*;

/// Wire contract: every enum's serialized value must match test-fixtures/api-samples.json,
/// which the frontend test suite checks against the TS enums as well.
#[cfg(test)]
mod enum_contract_tests {
    use super::settings::{SpeedLimitMode, Theme};
    use super::*;
    use serde::Serialize;

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
        use DownloadStatus::*;
        assert_eq!(
            wire(&[Pending, Initializing, Downloading, Paused, Cancelling, Cancelled,
                   Completed, Failed, VerifyingIntegrity, MergingSegments]),
            fixture("downloadStatus")
        );
        assert_eq!(
            wire(&[SegmentStatus::Pending, SegmentStatus::Downloading,
                   SegmentStatus::Completed, SegmentStatus::Failed]),
            fixture("segmentStatus")
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
