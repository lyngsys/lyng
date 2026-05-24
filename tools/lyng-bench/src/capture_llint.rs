//! `lyng-bench capture-llint` — extract JSC `LLInt` handler asm/source.
//!
//! Source-mode strategy:
//! - `auto`: try system → local → excerpt in order; report which mode produced each opcode.
//! - `system`: `otool -tvV` on the system JSC binary; finds `_llint_op_*` symbols.
//! - `local`: same approach but on a locally built JSC binary.
//! - `excerpt`: parse JSC's offlineasm source files directly; produces
//!   source-level reference instead of concrete asm.

use std::fmt::Write as _;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureLlintOptions {
    pub source: Source,
    pub jsc_binary: Option<PathBuf>,
    pub jsc_source: Option<PathBuf>,
    pub opcodes: Vec<String>,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Auto,
    System,
    Local,
    Excerpt,
}

/// Run capture-llint.
///
/// # Errors
///
/// Returns Err on CLI failure or when no source mode succeeds for any
/// requested opcode.
pub fn run(args: &[String]) -> Result<(), String> {
    let options = parse_args(args)?;
    std::fs::create_dir_all(&options.output_dir)
        .map_err(|err| format!("create output dir: {err}"))?;

    let mut produced: Vec<(String, Source)> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    for opcode in &options.opcodes {
        match capture_one(opcode, &options) {
            Ok(mode) => produced.push((opcode.clone(), mode)),
            Err(err) => failures.push(format!("{opcode}: {err}")),
        }
    }

    // Write a summary report at output_dir/README.md.
    let mut summary = String::from("# JSC LLInt reference asm\n\nCaptured by `lyng-bench capture-llint`.\n\n| Opcode | Source mode |\n|---|---|\n");
    for (opcode, mode) in &produced {
        writeln!(summary, "| `{opcode}` | {mode:?} |").expect("writing to a String cannot fail");
    }
    let summary_path = options.output_dir.join("README.md");
    std::fs::write(&summary_path, summary)
        .map_err(|err| format!("write {}: {err}", summary_path.display()))?;

    println!(
        "captured {} opcodes, {} failures",
        produced.len(),
        failures.len()
    );
    for failure in &failures {
        eprintln!("  {failure}");
    }
    if produced.is_empty() {
        Err("no opcodes captured".into())
    } else {
        Ok(())
    }
}

fn capture_one(opcode: &str, options: &CaptureLlintOptions) -> Result<Source, String> {
    let modes: Vec<Source> = match options.source {
        Source::Auto => vec![Source::System, Source::Local, Source::Excerpt],
        single => vec![single],
    };
    let mut errors = Vec::new();
    for mode in modes {
        match try_mode(mode, opcode, options) {
            Ok(asm) => {
                let path = options.output_dir.join(format!("{opcode}.md"));
                let body = format!(
                    "# JSC LLInt reference: `{opcode}`\n\nCapture mode: {mode:?}\n\n```asm\n{asm}\n```\n"
                );
                std::fs::write(&path, body)
                    .map_err(|err| format!("write {}: {err}", path.display()))?;
                return Ok(mode);
            }
            Err(err) => errors.push(format!("{mode:?}: {err}")),
        }
    }
    Err(errors.join(" | "))
}

fn try_mode(mode: Source, opcode: &str, options: &CaptureLlintOptions) -> Result<String, String> {
    match mode {
        Source::System | Source::Local => {
            let binary = match mode {
                Source::System => options.jsc_binary.clone()
                    .unwrap_or_else(|| PathBuf::from("/System/Library/Frameworks/JavaScriptCore.framework/Versions/Current/Helpers/jsc")),
                Source::Local => options.jsc_binary.clone()
                    .ok_or("--jsc-binary required for local mode")?,
                _ => unreachable!(),
            };
            capture_from_binary(&binary, opcode)
        }
        Source::Excerpt => {
            let source_root = options
                .jsc_source
                .clone()
                .ok_or("--jsc-source required for excerpt mode")?;
            capture_from_source(&source_root, opcode)
        }
        Source::Auto => unreachable!("Auto is expanded earlier"),
    }
}

