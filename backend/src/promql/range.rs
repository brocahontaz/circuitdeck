//! Supported dashboard time ranges shared by all historical endpoints.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A dashboard time range: one of the fixed buckets the frontend offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeRange {
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "6h")]
    SixHours,
    #[serde(rename = "24h")]
    TwentyFourHours,
    #[serde(rename = "7d")]
    SevenDays,
}

impl TimeRange {
    /// Canonical string form used in URLs and error messages.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            TimeRange::OneHour => "1h",
            TimeRange::SixHours => "6h",
            TimeRange::TwentyFourHours => "24h",
            TimeRange::SevenDays => "7d",
        }
    }

    /// Range length in seconds.
    #[must_use]
    pub fn secs(self) -> i64 {
        match self {
            TimeRange::OneHour => 3600,
            TimeRange::SixHours => 6 * 3600,
            TimeRange::TwentyFourHours => 24 * 3600,
            TimeRange::SevenDays => 7 * 24 * 3600,
        }
    }

    /// Prometheus range-vector selector, e.g. `[1h]`.
    #[must_use]
    pub fn selector(self) -> &'static str {
        match self {
            TimeRange::OneHour => "[1h]",
            TimeRange::SixHours => "[6h]",
            TimeRange::TwentyFourHours => "[24h]",
            TimeRange::SevenDays => "[7d]",
        }
    }

    /// A sensible query step for graphing this range.
    #[must_use]
    pub fn step_secs(self) -> u32 {
        match self {
            TimeRange::OneHour => 30,
            TimeRange::SixHours => 120,
            TimeRange::TwentyFourHours => 300,
            TimeRange::SevenDays => 1800,
        }
    }

    /// Parses a range string such as `"6h"`.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "1h" => Some(TimeRange::OneHour),
            "6h" => Some(TimeRange::SixHours),
            "24h" => Some(TimeRange::TwentyFourHours),
            "7d" => Some(TimeRange::SevenDays),
            _ => None,
        }
    }

    /// All supported ranges, in display order.
    #[must_use]
    pub fn all() -> [TimeRange; 4] {
        [
            TimeRange::OneHour,
            TimeRange::SixHours,
            TimeRange::TwentyFourHours,
            TimeRange::SevenDays,
        ]
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_ranges() {
        assert_eq!(TimeRange::parse("1h"), Some(TimeRange::OneHour));
        assert_eq!(TimeRange::parse("6h"), Some(TimeRange::SixHours));
        assert_eq!(TimeRange::parse("24h"), Some(TimeRange::TwentyFourHours));
        assert_eq!(TimeRange::parse("7d"), Some(TimeRange::SevenDays));
        assert_eq!(TimeRange::parse("3w"), None);
        assert_eq!(TimeRange::parse(""), None);
        assert_eq!(TimeRange::parse("1H"), None);
    }

    #[test]
    fn secs_match_ranges() {
        assert_eq!(TimeRange::OneHour.secs(), 3600);
        assert_eq!(TimeRange::SevenDays.secs(), 7 * 24 * 3600);
    }

    #[test]
    fn steps_increase_with_range() {
        assert!(TimeRange::OneHour.step_secs() <= TimeRange::SevenDays.step_secs());
    }

    #[test]
    fn serde_round_trip() {
        let json = serde_json::to_string(&TimeRange::TwentyFourHours).unwrap();
        assert_eq!(json, "\"24h\"");
        let back: TimeRange = serde_json::from_str(&json).unwrap();
        assert_eq!(back, TimeRange::TwentyFourHours);
    }
}
