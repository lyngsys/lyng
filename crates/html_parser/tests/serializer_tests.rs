use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
enum SerializerToken {
    StartTag {
        name: String,
        attrs: Vec<Attribute>,
    },
    EndTag {
        name: String,
    },
    EmptyTag {
        name: String,
        attrs: Vec<Attribute>,
    },
    Comment(String),
    Characters(String),
    Doctype {
        name: String,
        public_id: Option<String>,
        system_id: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Attribute {
    name: String,
    value: String,
}

#[derive(Debug, Default, Deserialize)]
struct SerializerOptions {
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    inject_meta_charset: bool,
    #[serde(default)]
    strip_whitespace: bool,
    #[serde(default)]
    use_trailing_solidus: bool,
    #[serde(default)]
    minimize_boolean_attributes: Option<bool>,
    #[serde(default)]
    escape_lt_in_attrs: bool,
    #[serde(default)]
    escape_rcdata: bool,
    #[serde(default)]
    #[allow(dead_code)]
    quote_attr_values: Option<bool>,
    #[serde(default)]
    quote_char: Option<String>,
}

impl SerializerOptions {
    fn minimize_boolean_attributes(&self) -> bool {
        self.minimize_boolean_attributes.unwrap_or(true)
    }

    fn preferred_quote(&self) -> Option<char> {
        self.quote_char.as_deref().and_then(|s| s.chars().next())
    }
}

#[derive(Debug, Deserialize)]
struct SerializerTestFile {
    tests: Vec<SerializerTestCase>,
}

#[derive(Debug, Deserialize)]
struct SerializerTestCase {
    #[allow(dead_code)]
    description: String,
    #[serde(default)]
    options: SerializerOptions,
    input: Vec<Value>,
    expected: Vec<String>,
    #[serde(default)]
    xhtml: Vec<String>,
}

fn parse_serializer_token(value: &Value) -> SerializerToken {
    let arr = value
        .as_array()
        .expect("serializer token should be an array");
    let kind = arr[0].as_str().expect("token kind should be a string");

    match kind {
        "StartTag" => SerializerToken::StartTag {
            name: arr[2].as_str().unwrap().to_string(),
            attrs: parse_attributes(&arr[3]),
        },
        "EndTag" => SerializerToken::EndTag {
            name: arr[2].as_str().unwrap().to_string(),
        },
        "EmptyTag" => SerializerToken::EmptyTag {
            name: arr[1].as_str().unwrap().to_string(),
            attrs: parse_attributes(&arr[2]),
        },
        "Comment" => SerializerToken::Comment(arr[1].as_str().unwrap().to_string()),
        "Characters" => SerializerToken::Characters(arr[1].as_str().unwrap().to_string()),
        "Doctype" => SerializerToken::Doctype {
            name: arr[1].as_str().unwrap().to_string(),
            public_id: arr.get(2).and_then(|v| v.as_str()).map(str::to_string),
            system_id: arr.get(3).and_then(|v| v.as_str()).map(str::to_string),
        },
        other => panic!("unsupported serializer token kind: {other}"),
    }
}

fn parse_attributes(value: &Value) -> Vec<Attribute> {
    match value {
        Value::Object(map) => map
            .iter()
            .map(|(name, value)| Attribute {
                name: name.clone(),
                value: value.as_str().unwrap_or_default().to_string(),
            })
            .collect(),
        Value::Array(items) => items
            .iter()
            .map(|item| Attribute {
                name: item["name"].as_str().unwrap().to_string(),
                value: item["value"].as_str().unwrap_or_default().to_string(),
            })
            .collect(),
        other => panic!("unsupported attribute payload: {other:?}"),
    }
}

fn serialize_tokens(
    tokens: &[SerializerToken],
    options: &SerializerOptions,
    xhtml: bool,
) -> String {
    let tokens = rewrite_head_charset(tokens, options, xhtml);
    let mut output = String::new();
    let mut context_stack: Vec<String> = Vec::new();
    let mut last_omitted_end_tag: Option<String> = None;

    for (index, token) in tokens.iter().enumerate() {
        let next = tokens.get(index + 1);
        let current_context = context_stack.last().map(String::as_str);

        match token {
            SerializerToken::StartTag { name, attrs } => {
                if !xhtml
                    && should_omit_start_tag(name, attrs, next, last_omitted_end_tag.as_deref())
                {
                    context_stack.push(name.clone());
                    last_omitted_end_tag = None;
                    continue;
                }

                output.push('<');
                output.push_str(name);
                serialize_attributes(&mut output, attrs, options, xhtml);
                if is_void_html_element(name) {
                    if xhtml || options.use_trailing_solidus {
                        output.push_str(" /");
                    }
                    output.push('>');
                } else {
                    output.push('>');
                    context_stack.push(name.clone());
                }
                last_omitted_end_tag = None;
            }
            SerializerToken::EndTag { name } => {
                if !xhtml && should_omit_end_tag(name, next) {
                    pop_matching_context(&mut context_stack, name);
                    last_omitted_end_tag = Some(name.clone());
                    continue;
                }

                output.push_str("</");
                output.push_str(name);
                output.push('>');
                pop_matching_context(&mut context_stack, name);
                last_omitted_end_tag = None;
            }
            SerializerToken::EmptyTag { name, attrs } => {
                output.push('<');
                output.push_str(name);
                serialize_attributes(&mut output, attrs, options, xhtml);
                if xhtml || options.use_trailing_solidus {
                    output.push_str(" /");
                }
                output.push('>');
                last_omitted_end_tag = None;
            }
            SerializerToken::Comment(data) => {
                output.push_str("<!--");
                output.push_str(data);
                output.push_str("-->");
                last_omitted_end_tag = None;
            }
            SerializerToken::Characters(data) => {
                let data = if options.strip_whitespace && !preserves_whitespace(&context_stack) {
                    collapse_whitespace(data)
                } else {
                    data.clone()
                };

                if is_rawtext_context(current_context) && !(xhtml || options.escape_rcdata) {
                    output.push_str(&data);
                } else {
                    output.push_str(&escape_text(&data));
                }
                last_omitted_end_tag = None;
            }
            SerializerToken::Doctype {
                name,
                public_id,
                system_id,
            } => {
                output.push_str("<!DOCTYPE ");
                output.push_str(name);
                match (public_id.as_deref(), system_id.as_deref()) {
                    (Some(public_id), Some(system_id)) => {
                        if public_id.is_empty() {
                            output.push_str(" SYSTEM \"");
                            output.push_str(system_id);
                            output.push('"');
                        } else {
                            output.push_str(" PUBLIC \"");
                            output.push_str(public_id);
                            output.push_str("\" \"");
                            output.push_str(system_id);
                            output.push('"');
                        }
                    }
                    (Some(public_id), None) => {
                        output.push_str(" PUBLIC \"");
                        output.push_str(public_id);
                        output.push('"');
                    }
                    (None, Some(system_id)) => {
                        output.push_str(" SYSTEM \"");
                        output.push_str(system_id);
                        output.push('"');
                    }
                    (None, None) => {}
                }
                output.push('>');
                last_omitted_end_tag = None;
            }
        }
    }

    output
}

fn rewrite_head_charset(
    tokens: &[SerializerToken],
    options: &SerializerOptions,
    _xhtml: bool,
) -> Vec<SerializerToken> {
    if !options.inject_meta_charset {
        return tokens.to_vec();
    }

    let Some(encoding) = options.encoding.as_deref() else {
        return tokens.to_vec();
    };

    let mut result = Vec::with_capacity(tokens.len());
    let mut index = 0;

    while index < tokens.len() {
        if let SerializerToken::StartTag { name, .. } = &tokens[index] {
            if name == "head" {
                let mut depth = 1usize;
                let mut end = index + 1;
                while end < tokens.len() && depth > 0 {
                    match &tokens[end] {
                        SerializerToken::StartTag { name, .. } if name == "head" => depth += 1,
                        SerializerToken::EndTag { name } if name == "head" => depth -= 1,
                        _ => {}
                    }
                    end += 1;
                }

                let mut body = tokens[index + 1..end.saturating_sub(1)].to_vec();
                let mut found_charset = false;
                for token in &mut body {
                    match token {
                        SerializerToken::StartTag { name, attrs }
                        | SerializerToken::EmptyTag { name, attrs }
                            if name == "meta" =>
                        {
                            if rewrite_meta_tag(attrs, encoding) {
                                found_charset = true;
                            }
                        }
                        _ => {}
                    }
                }

                if !found_charset {
                    body.insert(
                        0,
                        SerializerToken::EmptyTag {
                            name: "meta".to_string(),
                            attrs: vec![Attribute {
                                name: "charset".to_string(),
                                value: encoding.to_string(),
                            }],
                        },
                    );
                }

                result.push(tokens[index].clone());
                result.extend(body);
                if end > index + 1 {
                    result.push(tokens[end - 1].clone());
                }
                index = end;
                continue;
            }
        }

        result.push(tokens[index].clone());
        index += 1;
    }

    result
}

fn rewrite_meta_tag(attrs: &mut [Attribute], encoding: &str) -> bool {
    for attr in attrs.iter_mut() {
        if attr.name == "charset" {
            attr.value = encoding.to_string();
            return true;
        }
    }

    let http_equiv = attrs
        .iter()
        .any(|attr| attr.name == "http-equiv" && attr.value.eq_ignore_ascii_case("content-type"));
    if !http_equiv {
        return false;
    }

    if let Some(content) = attrs.iter_mut().find(|attr| attr.name == "content") {
        content.value = format!("text/html; charset={encoding}");
        return true;
    }

    false
}

fn serialize_attributes(
    output: &mut String,
    attrs: &[Attribute],
    options: &SerializerOptions,
    xhtml: bool,
) {
    let mut attrs = attrs.to_vec();
    attrs.sort_by(|a, b| a.name.cmp(&b.name));

    for attr in attrs {
        output.push(' ');
        output.push_str(&attr.name);

        if !xhtml
            && options.minimize_boolean_attributes()
            && !attr.value.is_empty()
            && attr.value.eq_ignore_ascii_case(&attr.name)
        {
            continue;
        }

        output.push('=');

        let needs_quotes = xhtml
            || attr.value.is_empty()
            || attr.value.chars().any(|c| {
                matches!(
                    c,
                    '\t' | '\n' | '\r' | '\u{000C}' | ' ' | '"' | '\'' | '=' | '>'
                )
            });

        let quote = if xhtml {
            '"'
        } else if let Some(preferred) = options.preferred_quote() {
            preferred
        } else if !needs_quotes {
            '\0'
        } else {
            choose_quote(&attr.value)
        };

        let escaped = escape_attribute_value(
            &attr.value,
            if quote == '\0' { None } else { Some(quote) },
            xhtml || options.escape_lt_in_attrs,
        );

        if quote == '\0' {
            output.push_str(&escaped);
        } else {
            output.push(quote);
            output.push_str(&escaped);
            output.push(quote);
        }
    }
}

fn choose_quote(value: &str) -> char {
    let double_quotes = value.chars().filter(|&c| c == '"').count();
    let single_quotes = value.chars().filter(|&c| c == '\'').count();
    if double_quotes < single_quotes {
        '"'
    } else if single_quotes < double_quotes {
        '\''
    } else {
        '"'
    }
}

fn escape_attribute_value(value: &str, quote: Option<char>, escape_lt: bool) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' if escape_lt => out.push_str("&lt;"),
            '"' if quote == Some('"') => out.push_str("&quot;"),
            '\'' if quote == Some('\'') => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn escape_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

fn collapse_whitespace(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut in_whitespace = false;

    for ch in value.chars() {
        if is_html_space(ch) {
            if !in_whitespace {
                out.push(' ');
                in_whitespace = true;
            }
        } else {
            out.push(ch);
            in_whitespace = false;
        }
    }

    out
}

fn is_html_space(ch: char) -> bool {
    matches!(ch, '\t' | '\n' | '\r' | '\u{000C}' | ' ')
}

fn preserves_whitespace(stack: &[String]) -> bool {
    stack
        .iter()
        .rev()
        .any(|name| matches!(name.as_str(), "pre" | "textarea" | "script" | "style"))
}

fn is_rawtext_context(context: Option<&str>) -> bool {
    matches!(context, Some("script" | "style"))
}

fn pop_matching_context(stack: &mut Vec<String>, name: &str) {
    if let Some(pos) = stack.iter().rposition(|item| item == name) {
        stack.truncate(pos);
    }
}

fn should_omit_start_tag(
    name: &str,
    attrs: &[Attribute],
    next: Option<&SerializerToken>,
    last_omitted_end_tag: Option<&str>,
) -> bool {
    if !attrs.is_empty() {
        return false;
    }

    match name {
        "html" => !is_comment(next) && !starts_with_space(next),
        "head" => is_start_or_empty(next) || is_named_end_tag(next, "head") || next.is_none(),
        "body" => !is_comment(next) && !starts_with_space(next),
        "colgroup" => {
            last_omitted_end_tag != Some("colgroup")
                && (matches!(
                    next,
                    Some(SerializerToken::StartTag { name, .. }) if name == "col"
                ) || matches!(next, Some(SerializerToken::EmptyTag { name, .. }) if name == "col")
                    || matches!(next, Some(SerializerToken::EndTag { name }) if name == "colgroup"))
        }
        "tbody" => {
            !matches!(last_omitted_end_tag, Some("tbody" | "thead" | "tfoot"))
                && (matches!(
                    next,
                    Some(SerializerToken::StartTag { name, .. }) if name == "tr"
                ) || matches!(next, Some(SerializerToken::EmptyTag { name, .. }) if name == "tr")
                    || matches!(next, Some(SerializerToken::EndTag { name }) if name == "tbody"))
        }
        _ => false,
    }
}

fn should_omit_end_tag(name: &str, next: Option<&SerializerToken>) -> bool {
    match name {
        "html" | "body" => !is_comment(next) && !starts_with_space(next),
        "head" => !is_comment(next) && !starts_with_space(next),
        "li" => {
            next.is_none()
                || matches!(next, Some(SerializerToken::StartTag { name, .. }) if name == "li")
                || matches!(next, Some(SerializerToken::EndTag { .. }))
        }
        "dt" => {
            matches!(next, Some(SerializerToken::StartTag { name, .. }) if name == "dt" || name == "dd")
        }
        "dd" => {
            next.is_none()
                || matches!(next, Some(SerializerToken::StartTag { name, .. }) if name == "dt" || name == "dd")
                || matches!(next, Some(SerializerToken::EndTag { .. }))
        }
        "p" => {
            next.is_none()
                || matches!(next, Some(SerializerToken::StartTag { name, .. }) if is_p_breaker(name))
                || matches!(next, Some(SerializerToken::EmptyTag { name, .. }) if is_p_breaker(name))
                || matches!(next, Some(SerializerToken::EndTag { .. }))
        }
        "rt" | "rp" => {
            next.is_none()
                || matches!(next, Some(SerializerToken::StartTag { name, .. }) if name == "rt" || name == "rp")
                || matches!(next, Some(SerializerToken::EndTag { .. }))
        }
        "option" => {
            next.is_none()
                || matches!(next, Some(SerializerToken::StartTag { name, .. }) if name == "option" || name == "optgroup")
                || matches!(next, Some(SerializerToken::EndTag { .. }))
        }
        "optgroup" => {
            next.is_none()
                || matches!(next, Some(SerializerToken::StartTag { name, .. }) if name == "optgroup")
                || matches!(next, Some(SerializerToken::EndTag { .. }))
        }
        "colgroup" => next.is_none() || (!is_comment(next) && !starts_with_space(next)),
        "tbody" => {
            next.is_none()
                || matches!(next, Some(SerializerToken::StartTag { name, .. }) if name == "tbody" || name == "tfoot")
                || matches!(next, Some(SerializerToken::EndTag { .. }))
        }
        "thead" => matches!(
            next,
            Some(SerializerToken::StartTag { name, .. }) if name == "tbody" || name == "tfoot"
        ),
        "tfoot" => {
            next.is_none()
                || matches!(next, Some(SerializerToken::StartTag { name, .. }) if name == "tbody")
                || matches!(next, Some(SerializerToken::EndTag { .. }))
        }
        "tr" => {
            next.is_none()
                || matches!(next, Some(SerializerToken::StartTag { name, .. }) if name == "tr")
                || matches!(next, Some(SerializerToken::EndTag { .. }))
        }
        "td" | "th" => {
            next.is_none()
                || matches!(next, Some(SerializerToken::StartTag { name, .. }) if name == "td" || name == "th")
                || matches!(next, Some(SerializerToken::EndTag { .. }))
        }
        _ => false,
    }
}

fn is_void_html_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn is_comment(token: Option<&SerializerToken>) -> bool {
    matches!(token, Some(SerializerToken::Comment(_)))
}

fn starts_with_space(token: Option<&SerializerToken>) -> bool {
    matches!(
        token,
        Some(SerializerToken::Characters(data)) if data.starts_with(is_html_space)
    )
}

fn is_start_or_empty(token: Option<&SerializerToken>) -> bool {
    matches!(
        token,
        Some(SerializerToken::StartTag { .. }) | Some(SerializerToken::EmptyTag { .. })
    )
}

fn is_named_end_tag(token: Option<&SerializerToken>, name: &str) -> bool {
    matches!(token, Some(SerializerToken::EndTag { name: next_name }) if next_name == name)
}

fn is_p_breaker(name: &str) -> bool {
    matches!(
        name,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "datagrid"
            | "dialog"
            | "dir"
            | "div"
            | "dl"
            | "fieldset"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hr"
            | "menu"
            | "nav"
            | "ol"
            | "p"
            | "pre"
            | "section"
            | "table"
            | "ul"
    )
}

fn run_serializer_file(path: &Path) -> (usize, usize) {
    let content = fs::read_to_string(path).unwrap();
    let file: SerializerTestFile = serde_json::from_str(&content).unwrap();
    let mut passed = 0;
    let mut total = 0;

    for test in file.tests {
        let tokens: Vec<_> = test.input.iter().map(parse_serializer_token).collect();

        total += 1;
        let actual = serialize_tokens(&tokens, &test.options, false);
        if test.expected.iter().any(|expected| expected == &actual) {
            passed += 1;
        } else {
            eprintln!(
                "FAIL [{}] {}\n  expected: {:?}\n  actual:   {:?}",
                path.display(),
                test.description,
                test.expected,
                actual
            );
        }

        if !test.xhtml.is_empty() {
            total += 1;
            let actual_xhtml = serialize_tokens(&tokens, &test.options, true);
            if test.xhtml.iter().any(|expected| expected == &actual_xhtml) {
                passed += 1;
            } else {
                eprintln!(
                    "FAIL XHTML [{}] {}\n  expected: {:?}\n  actual:   {:?}",
                    path.display(),
                    test.description,
                    test.xhtml,
                    actual_xhtml
                );
            }
        }
    }

    (passed, total)
}

#[test]
fn html5lib_serializer_tests() {
    let serializer_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/html5lib/serializer");
    if !serializer_dir.exists() {
        eprintln!("html5lib serializer tests not found, skipping");
        return;
    }

    let mut entries: Vec<_> = fs::read_dir(&serializer_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "test"))
        .collect();
    entries.sort_by_key(|entry| entry.file_name());

    let mut total_passed = 0;
    let mut total_tests = 0;

    for entry in entries {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy().into_owned();
        let (passed, total) = run_serializer_file(&path);
        eprintln!("  {filename}: {passed}/{total}");
        total_passed += passed;
        total_tests += total;
    }

    assert_eq!(total_passed, total_tests, "Not all serializer tests passed");
}