fn capture_from_binary(binary: &std::path::Path, opcode: &str) -> Result<String, String> {
    let symbol = format!("_llint_{opcode}");
    let tool = if cfg!(target_os = "macos") {
        "otool"
    } else {
        "objdump"
    };
    let args: Vec<String> = if cfg!(target_os = "macos") {
        vec!["-tvV".into(), binary.display().to_string()]
    } else {
        vec![
            "-d".into(),
            "--no-show-raw-insn".into(),
            binary.display().to_string(),
        ]
    };
    let output = std::process::Command::new(tool)
        .args(args)
        .output()
        .map_err(|err| format!("run {tool}: {err}"))?;
    if !output.status.success() {
        return Err(format!("{tool} exited {}", output.status));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    extract_llint_symbol(&text, &symbol)
}

fn extract_llint_symbol(disasm: &str, symbol: &str) -> Result<String, String> {
    let iter = disasm.lines();
    let mut body: Vec<String> = Vec::new();
    let mut found = false;
    for line in iter {
        if found {
            if line.contains("_llint_op_") && !line.contains(symbol) {
                break;
            }
            body.push(line.to_string());
            if body.len() > 200 {
                break;
            }
        } else if line.contains(symbol) {
            found = true;
            body.push(line.to_string());
        }
    }
    if !found {
        return Err(format!(
            "symbol {symbol} not found (binary may be stripped)"
        ));
    }
    Ok(body.join("\n"))
}

fn capture_from_source(source_root: &std::path::Path, opcode: &str) -> Result<String, String> {
    let candidates = [
        source_root.join("Source/JavaScriptCore/llint/LowLevelInterpreter64.asm"),
        source_root.join("Source/JavaScriptCore/llint/LowLevelInterpreter.asm"),
    ];

    // Build search patterns. JSC wraps opcodes through several macro variants:
    //   - llintOp(name, ...)              basic
    //   - llintOpWithReturn(name, ...)    return-value form
    //   - llintOpWithMetadata(name, ...)  IC metadata form
    //   - llintOpWithJump(name, ...)      jump form
    //   - llintOpWithProfile(name, ...)   value-profile form
    // In addition, many opcodes are defined via per-family helper macros that
    // strip the `op_` prefix from the name, e.g.
    //   - binaryOp(add, OpAdd, ...)          → op_add
    //   - binaryOpCustomStore(mul, OpMul, ...)→ op_mul
    //   - bitOp(bitand, OpBitand, ...)       → op_bitand
    //   - preOp(inc, OpInc, ...)             → op_inc
    //   - compareJumpOp(jless, OpJless, ...) → op_jless
    //   - equalityJumpOp(jeq, OpJeq, ...)    → op_jeq
    //   - llintJumpTrueOrFalseOp(jtrue, ...) → op_jtrue
    //   - strictEqOp / strictEqualityJumpOp / compareOp / compareUnsignedJumpOp / equalityComparisonOp
    //   - commonCallOp(op_call, ...)         → op_call (full name kept)
    //   - commonOp(llint_op_catch, ...)      → op_catch (prefix `llint_` stripped)
    // We try each pattern in order; the first hit wins. For each match we
    // capture up to 80 lines (a generous handler-sized window).

    let direct_macros = [
        "llintOp(",
        "llintOpWithReturn(",
        "llintOpWithMetadata(",
        "llintOpWithJump(",
        "llintOpWithProfile(",
    ];

    // Strip leading "op_" for helper-macro shorthand names.
    let stripped: Option<&str> = opcode.strip_prefix("op_");

    // (macro_name, name_to_use). For direct macros the `op_` prefix is kept.
    // For helper macros the prefix is stripped (or kept for commonCallOp).
    let mut needles: Vec<String> = Vec::new();
    for macro_name in &direct_macros {
        needles.push(format!("{macro_name}{opcode},"));
    }
    if let Some(short) = stripped {
        for helper in [
            "binaryOp(",
            "binaryOpCustomStore(",
            "bitOp(",
            "preOp(",
            "compareOp(",
            "compareJumpOp(",
            "compareUnsignedJumpOp(",
            "equalityJumpOp(",
            "equalityComparisonOp(",
            "llintJumpTrueOrFalseOp(",
            "strictEqOp(",
            "strictEqualityJumpOp(",
            "putByValOp(",
        ] {
            needles.push(format!("{helper}{short},"));
            // Some helper macros are called with leading whitespace + name on
            // the next line; account for the trailing newline form by also
            // trying `(<short>,` with a newline. (Currently uniform: name is on
            // same line in JSC, but `compareJumpOp(\n    jless, ...)` happens
            // — we handle this by also searching for `\n    <short>,`.)
            needles.push(format!("\n    {short},"));
        }
        // commonCallOp keeps the `op_` prefix.
        needles.push(format!("commonCallOp({opcode},"));
        needles.push(format!("commonOp(llint_{opcode},"));
    }

    for file in &candidates {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        for needle in &needles {
            if let Some(start) = text.find(needle.as_str()) {
                // For the `\n    <short>,` form, back up to the start of the
                // preceding macro-callsite line so the captured excerpt
                // includes the macro name (e.g. `compareJumpOp(`).
                let start = if needle.starts_with('\n') {
                    text[..start].rfind('\n').map_or(start, |i| {
                        // Find the start of the line *before* the one we
                        // just located: that's the macro-name line.
                        text[..i].rfind('\n').map_or(0, |j| j + 1)
                    })
                } else {
                    start
                };
                let body: String = text[start..]
                    .lines()
                    .take(80)
                    .collect::<Vec<_>>()
                    .join("\n");
                return Ok(body);
            }
        }
    }
    Err("opcode not found in any offlineasm source file".into())
}

fn parse_args(args: &[String]) -> Result<CaptureLlintOptions, String> {
    let mut source = Source::Auto;
    let mut jsc_binary: Option<PathBuf> = None;
    let mut jsc_source: Option<PathBuf> = None;
    let mut opcodes: Vec<String> = Vec::new();
    let mut output_dir = PathBuf::from("reports/lyng/llint-reference");

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--source" => match iter.next().map(String::as_str) {
                Some("auto") => source = Source::Auto,
                Some("system") => source = Source::System,
                Some("local") => source = Source::Local,
                Some("excerpt") => source = Source::Excerpt,
                Some(other) => return Err(format!("--source: unknown {other}")),
                None => return Err("--source requires a value".into()),
            },
            "--jsc-binary" => {
                jsc_binary = Some(iter.next().ok_or("--jsc-binary requires a path")?.into());
            }
            "--jsc-source" => {
                jsc_source = Some(iter.next().ok_or("--jsc-source requires a path")?.into());
            }
            "--opcodes" => {
                let list = iter
                    .next()
                    .ok_or("--opcodes requires a comma-separated list")?;
                opcodes.extend(list.split(',').map(str::trim).map(String::from));
            }
            "--output" => output_dir = iter.next().ok_or("--output requires a path")?.into(),
            "--help" | "-h" => return Err(help_text()),
            other => {
                return Err(format!(
                    "capture-llint: unknown arg {other}\n\n{}",
                    help_text()
                ))
            }
        }
    }

    if opcodes.is_empty() {
        return Err("--opcodes <comma-separated list> is required".into());
    }
    Ok(CaptureLlintOptions {
        source,
        jsc_binary,
        jsc_source,
        opcodes,
        output_dir,
    })
}

fn help_text() -> String {
    [
        "Usage: lyng-bench capture-llint --opcodes <list> [options]",
        "",
        "Options:",
        "  --source auto|system|local|excerpt   Capture strategy (default auto)",
        "  --jsc-binary PATH                    JSC binary for system/local mode",
        "  --jsc-source PATH                    WebKit source root for excerpt mode",
        "  --opcodes a,b,c                      Comma-separated LLInt opcode names (without `_llint_` prefix)",
        "  --output PATH                        Output directory (default reports/lyng/llint-reference)",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_args() {
        let opts = parse_args(&["--opcodes".into(), "op_add,op_mov".into()]).unwrap();
        assert_eq!(opts.source, Source::Auto);
        assert_eq!(opts.opcodes, vec!["op_add", "op_mov"]);
    }

    #[test]
    fn rejects_missing_opcodes() {
        let err = parse_args(&[]).unwrap_err();
        assert!(err.contains("--opcodes"));
    }
}
