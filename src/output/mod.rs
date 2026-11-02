//! Output handlers for generated keys.

mod console;
mod multi;
mod stream;
#[cfg(feature = "storage-query")]
mod query_format;
#[cfg(feature = "storage")]
mod storage;

pub use console::ConsoleOutput;
pub use multi::MultiOutput;
pub use stream::StreamOutput;
#[cfg(feature = "storage-query")]
pub use query_format::{format_csv, format_json, format_schema, format_table, OutputFormat};
#[cfg(feature = "storage")]
pub use storage::{StorageOutput, StorageSummary};

use crate::derive::DerivedKey;
use crate::matcher::MatchInfo;
use anyhow::Result;

pub(crate) fn escape_csv_field(field: &str) -> String {
    let has_boundary_whitespace = field.chars().next().is_some_and(char::is_whitespace)
        || field.chars().last().is_some_and(char::is_whitespace);

    if has_boundary_whitespace
        || field.contains(',')
        || field.contains('"')
        || field.contains('\n')
        || field.contains('\r')
    {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// Output trait for handling generated keys.
pub trait Output: Send + Sync {
    /// Whether this output wants every generated key written via `key()`
    /// even when a matcher is active (streaming mode). Sources call `key()`
    /// for all keys when this returns true, and still report matches via
    /// `hit()` in addition. Default: false (only unmatched keys are written).
    fn wants_all_keys(&self) -> bool {
        false
    }

    /// Whether this output consumes the derived address data. When false and
    /// no matcher is active, sources may skip public key/address derivation
    /// entirely and pass `DerivedKey::raw_only` (raw streaming to external
    /// matchers such as brainflayer, which derive addresses themselves).
    fn needs_derivation(&self) -> bool {
        true
    }

    /// Output a key (no matcher, output all keys).
    fn key(&self, source: &str, transform: &str, derived: &DerivedKey) -> Result<()>;

    /// Output a match hit (matcher found target).
    fn hit(
        &self,
        source: &str,
        transform: &str,
        derived: &DerivedKey,
        match_info: &MatchInfo,
    ) -> Result<()>;

    /// Flush any buffered output.
    fn flush(&self) -> Result<()>;
}
