use encoding_rs::{Encoding, UTF_16BE, UTF_16LE, UTF_8, WINDOWS_1252};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct EncodingCase {
    data: Vec<u8>,
    expected: String,
}

fn parse_encoding_file(path: &Path) -> Vec<EncodingCase> {
    let bytes = fs::read(path).unwrap();
    let mut cases = Vec::new();
    let mut index = 0;

    while let Some(data_start) = find_bytes(&bytes[index..], b"#data\n") {
        let data_start = index + data_start + b"#data\n".len();
        let encoding_marker = find_bytes(&bytes[data_start..], b"\n#encoding\n")
            .map(|pos| data_start + pos)
            .expect("encoding marker should exist");

        let expected_start = encoding_marker + b"\n#encoding\n".len();
        let expected_end = bytes[expected_start..]
            .iter()
            .position(|&byte| byte == b'\n')
            .map(|pos| expected_start + pos)
            .unwrap_or(bytes.len());

        let data = bytes[data_start..encoding_marker].to_vec();
        let expected = String::from_utf8_lossy(&bytes[expected_start..expected_end])
            .trim()
            .to_ascii_lowercase();

        cases.push(EncodingCase { data, expected });
        index = expected_end;
    }

    cases
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn detect_encoding(data: &[u8]) -> &'static Encoding {
    sniff_encoding(data).unwrap_or(WINDOWS_1252)
}

fn sniff_encoding(data: &[u8]) -> Option<&'static Encoding> {
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Some(UTF_8);
    }

    let mut index = 0;
    while index < data.len() {
        if data[index] != b'<' {
            index += 1;
            continue;
        }

        let Some((tag, next_index)) = parse_tag(data, index) else {
            index += 1;
            continue;
        };

        match tag {
            ParsedTag::Meta(attrs) => {
                if let Some(encoding) = detect_meta_encoding(&attrs) {
                    return Some(encoding);
                }
            }
            ParsedTag::Script => {
                let script_end = find_ascii_case(&data[next_index..], b"</script>")?;
                let script_body = &data[next_index..next_index + script_end];
                if let Some(encoding) = detect_scripted_meta_encoding(script_body) {
                    return Some(encoding);
                }
            }
            ParsedTag::Other => {}
        }

        index = next_index;
    }

    None
}

enum ParsedTag {
    Other,
    Meta(Vec<(Vec<u8>, Vec<u8>)>),
    Script,
}

fn parse_tag(data: &[u8], start: usize) -> Option<(ParsedTag, usize)> {
    if data.get(start).is_none_or(|byte| *byte != b'<') {
        return None;
    }

    if data[start..].starts_with(b"<!--") {
        let end = find_bytes(&data[start + 4..], b"-->")
            .map(|pos| start + 4 + pos + 3)
            .unwrap_or(data.len());
        return Some((ParsedTag::Other, end));
    }

    let mut index = start + 1;
    if index >= data.len() {
        return None;
    }

    if matches!(data[index], b'!' | b'/' | b'?') {
        let end = scan_tag_end(data, index + 1, None);
        return Some((ParsedTag::Other, end));
    }

    if !data[index].is_ascii_alphabetic() {
        return None;
    }

    let name_start = index;
    while index < data.len() && is_tag_name_byte(data[index]) {
        index += 1;
    }
    let name = &data[name_start..index];
    let is_meta = eq_ascii_case(name, b"meta");
    let is_script = eq_ascii_case(name, b"script");

    let mut attrs = Vec::new();
    let mut found_tag_end = false;
    while index < data.len() {
        let byte = data[index];
        match byte {
            b'>' => {
                index += 1;
                found_tag_end = true;
                break;
            }
            b'\t' | b'\n' | b'\r' | b'\x0C' | b' ' | b'/' => {
                index += 1;
            }
            _ => {
                let attr_start = index;
                while index < data.len()
                    && !matches!(
                        data[index],
                        b'\t' | b'\n' | b'\r' | b'\x0C' | b' ' | b'/' | b'=' | b'>'
                    )
                {
                    index += 1;
                }
                let mut attr_name = data[attr_start..index].to_vec();
                ascii_lowercase_in_place(&mut attr_name);

                while index < data.len()
                    && matches!(data[index], b'\t' | b'\n' | b'\r' | b'\x0C' | b' ')
                {
                    index += 1;
                }

                let mut attr_value = Vec::new();
                if index < data.len() && data[index] == b'=' {
                    index += 1;
                    while index < data.len()
                        && matches!(data[index], b'\t' | b'\n' | b'\r' | b'\x0C' | b' ')
                    {
                        index += 1;
                    }

                    if index >= data.len() {
                        return None;
                    }

                    match data[index] {
                        b'\'' | b'"' => {
                            let quote_byte = data[index];
                            index += 1;
                            let value_start = index;
                            while index < data.len() && data[index] != quote_byte {
                                index += 1;
                            }
                            if index >= data.len() {
                                return None;
                            }
                            attr_value.extend_from_slice(&data[value_start..index]);
                            index += 1;
                        }
                        _ => {
                            let value_start = index;
                            while index < data.len()
                                && !matches!(
                                    data[index],
                                    b'\t' | b'\n' | b'\r' | b'\x0C' | b' ' | b'>'
                                )
                            {
                                index += 1;
                            }
                            attr_value.extend_from_slice(&data[value_start..index]);
                        }
                    }
                }

                attrs.push((attr_name, attr_value));
            }
        }
    }

    if !found_tag_end {
        return None;
    }

    let tag = if is_meta {
        ParsedTag::Meta(attrs)
    } else if is_script {
        ParsedTag::Script
    } else {
        ParsedTag::Other
    };
    Some((tag, index))
}

