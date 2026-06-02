//! Opcode-shaped semantic logic must live only in
//! `crates/vm/src/vm/semantics/` and `crates/vm/src/vm/dispatch_handlers/`.
//!
//! Reads source files and rejects `op_xxx` function definitions outside those
//! two directories.

use std::fs;
use std::path::{Path, PathBuf};

const VM_SRC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src");

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read_dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_op_functions_outside_semantics_and_handlers() {
    let mut files = Vec::new();
    collect_rs(Path::new(VM_SRC), &mut files);

    let allowlist_prefixes = [
        format!("{VM_SRC}/vm/semantics/"),
        format!("{VM_SRC}/vm/dispatch_handlers/"),
        format!("{VM_SRC}/dsl/handlers/"),
    ];

    let mut offenders: Vec<(PathBuf, usize, String)> = Vec::new();
    for path in &files {
        let path_str = path.to_string_lossy();
        if allowlist_prefixes.iter().any(|p| path_str.starts_with(p)) {
            continue;
        }
        let body = fs::read_to_string(path).expect("read source");
        for (line_no, line) in body.lines().enumerate() {
            let trimmed = line.trim_start();
            if (trimmed.starts_with("pub fn op_")
                || trimmed.starts_with("pub(crate) fn op_")
                || trimmed.starts_with("pub(super) fn op_")
                || trimmed.starts_with("fn op_"))
                && trimmed.contains('(')
                // `op_xxx_slow` helpers are allowed in dispatch_handlers/ and dsl/handlers/.
                && !trimmed.contains("_slow(")
            {
                offenders.push((path.clone(), line_no + 1, trimmed.to_string()));
            }
        }
    }

    if !offenders.is_empty() {
        let report: Vec<String> = offenders
            .iter()
            .map(|(p, n, l)| format!("{}:{}: {}", p.display(), n, l))
            .collect();
        panic!(
            "Found op_* function(s) outside semantics/, dispatch_handlers/, or dsl/handlers/:\n{}",
            report.join("\n"),
        );
    }
}
