#[derive(Debug, Clone)]
pub(crate) struct TestMetadata {
    pub(crate) features: Vec<String>,
    pub(crate) flags: Vec<String>,
    pub(crate) includes: Vec<String>,
    pub(crate) negative: Option<NegativeExpectation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NegativeExpectation {
    pub(crate) phase: String,
    pub(crate) error_type: Option<String>,
}

pub(crate) fn parse_metadata(source: &str) -> TestMetadata {
    let mut features = Vec::new();
    let mut flags = Vec::new();
    let mut includes = Vec::new();
    let mut negative = None;

    let start = source.find("/*---");
    let end = source.find("---*/");
    if let (Some(start), Some(end)) = (start, end) {
        let yaml = source[start + 5..end]
            .replace("\r\n", "\n")
            .replace('\r', "\n");
        let mut in_negative = false;
        let mut in_features = false;
        let mut in_flags = false;
        let mut in_includes = false;
        let mut negative_phase = String::new();
        let mut negative_type = None;

        for line in yaml.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("features:") {
                in_features = true;
                in_flags = false;
                in_includes = false;
                in_negative = false;
                if let Some(rest) = trimmed.strip_prefix("features:") {
                    parse_inline_list(rest, &mut features);
                }
                continue;
            }
            if trimmed.starts_with("flags:") {
                in_flags = true;
                in_features = false;
                in_includes = false;
                in_negative = false;
                if let Some(rest) = trimmed.strip_prefix("flags:") {
                    parse_inline_list(rest, &mut flags);
                }
                continue;
            }
            if trimmed.starts_with("includes:") {
                in_includes = true;
                in_features = false;
                in_flags = false;
                in_negative = false;
                if let Some(rest) = trimmed.strip_prefix("includes:") {
                    parse_inline_list(rest, &mut includes);
                }
                continue;
            }
            if trimmed.starts_with("negative:") {
                in_negative = true;
                in_features = false;
                in_flags = false;
                in_includes = false;
                continue;
            }

            let is_indented = line.starts_with(' ') || line.starts_with('\t');
            if !is_indented && !trimmed.starts_with('-') && trimmed.contains(':') && !in_negative {
                in_features = false;
                in_flags = false;
                in_includes = false;
                continue;
            }

            if in_features && trimmed.starts_with('-') {
                features.push(trimmed.trim_start_matches('-').trim().to_string());
            }
            if in_flags && trimmed.starts_with('-') {
                flags.push(trimmed.trim_start_matches('-').trim().to_string());
            }
            if in_includes && trimmed.starts_with('-') {
                includes.push(trimmed.trim_start_matches('-').trim().to_string());
            }
            if in_negative && trimmed.starts_with("phase:") {
                negative_phase = trimmed.trim_start_matches("phase:").trim().to_string();
            }
            if in_negative && trimmed.starts_with("type:") {
                negative_type = Some(trimmed.trim_start_matches("type:").trim().to_string());
            }
        }

        if !negative_phase.is_empty() {
            negative = Some(NegativeExpectation {
                phase: negative_phase,
                error_type: negative_type,
            });
        }
    }

    TestMetadata {
        features,
        flags,
        includes,
        negative,
    }
}

fn parse_inline_list(rest: &str, dest: &mut Vec<String>) {
    let rest = rest.trim();
    if !rest.starts_with('[') || !rest.ends_with(']') {
        return;
    }
    for item in rest[1..rest.len() - 1].split(',') {
        let item = item.trim();
        if !item.is_empty() {
            dest.push(item.to_string());
        }
    }
}

pub(crate) fn is_module_test(metadata: &TestMetadata) -> bool {
    metadata.flags.iter().any(|flag| flag == "module")
}

pub(crate) fn has_async_flag(metadata: &TestMetadata) -> bool {
    metadata.flags.iter().any(|flag| flag == "async")
}

pub(crate) fn effective_parse_source(source: &str, metadata: &TestMetadata) -> String {
    if metadata.flags.iter().any(|flag| flag == "onlyStrict") {
        format!("\"use strict\";\n{source}")
    } else {
        source.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::parse_metadata;

    #[test]
    fn parse_metadata_preserves_negative_type() {
        let metadata = parse_metadata(
            r"
            /*---
            negative:
              phase: runtime
              type: TypeError
            ---*/
            throw new TypeError();
            ",
        );

        let negative = metadata
            .negative
            .as_ref()
            .expect("negative metadata should be parsed");
        assert_eq!(negative.phase, "runtime");
        assert_eq!(negative.error_type.as_deref(), Some("TypeError"));
    }
}