fn scan_tag_end(data: &[u8], mut index: usize, mut quote: Option<u8>) -> usize {
    while index < data.len() {
        let byte = data[index];
        if let Some(q) = quote {
            if byte == q {
                quote = None;
            }
            index += 1;
            continue;
        }

        match byte {
            b'\'' | b'"' => {
                quote = Some(byte);
                index += 1;
            }
            b'>' => return index + 1,
            _ => index += 1,
        }
    }
    data.len()
}

fn detect_meta_encoding(attrs: &[(Vec<u8>, Vec<u8>)]) -> Option<&'static Encoding> {
    for (name, value) in attrs {
        if name == b"charset" {
            if let Some(encoding) = normalize_encoding(value) {
                return Some(encoding);
            }
        }
    }

    let http_equiv = attrs
        .iter()
        .any(|(name, value)| name == b"http-equiv" && eq_ascii_case(value, b"content-type"));
    if !http_equiv {
        return None;
    }

    attrs
        .iter()
        .find(|(name, _)| name == b"content")
        .and_then(|(_, value)| extract_charset_from_content(value))
        .and_then(normalize_encoding)
}

fn detect_scripted_meta_encoding(script: &[u8]) -> Option<&'static Encoding> {
    let lower = ascii_lowercase_bytes(script);
    let mut search_index = 0;

    while let Some(offset) = find_bytes(&lower[search_index..], b"document.write") {
        let mut index = search_index + offset + "document.write".len();
        while index < script.len() && is_ascii_space(script[index]) {
            index += 1;
        }
        if script.get(index) != Some(&b'(') {
            search_index += offset + 1;
            continue;
        }
        index += 1;

        if let Some((markup, end_index)) = parse_document_write_argument(script, index) {
            if let Some(encoding) = sniff_encoding(&markup) {
                return Some(encoding);
            }
            search_index = end_index;
        } else {
            search_index += offset + 1;
        }
    }

    None
}

fn parse_document_write_argument(script: &[u8], mut index: usize) -> Option<(Vec<u8>, usize)> {
    let mut output = Vec::new();

    loop {
        while index < script.len() && is_ascii_space(script[index]) {
            index += 1;
        }
        let quote = *script.get(index)?;
        if !matches!(quote, b'\'' | b'"') {
            return None;
        }
        index += 1;

        while index < script.len() {
            match script[index] {
                b'\\' if index + 1 < script.len() => {
                    output.push(script[index + 1]);
                    index += 2;
                }
                byte if byte == quote => {
                    index += 1;
                    break;
                }
                byte => {
                    output.push(byte);
                    index += 1;
                }
            }
        }

        while index < script.len() && is_ascii_space(script[index]) {
            index += 1;
        }

        match script.get(index) {
            Some(b'+') => index += 1,
            Some(b')') => return Some((output, index + 1)),
            _ => return None,
        }
    }
}

