//! Cross-module helpers for `bwoc-cli`. Time helpers live in `bwoc-core::time`
//! since both `bwoc-cli` and `bwoc-agent` consume them.

pub use bwoc_core::time::utc_now_iso8601;
