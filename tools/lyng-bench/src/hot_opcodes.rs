//! Parser for `tools/lyng-bench/hot-opcodes.toml`.
//!
//! Consumed by `asm-diff`, `microbench`, and `--count-slow-path-share`.
//! The config is the single source of truth for which opcodes count
//! as "hot" and what their per-opcode invariant thresholds are.

use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct HotOpcodesConfig {
    pub meta: Meta,
    #[serde(default)]
    pub opcodes: Vec<OpcodeEntry>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Meta {
    pub source_data: String,
    pub refresh_command: String,
    pub default_target_slow_path_share: f64,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct OpcodeEntry {
    pub name: String,
    #[serde(default)]
    pub target_slow_path_share: Option<f64>,
    #[serde(default)]
    pub aarch64_max_instructions: Option<u32>,
    #[serde(default)]
    pub x86_64_max_instructions: Option<u32>,
}

impl HotOpcodesConfig {
    /// Load from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or the TOML cannot be parsed.
    pub fn load(path: &Path) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|err| format!("read {}: {err}", path.display()))?;
        toml::from_str::<Self>(&raw).map_err(|err| format!("parse {}: {err}", path.display()))
    }

    /// Effective slow-path share threshold for an opcode (override or default).
    #[must_use]
    pub fn target_slow_path_share(&self, opcode_name: &str) -> f64 {
        self.opcodes
            .iter()
            .find(|entry| entry.name == opcode_name)
            .and_then(|entry| entry.target_slow_path_share)
            .unwrap_or(self.meta.default_target_slow_path_share)
    }

    /// Hot-opcode name list, in config order.
    #[must_use]
    pub fn hot_opcode_names(&self) -> Vec<&str> {
        self.opcodes
            .iter()
            .map(|entry| entry.name.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_config() {
        let raw = r#"
            [meta]
            source_data = "x.tsv"
            refresh_command = "cmd"
            default_target_slow_path_share = 0.20

            [[opcodes]]
            name = "Move"
        "#;
        let config: HotOpcodesConfig = toml::from_str(raw).unwrap();
        assert_eq!(config.opcodes.len(), 1);
        assert_eq!(config.opcodes[0].name, "Move");
        assert!((config.target_slow_path_share("Move") - 0.20).abs() < 1e-9);
    }

    #[test]
    fn per_opcode_threshold_override_takes_precedence() {
        let raw = r#"
            [meta]
            source_data = "x"
            refresh_command = "y"
            default_target_slow_path_share = 0.20

            [[opcodes]]
            name = "GetNamedProperty"
            target_slow_path_share = 0.35
        "#;
        let config: HotOpcodesConfig = toml::from_str(raw).unwrap();
        assert!((config.target_slow_path_share("GetNamedProperty") - 0.35).abs() < 1e-9);
    }

    #[test]
    fn missing_opcode_falls_back_to_default() {
        let raw = r#"
            [meta]
            source_data = "x"
            refresh_command = "y"
            default_target_slow_path_share = 0.20
        "#;
        let config: HotOpcodesConfig = toml::from_str(raw).unwrap();
        assert!((config.target_slow_path_share("MissingOpcode") - 0.20).abs() < 1e-9);
    }

    #[test]
    fn parses_the_committed_hot_opcodes_toml() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("hot-opcodes.toml");
        let config = HotOpcodesConfig::load(&path).expect("load");
        // The hot-opcodes.toml tracks the top-30 V8 v7 opcodes plus any
        // macro-shared symmetric pairs deemed in-scope by per-phase
        // retrospectives. Phase 1.A landed 7 ports (including 5 adjacent-
        // family completions under the pre-rule); Phase 1.B added 9 + 2
        // backfill; Phase 1.C added 7; current count is 37. Upper bound
        // accommodates ~5 more for Phase 1.D + opportunistic pickups.
        assert!(
            config.opcodes.len() >= 25,
            "expected at least 25 hot opcodes, got {}",
            config.opcodes.len()
        );
        assert!(
            config.opcodes.len() <= 45,
            "expected at most 45 hot opcodes (top-30 + ~15 macro-shared/adjacent), got {}",
            config.opcodes.len()
        );
    }
}