fn extract_charset_from_content(content: &[u8]) -> Option<&[u8]> {
    let mut index = 0;
    while index + 7 <= content.len() {
        if !eq_ascii_case(&content[index..index + 7], b"charset") {
            index += 1;
            continue;
        }

        let mut cursor = index + 7;
        while cursor < content.len() && is_ascii_space(content[cursor]) {
            cursor += 1;
        }
        if cursor >= content.len() || content[cursor] != b'=' {
            index += 1;
            continue;
        }
        cursor += 1;
        while cursor < content.len() && is_ascii_space(content[cursor]) {
            cursor += 1;
        }
        if cursor >= content.len() {
            return None;
        }

        return match content[cursor] {
            b'\'' | b'"' => {
                let quote = content[cursor];
                cursor += 1;
                let start = cursor;
                while cursor < content.len() && content[cursor] != quote {
                    cursor += 1;
                }
                if cursor >= content.len() {
                    None
                } else {
                    Some(&content[start..cursor])
                }
            }
            _ => {
                let start = cursor;
                while cursor < content.len()
                    && !is_ascii_space(content[cursor])
                    && content[cursor] != b';'
                {
                    cursor += 1;
                }
                Some(&content[start..cursor])
            }
        };
    }

    None
}

fn normalize_encoding(label: &[u8]) -> Option<&'static Encoding> {
    let encoding = Encoding::for_label(trim_ascii_space(label))?;
    if encoding == UTF_16BE || encoding == UTF_16LE {
        Some(UTF_8)
    } else {
        Some(encoding)
    }
}

fn trim_ascii_space(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| !is_ascii_space(*byte))
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|byte| !is_ascii_space(*byte))
        .map(|pos| pos + 1)
        .unwrap_or(start);
    &bytes[start..end]
}

fn ascii_lowercase_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut lowercased = bytes.to_vec();
    lowercased.make_ascii_lowercase();
    lowercased
}

fn ascii_lowercase_in_place(bytes: &mut [u8]) {
    bytes.make_ascii_lowercase();
}

fn eq_ascii_case(left: &[u8], right: &[u8]) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn is_ascii_space(byte: u8) -> bool {
    matches!(byte, b'\t' | b'\n' | b'\r' | b'\x0C' | b' ')
}

fn is_tag_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b':')
}

fn find_ascii_case(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle))
}

fn run_encoding_file(path: &Path) -> (usize, usize) {
    let cases = parse_encoding_file(path);
    let mut passed = 0;
    let mut total = 0;

    for case in cases {
        total += 1;
        let actual = detect_encoding(&case.data).name().to_ascii_lowercase();
        if actual == case.expected {
            passed += 1;
        } else {
            eprintln!(
                "FAIL [{}]\n  expected: {}\n  actual:   {}\n  input:    {:?}",
                path.display(),
                case.expected,
                actual,
                String::from_utf8_lossy(&case.data)
            );
        }
    }

    (passed, total)
}

fn collect_encoding_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .flat_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                fs::read_dir(path)
                    .unwrap()
                    .filter_map(|child| child.ok())
                    .map(|child| child.path())
                    .collect::<Vec<_>>()
            } else {
                vec![path]
            }
        })
        .filter(|path| path.extension().is_some_and(|ext| ext == "dat"))
        .collect();
    files.sort();
    files
}

#[test]
fn html5lib_encoding_tests() {
    let encoding_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/html5lib/encoding");
    if !encoding_dir.exists() {
        eprintln!("html5lib encoding tests not found, skipping");
        return;
    }

    let mut total_passed = 0;
    let mut total_tests = 0;

    for path in collect_encoding_files(&encoding_dir) {
        let filename = path
            .strip_prefix(&encoding_dir)
            .unwrap()
            .display()
            .to_string();
        let (passed, total) = run_encoding_file(&path);
        eprintln!("  {filename}: {passed}/{total}");
        total_passed += passed;
        total_tests += total;
    }

    assert_eq!(total_passed, total_tests, "Not all encoding tests passed");
}
