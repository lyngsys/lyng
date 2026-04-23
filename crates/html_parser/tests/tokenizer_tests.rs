use lyng_html_parser::input::InputStream;
use lyng_html_parser::tokenizer::states::State;
use lyng_html_parser::tokenizer::tokens::Token;
use lyng_html_parser::tokenizer::Tokenizer;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
#[serde(untagged)]
enum TestFile {
    Standard {
        tests: Vec<TestCase>,
    },
    XmlViolation {
        #[serde(rename = "xmlViolationTests")]
        tests: Vec<TestCase>,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestCase {
    #[allow(dead_code)]
    description: String,
    input: String,
    output: Vec<serde_json::Value>,
    #[serde(default)]
    #[allow(dead_code)]
    errors: Vec<TestError>,
    #[serde(default)]
    initial_states: Vec<String>,
    #[serde(default)]
    last_start_tag: Option<String>,
    #[serde(default)]
    double_escaped: Option<bool>,
}

#[derive(Deserialize)]
struct TestError {
    #[allow(dead_code)]
    code: String,
    #[allow(dead_code)]
    line: u32,
    #[allow(dead_code)]
    col: u32,
}

fn state_from_string(s: &str) -> State {
    match s {
        "Data state" => State::Data,
        "PLAINTEXT state" => State::PlainText,
        "RCDATA state" => State::RcData,
        "RAWTEXT state" => State::RawText,
        "Script data state" => State::ScriptData,
        "CDATA section state" => State::CdataSection,
        other => panic!("Unknown initial state: {other}"),
    }
}

/// Unescape double-escaped test input (html5lib uses this for some tests).
fn unescape_double(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == 'u' {
            // Parse \uXXXX
            if i + 5 < chars.len() {
                let hex: String = chars[i + 2..i + 6].iter().collect();
                if let Ok(cp) = u32::from_str_radix(&hex, 16) {
                    if let Some(c) = char::from_u32(cp) {
                        result.push(c);
                        i += 6;
                        continue;
                    }
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// Recursively unescape double-escaped JSON values.
fn unescape_json_value(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::String(s) => serde_json::Value::String(unescape_double(s)),
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(unescape_json_value).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut m = serde_json::Map::new();
            for (k, val) in obj {
                m.insert(unescape_double(k), unescape_json_value(val));
            }
            serde_json::Value::Object(m)
        }
        other => other.clone(),
    }
}

/// Convert tokenizer output to a comparable format matching html5lib JSON.
fn tokens_to_json(tokens: &[Token]) -> Vec<serde_json::Value> {
    let mut result = Vec::new();
    let mut pending_chars = String::new();

    for token in tokens {
        match token {
            Token::Character { data } => {
                pending_chars.push(*data);
            }
            _ => {
                if !pending_chars.is_empty() {
                    result.push(serde_json::json!(["Character", pending_chars]));
                    pending_chars.clear();
                }
                match token {
                    Token::StartTag {
                        name,
                        attributes,
                        self_closing,
                    } => {
                        let mut attrs = serde_json::Map::new();
                        for attr in attributes {
                            attrs.insert(
                                attr.name.clone(),
                                serde_json::Value::String(attr.value.clone()),
                            );
                        }
                        if *self_closing {
                            result.push(serde_json::json!(["StartTag", name, attrs, true]));
                        } else {
                            result.push(serde_json::json!(["StartTag", name, attrs]));
                        }
                    }
                    Token::EndTag { name } => {
                        result.push(serde_json::json!(["EndTag", name]));
                    }
                    Token::Comment { data } => {
                        result.push(serde_json::json!(["Comment", data]));
                    }
                    Token::Doctype {
                        name,
                        public_id,
                        system_id,
                        force_quirks,
                    } => {
                        result.push(serde_json::json!([
                            "DOCTYPE",
                            name,
                            public_id,
                            system_id,
                            !force_quirks
                        ]));
                    }
                    Token::EndOfFile => {}
                    Token::Character { .. } => unreachable!(),
                }
            }
        }
    }

    if !pending_chars.is_empty() {
        result.push(serde_json::json!(["Character", pending_chars]));
    }

    result
}

fn coerce_infoset_string(s: &str, in_comment: bool) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        let normalized = if ch == '\u{000C}' {
            ' '
        } else if !is_valid_xml_char(ch) {
            '\u{FFFD}'
        } else {
            ch
        };

        if in_comment && normalized == '-' && chars.peek() == Some(&'-') {
            result.push('-');
            result.push(' ');
            continue;
        }

        result.push(normalized);
    }

    result
}

fn is_valid_xml_char(ch: char) -> bool {
    matches!(
        ch as u32,
        0x9 | 0xA | 0xD | 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..=0x10FFFF
    )
}

fn coerce_infoset_output(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Array(items) if !items.is_empty() => {
            let kind = items[0].as_str().unwrap_or_default();
            match kind {
                "Character" => serde_json::json!([
                    "Character",
                    coerce_infoset_string(items[1].as_str().unwrap_or_default(), false)
                ]),
                "Comment" => serde_json::json!([
                    "Comment",
                    coerce_infoset_string(items[1].as_str().unwrap_or_default(), true)
                ]),
                "StartTag" => {
                    let mut coerced = items.clone();
                    if let Some(attrs) = items.get(2).and_then(|value| value.as_object()) {
                        let mut next_attrs = serde_json::Map::new();
                        for (key, value) in attrs {
                            let coerced_value =
                                coerce_infoset_string(value.as_str().unwrap_or_default(), false);
                            next_attrs
                                .insert(key.clone(), serde_json::Value::String(coerced_value));
                        }
                        coerced[2] = serde_json::Value::Object(next_attrs);
                    }
                    serde_json::Value::Array(coerced)
                }
                _ => v.clone(),
            }
        }
        _ => v.clone(),
    }
}

fn run_test_file(path: &Path) -> (usize, usize) {
    let content = fs::read_to_string(path).unwrap();
    let test_file: TestFile = serde_json::from_str(&content).unwrap();
    let (tests, xml_violation) = match test_file {
        TestFile::Standard { tests } => (tests, false),
        TestFile::XmlViolation { tests } => (tests, true),
    };

    let mut passed = 0;
    let mut total = 0;

    for test in &tests {
        let states = if test.initial_states.is_empty() {
            vec![State::Data]
        } else {
            test.initial_states
                .iter()
                .map(|s| state_from_string(s))
                .collect()
        };

        let is_double_escaped = test.double_escaped.unwrap_or(false);

        for initial_state in &states {
            total += 1;

            let input = if is_double_escaped {
                unescape_double(&test.input)
            } else {
                test.input.clone()
            };

            let stream = InputStream::new(&input);
            let mut tokenizer = Tokenizer::new(stream);
            tokenizer.set_state(*initial_state);

            if let Some(ref last) = test.last_start_tag {
                tokenizer.set_last_start_tag(last);
            }

            let mut tokens = Vec::new();
            loop {
                let token = tokenizer.next_token();
                let is_eof = token == Token::EndOfFile;
                tokens.push(token);
                if is_eof {
                    break;
                }
            }

            let actual = if xml_violation {
                tokens_to_json(&tokens)
                    .iter()
                    .map(coerce_infoset_output)
                    .collect()
            } else {
                tokens_to_json(&tokens)
            };

            // Unescape expected output if double_escaped
            let expected: Vec<serde_json::Value> = if is_double_escaped {
                test.output.iter().map(unescape_json_value).collect()
            } else {
                test.output.clone()
            };

            if actual == expected {
                passed += 1;
            }
            // else {
            //     let state_name = format!("{:?}", initial_state);
            //     eprintln!(
            //         "FAIL [{}]: {} (state: {})\n  input: {:?}\n  expected: {}\n  actual:   {}",
            //         path.display(),
            //         test.description,
            //         state_name,
            //         input,
            //         serde_json::to_string(&expected).unwrap(),
            //         serde_json::to_string(&actual).unwrap(),
            //     );
            // }
        }
    }

    (passed, total)
}

#[test]
fn html5lib_tokenizer_tests() {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/html5lib/tokenizer");
    if !test_dir.exists() {
        eprintln!("html5lib-tests not found, skipping");
        return;
    }

    let mut total_passed = 0;
    let mut total_tests = 0;

    let mut entries: Vec<_> = fs::read_dir(&test_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "test"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_str().unwrap();
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
    eprintln!("\nTokenizer test results: {total_passed}/{total_tests} ({pct:.1}%)");

    assert_eq!(total_passed, total_tests, "Not all tokenizer tests passed");
}
