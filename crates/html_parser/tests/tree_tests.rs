use lyng_html_parser::dom::element::Namespace;
use lyng_html_parser::dom::serialize::serialize_tree;
use lyng_html_parser::tree::builder::{
    parse_fragment, parse_str, parse_str_scripting, FragmentContext,
};
use std::fs;
use std::path::Path;

struct TreeTest {
    data: String,
    expected: String,
    #[allow(dead_code)]
    fragment: Option<String>,
    script_on: bool,
}

fn parse_test_file(path: &Path) -> Vec<TreeTest> {
    let content = fs::read_to_string(path).unwrap_or_else(|_| {
        // Try reading as bytes and converting lossy
        let bytes = fs::read(path).unwrap();
        String::from_utf8_lossy(&bytes).into_owned()
    });

    let mut tests = Vec::new();
    let mut lines = content.lines().peekable();

    while lines.peek().is_some() {
        // Find #data
        loop {
            match lines.next() {
                Some("#data") => break,
                None => return tests,
                _ => continue,
            }
        }

        // Read input data until next #
        let mut data = String::new();
        loop {
            match lines.peek() {
                Some(line) if line.starts_with('#') => break,
                Some(line) => {
                    if !data.is_empty() {
                        data.push('\n');
                    }
                    data.push_str(line);
                    lines.next();
                }
                None => break,
            }
        }

        // Skip #errors and other sections until #document or #document-fragment
        let mut fragment = None;
        let mut script_on = false;
        loop {
            match lines.peek() {
                Some(&"#document") => {
                    lines.next();
                    break;
                }
                Some(&"#script-on") => {
                    script_on = true;
                    lines.next();
                }
                Some(&"#script-off") => {
                    script_on = false;
                    lines.next();
                }
                Some(line) if line.starts_with("#document-fragment") => {
                    let frag_line = lines.next().unwrap();
                    if let Some(ctx) = lines.next() {
                        fragment = Some(ctx.to_string());
                    }
                    let _ = frag_line;
                    continue;
                }
                Some(_) => {
                    lines.next();
                }
                None => break,
            }
        }

        // Read expected tree until empty line or EOF.
        // Multi-line text nodes: an opening `| ... "text` continues until a line
        // containing only `"` closes it. Blank lines between are literal newlines.
        let mut expected = String::new();
        let mut in_multiline_text = false;
        loop {
            match lines.peek() {
                Some(&"") => {
                    if in_multiline_text {
                        // Blank line inside multi-line text = literal newline in content
                        lines.next();
                        expected.push('\n');
                        continue;
                    }
                    // Otherwise, end of expected tree
                    lines.next();
                    break;
                }
                Some(&"\"") if in_multiline_text => {
                    // Closing quote of multi-line text node
                    if !expected.is_empty() {
                        expected.push('\n');
                    }
                    expected.push_str(lines.next().unwrap());
                    in_multiline_text = false;
                }
                Some(line) if line.starts_with("| ") => {
                    if !expected.is_empty() {
                        expected.push('\n');
                    }
                    // Check if this line opens a multi-line text node
                    // (contains `"` but doesn't close with `"`)
                    let content = &line[2..]; // skip "| "
                    let trimmed = content.trim_start();
                    if trimmed == "\"" || (trimmed.starts_with('"') && !trimmed.ends_with('"')) {
                        // Opening quote without a closing quote on the same line
                        in_multiline_text = true;
                    } else if trimmed.ends_with('"') && trimmed.starts_with('"') {
                        // Quoted text entirely on one line
                        in_multiline_text = false;
                    }
                    expected.push_str(lines.next().unwrap());
                }
                Some(line) if line.starts_with("#") => break,
                Some(_line) => {
                    if in_multiline_text {
                        // Continuation line inside multi-line text node
                        if !expected.is_empty() {
                            expected.push('\n');
                        }
                        expected.push_str(lines.next().unwrap());
                    } else {
                        // Continuation of tree (e.g., attribute values)
                        if !expected.is_empty() {
                            expected.push('\n');
                        }
                        expected.push_str(lines.next().unwrap());
                    }
                }
                None => break,
            }
        }

        if !expected.is_empty() {
            tests.push(TreeTest {
                data,
                expected,
                fragment,
                script_on,
            });
        }
    }

    tests
}

fn run_test_file(path: &Path) -> (usize, usize) {
    let tests = parse_test_file(path);
    let mut passed = 0;
    let mut total = 0;

    for test in &tests {
        total += 1;

        let result = if let Some(fragment_ctx) = &test.fragment {
            let context = parse_fragment_context(fragment_ctx);
            parse_fragment(&test.data, context, test.script_on)
        } else if test.script_on {
            parse_str_scripting(&test.data, true)
        } else {
            parse_str(&test.data)
        };
        let actual = if let Some(fragment_root) = result.fragment_root {
            serialize_tree(&result.arena, fragment_root)
        } else if let Some(context_node) = result.fragment_context {
            serialize_tree(&result.arena, context_node)
        } else {
            serialize_tree(&result.arena, result.document)
        };
        let actual_trimmed = actual.trim_end();
        let expected_trimmed = test.expected.trim_end();

        if actual_trimmed == expected_trimmed {
            passed += 1;
        } else {
            eprintln!(
                "FAIL [{}]\n  input: {:?}\n  expected:\n{}\n  actual:\n{}\n---",
                path.display(),
                test.data,
                expected_trimmed,
                actual_trimmed
            );
        }
    }

    (passed, total)
}

fn collect_tree_test_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let mut dirs = vec![dir.to_path_buf()];

    while let Some(current) = dirs.pop() {
        let mut entries: Vec<_> = fs::read_dir(&current)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.extension().is_some_and(|ext| ext == "dat") {
                files.push(path);
            }
        }
    }

    files.sort();
    files
}

fn parse_fragment_context(raw: &str) -> FragmentContext {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix("svg ") {
        FragmentContext {
            namespace: Namespace::Svg,
            tag_name: rest.to_string(),
        }
    } else if let Some(rest) = trimmed.strip_prefix("math ") {
        FragmentContext {
            namespace: Namespace::MathML,
            tag_name: rest.to_string(),
        }
    } else {
        FragmentContext {
            namespace: Namespace::Html,
            tag_name: trimmed.to_string(),
        }
    }
}

#[test]
fn html5lib_tree_construction_tests() {
    let test_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/html5lib/tree-construction");
    if !test_dir.exists() {
        eprintln!("html5lib-tests not found, skipping");
        return;
    }

    let mut total_passed = 0;
    let mut total_tests = 0;

    for path in collect_tree_test_files(&test_dir) {
        let filename = path.strip_prefix(&test_dir).unwrap().display().to_string();
        let (passed, total) = run_test_file(&path);
        eprintln!("  {filename}: {passed}/{total}");
        total_passed += passed;
        total_tests += total;
    }

    let pct = if total_tests > 0 {
        (total_passed as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };
    eprintln!("\nTree construction test results: {total_passed}/{total_tests} ({pct:.1}%)");

    assert_eq!(
        total_passed, total_tests,
        "Not all tree-construction tests passed"
    );
}
