//! Raw key-stream output for piping into external matchers (e.g. brainflayer).
//!
//! Every generated private key is written to stdout as 64 lowercase hex
//! characters per line (no `0x` prefix) — the exact format brainflayer
//! expects for `-x -t priv`:
//!
//! ```text
//! vuke generate --transform=sha256 --stream-keys range --start 1 --end 1000000 \
//!   | brainflayer -x -t priv -b bitcoin.blf
//! ```
//!
//! Match hits are reported on stderr so stdout stays a clean key feed.
//! Output is buffered and flushed periodically so the consumer sees keys
//! in near real time without one syscall per key.

use anyhow::Result;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use super::Output;
use crate::derive::DerivedKey;
use crate::matcher::MatchInfo;

const HEX_LOWER: &[u8; 16] = b"0123456789abcdef";

/// Flush the stream after this many buffered keys (also flushed on drop of
/// the run via `flush()`), so pipe consumers receive keys in near real time.
const FLUSH_INTERVAL: usize = 8192;

/// Stream every generated private key to stdout, one 64-char lowercase hex
/// line per key. Match hits go to stderr, keeping stdout a clean key feed.
pub struct StreamOutput {
    writer: Mutex<BufWriter<Box<dyn Write + Send>>>,
    buffered: AtomicUsize,
}

impl StreamOutput {
    /// Create a stream output writing to stdout.
    pub fn new() -> Self {
        Self {
            writer: Mutex::new(BufWriter::with_capacity(
                64 * 1024,
                Box::new(std::io::stdout()),
            )),
            buffered: AtomicUsize::new(0),
        }
    }

    /// Create a stream output writing to an arbitrary writer (used in tests).
    pub fn to_writer(writer: Box<dyn Write + Send>) -> Self {
        Self {
            writer: Mutex::new(BufWriter::with_capacity(64 * 1024, writer)),
            buffered: AtomicUsize::new(0),
        }
    }

    /// Format a 32-byte key as 64 lowercase hex chars followed by a newline.
    fn format_line(raw: &[u8; 32], out: &mut [u8; 65]) {
        for (i, b) in raw.iter().enumerate() {
            out[i * 2] = HEX_LOWER[(b >> 4) as usize];
            out[i * 2 + 1] = HEX_LOWER[(b & 0x0f) as usize];
        }
        out[64] = b'\n';
    }
}

impl Default for StreamOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl Output for StreamOutput {
    /// Always stream every key, in addition to reporting hits via `hit()`.
    fn wants_all_keys(&self) -> bool {
        true
    }

    /// Raw keys only — no address data is consumed, so sources may skip
    /// derivation when streaming through this output without a matcher.
    fn needs_derivation(&self) -> bool {
        false
    }

    fn key(&self, _source: &str, _transform: &str, derived: &DerivedKey) -> Result<()> {
        let mut line = [0u8; 65];
        Self::format_line(&derived.raw, &mut line);

        let mut w = self.writer.lock().unwrap();
        w.write_all(&line)?;

        if self.buffered.fetch_add(1, Ordering::Relaxed) + 1 >= FLUSH_INTERVAL {
            self.buffered.store(0, Ordering::Relaxed);
            w.flush()?;
        }

        Ok(())
    }

    fn hit(
        &self,
        source: &str,
        transform: &str,
        derived: &DerivedKey,
        match_info: &MatchInfo,
    ) -> Result<()> {
        // Report matches on stderr so stdout stays a clean key stream.
        let mut err = std::io::stderr().lock();
        writeln!(err, "\n========== HIT ==========")?;
        writeln!(err, "Source: {}", source)?;
        writeln!(err, "Transform: {}", transform)?;
        writeln!(
            err,
            "Matched: {} ({})",
            match_info.address,
            match_info.address_type.as_str()
        )?;
        writeln!(err, "---")?;
        writeln!(err, "Private Key: {}", derived.private_key_hex)?;
        writeln!(err, "WIF (compressed): {}", derived.wif_compressed)?;
        writeln!(err, "---")?;
        writeln!(err, "P2PKH (compressed): {}", derived.p2pkh_compressed)?;
        writeln!(err, "P2WPKH: {}", derived.p2wpkh)?;
        writeln!(err, "=========================")?;
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        let mut w = self.writer.lock().unwrap();
        w.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn make_key(byte: u8) -> DerivedKey {
        DerivedKey {
            raw: [byte; 32],
            private_key_hex: format!("{:02x}", byte),
            private_key_decimal: "1".to_string(),
            private_key_binary: "1".to_string(),
            bit_length: 8,
            hamming_weight: 8,
            leading_zeros: 0,
            pubkey_compressed: "02abc".to_string(),
            pubkey_uncompressed: "04abc".to_string(),
            wif_compressed: "WIF_C".to_string(),
            wif_uncompressed: "WIF_U".to_string(),
            p2pkh_compressed: "1Address".to_string(),
            p2pkh_uncompressed: "1Uncompressed".to_string(),
            p2sh_p2wpkh: "3Wrapped".to_string(),
            p2wpkh: "bc1q".to_string(),
            p2tr: "bc1p".to_string(),
        }
    }

    fn make_temp() -> (NamedTempFile, std::fs::File) {
        let temp = NamedTempFile::new().unwrap();
        let writer = std::fs::OpenOptions::new()
            .write(true)
            .open(temp.path())
            .unwrap();
        (temp, writer)
    }

    #[test]
    fn writes_64_lowercase_hex_per_line() {
        let (temp, writer) = make_temp();
        let out = StreamOutput::to_writer(Box::new(writer));

        out.key("source", "sha256", &make_key(0xab)).unwrap();
        out.flush().unwrap();

        let text = std::fs::read_to_string(temp.path()).unwrap();
        let expected = format!("{}\n", "ab".repeat(32));
        assert_eq!(text, expected);
        assert!(text.lines().all(|l| l.len() == 64));
        assert_eq!(text, text.to_lowercase());
    }

    #[test]
    fn line_format_matches_known_hex() {
        let (temp, writer) = make_temp();
        let out = StreamOutput::to_writer(Box::new(writer));

        let mut key = make_key(0x00);
        key.raw[0] = 0xc4;
        key.raw[1] = 0xbb;
        key.raw[31] = 0x8a;
        out.key("s", "sha256", &key).unwrap();
        out.flush().unwrap();

        let text = std::fs::read_to_string(temp.path()).unwrap();
        assert!(text.starts_with("c4bb"));
        assert!(text.trim_end().ends_with("8a"));
        assert_eq!(text.trim_end().len(), 64);
    }

    #[test]
    fn flushes_at_interval() {
        let (temp, writer) = make_temp();
        let out = StreamOutput::to_writer(Box::new(writer));

        for _ in 0..FLUSH_INTERVAL {
            out.key("s", "sha256", &make_key(0x01)).unwrap();
        }

        // Exactly FLUSH_INTERVAL lines must already be visible without
        // calling flush() — the interval flush must have fired.
        let text = std::fs::read_to_string(temp.path()).unwrap();
        assert_eq!(text.len(), FLUSH_INTERVAL * 65);
    }
}
