//! Centralized PromQL definitions and time-range handling.

pub mod query;
pub mod range;

pub use self::query::Query;
pub use self::range::TimeRange;
