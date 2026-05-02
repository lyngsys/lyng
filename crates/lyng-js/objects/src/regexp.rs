use regress::Regex;
use std::{fmt::Write as _, mem::size_of, ops::Range};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RegExpObjectFlags(u8);

impl RegExpObjectFlags {
    const HAS_INDICES: u8 = 1 << 0;
    const GLOBAL: u8 = 1 << 1;
    const IGNORE_CASE: u8 = 1 << 2;
    const MULTILINE: u8 = 1 << 3;
    const DOT_ALL: u8 = 1 << 4;
    const UNICODE: u8 = 1 << 5;
    const UNICODE_SETS: u8 = 1 << 6;
    const STICKY: u8 = 1 << 7;

    #[inline]
    pub fn from_flag_text(flags: &str) -> Self {
        let mut parsed = Self::default();
        for ch in flags.chars() {
            parsed = match ch {
                'd' => parsed.with_has_indices(true),
                'g' => parsed.with_global(true),
                'i' => parsed.with_ignore_case(true),
                'm' => parsed.with_multiline(true),
                's' => parsed.with_dot_all(true),
                'u' => parsed.with_unicode(true),
                'v' => parsed.with_unicode_sets(true),
                'y' => parsed.with_sticky(true),
                _ => parsed,
            };
        }
        parsed
    }

    #[inline]
    pub const fn has_indices(self) -> bool {
        (self.0 & Self::HAS_INDICES) != 0
    }

    #[inline]
    pub const fn global(self) -> bool {
        (self.0 & Self::GLOBAL) != 0
    }

    #[inline]
    pub const fn ignore_case(self) -> bool {
        (self.0 & Self::IGNORE_CASE) != 0
    }

    #[inline]
    pub const fn multiline(self) -> bool {
        (self.0 & Self::MULTILINE) != 0
    }

    #[inline]
    pub const fn dot_all(self) -> bool {
        (self.0 & Self::DOT_ALL) != 0
    }

    #[inline]
    pub const fn unicode(self) -> bool {
        (self.0 & Self::UNICODE) != 0
    }

    #[inline]
    pub const fn unicode_sets(self) -> bool {
        (self.0 & Self::UNICODE_SETS) != 0
    }

    #[inline]
    pub const fn sticky(self) -> bool {
        (self.0 & Self::STICKY) != 0
    }

    #[inline]
    pub const fn unicode_aware(self) -> bool {
        self.unicode() || self.unicode_sets()
    }

    #[inline]
    pub fn ordered_flag_text(self) -> String {
        let mut text = String::with_capacity(8);
        if self.has_indices() {
            text.push('d');
        }
        if self.global() {
            text.push('g');
        }
        if self.ignore_case() {
            text.push('i');
        }
        if self.multiline() {
            text.push('m');
        }
        if self.dot_all() {
            text.push('s');
        }
        if self.unicode() {
            text.push('u');
        }
        if self.unicode_sets() {
            text.push('v');
        }
        if self.sticky() {
            text.push('y');
        }
        text
    }

    #[inline]
    pub fn compile_flags(self) -> regress::Flags {
        regress::Flags {
            icase: self.ignore_case(),
            multiline: self.multiline(),
            dot_all: self.dot_all(),
            no_opt: false,
            unicode: self.unicode(),
            unicode_sets: self.unicode_sets(),
        }
    }

    #[inline]
    const fn with_has_indices(mut self, enabled: bool) -> Self {
        self = Self::with_bit(self, Self::HAS_INDICES, enabled);
        self
    }

    #[inline]
    const fn with_global(mut self, enabled: bool) -> Self {
        self = Self::with_bit(self, Self::GLOBAL, enabled);
        self
    }

    #[inline]
    const fn with_ignore_case(mut self, enabled: bool) -> Self {
        self = Self::with_bit(self, Self::IGNORE_CASE, enabled);
        self
    }

    #[inline]
    const fn with_multiline(mut self, enabled: bool) -> Self {
        self = Self::with_bit(self, Self::MULTILINE, enabled);
        self
    }

    #[inline]
    const fn with_dot_all(mut self, enabled: bool) -> Self {
        self = Self::with_bit(self, Self::DOT_ALL, enabled);
        self
    }

    #[inline]
    const fn with_unicode(mut self, enabled: bool) -> Self {
        self = Self::with_bit(self, Self::UNICODE, enabled);
        self
    }

    #[inline]
    const fn with_unicode_sets(mut self, enabled: bool) -> Self {
        self = Self::with_bit(self, Self::UNICODE_SETS, enabled);
        self
    }

    #[inline]
    const fn with_sticky(mut self, enabled: bool) -> Self {
        self = Self::with_bit(self, Self::STICKY, enabled);
        self
    }

    #[inline]
    const fn with_bit(mut current: Self, bit: u8, enabled: bool) -> Self {
        if enabled {
            current.0 |= bit;
        } else {
            current.0 &= !bit;
        }
        current
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpNamedCapture {
    name: Box<str>,
    range: Option<Range<usize>>,
}

impl RegExpNamedCapture {
    #[inline]
    pub fn new(name: Box<str>, range: Option<Range<usize>>) -> Self {
        Self { name, range }
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn range(&self) -> Option<Range<usize>> {
        self.range.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegExpMatchRecord {
    range: Range<usize>,
    captures: Box<[Option<Range<usize>>]>,
    named_captures: Box<[RegExpNamedCapture]>,
}

impl RegExpMatchRecord {
    #[inline]
    pub fn new(
        range: Range<usize>,
        captures: Box<[Option<Range<usize>>]>,
        named_captures: Box<[RegExpNamedCapture]>,
    ) -> Self {
        Self {
            range,
            captures,
            named_captures,
        }
    }

    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    #[inline]
    pub fn start(&self) -> usize {
        self.range.start
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.range.end
    }

    #[inline]
    pub fn captures(&self) -> &[Option<Range<usize>>] {
        &self.captures
    }

    #[inline]
    pub fn named_captures(&self) -> &[RegExpNamedCapture] {
        &self.named_captures
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RegExpPayloadAccounting {
    pub records: usize,
    pub metadata_bytes: usize,
    pub payload_bytes: usize,
    pub live_bytes: usize,
}

#[derive(Clone, Debug)]
pub struct RegExpPayload {
    source: Box<str>,
    source_units: Option<Box<[u16]>>,
    flags: RegExpObjectFlags,
    flag_text: Box<str>,
    backend: Regex,
    fast_pattern: Option<RegExpFastPattern>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RegExpFastPattern {
    AsciiDigit,
    AsciiNonDigit,
    Whitespace,
    NonWhitespace,
    AsciiWord,
    AsciiNonWord,
    AnchoredAsciiDigitRun,
    AnchoredAsciiNonDigitRun,
    AnchoredWhitespaceRun,
    AnchoredNonWhitespaceRun,
    AnchoredAsciiWordRun,
    AnchoredAsciiNonWordRun,
}

fn normalize_backend_pattern(pattern: &str, flags: RegExpObjectFlags) -> String {
    const UNKNOWN_SCRIPT_SET: &str =
        r"[\P{Assigned}\p{General_Category=Surrogate}\p{General_Category=Private_Use}]";

    let pattern = if flags.unicode_aware() {
        pattern.to_owned()
    } else {
        normalize_legacy_identity_escapes(pattern)
    };
    let normalized = [
        (r"\p{Script=Unknown}", UNKNOWN_SCRIPT_SET),
        (r"\p{Script=Zzzz}", UNKNOWN_SCRIPT_SET),
        (r"\p{sc=Unknown}", UNKNOWN_SCRIPT_SET),
        (r"\p{sc=Zzzz}", UNKNOWN_SCRIPT_SET),
        (r"\p{Script_Extensions=Unknown}", UNKNOWN_SCRIPT_SET),
        (r"\p{Script_Extensions=Zzzz}", UNKNOWN_SCRIPT_SET),
        (r"\p{scx=Unknown}", UNKNOWN_SCRIPT_SET),
        (r"\p{scx=Zzzz}", UNKNOWN_SCRIPT_SET),
        (r"\P{Script=Unknown}", r"\p{Assigned}"),
        (r"\P{Script=Zzzz}", r"\p{Assigned}"),
        (r"\P{sc=Unknown}", r"\p{Assigned}"),
        (r"\P{sc=Zzzz}", r"\p{Assigned}"),
        (r"\P{Script_Extensions=Unknown}", r"\p{Assigned}"),
        (r"\P{Script_Extensions=Zzzz}", r"\p{Assigned}"),
        (r"\P{scx=Unknown}", r"\p{Assigned}"),
        (r"\P{scx=Zzzz}", r"\p{Assigned}"),
    ]
    .into_iter()
    .fold(pattern, |current, (from, to)| current.replace(from, to));

    if flags.unicode_aware() {
        normalized
    } else {
        expand_astral_source_for_ucs2(&normalized)
    }
}

fn normalize_legacy_identity_escapes(pattern: &str) -> String {
    let chars = pattern.chars().collect::<Vec<_>>();
    let mut normalized = String::with_capacity(pattern.len());
    let mut in_class = false;
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if !in_class {
            if let Some(next) =
                normalize_legacy_quantifiable_assertion(&chars, index, &mut normalized)
            {
                index = next;
                continue;
            }
        }
        if ch == '[' && !in_class {
            in_class = true;
            normalized.push(ch);
            index += 1;
            continue;
        }
        if ch == ']' && in_class {
            in_class = false;
            normalized.push(ch);
            index += 1;
            continue;
        }
        if !in_class {
            if let Some(next) = copy_valid_legacy_braced_quantifier(&chars, index, &mut normalized)
            {
                index = next;
                continue;
            }
            if matches!(ch, ']' | '{' | '}') {
                normalized.push('\\');
                normalized.push(ch);
                index += 1;
                continue;
            }
        }
        if ch != '\\' {
            normalized.push(ch);
            index += 1;
            continue;
        }

        let Some(&escaped) = chars.get(index + 1) else {
            normalized.push(ch);
            index += 1;
            continue;
        };

        match escaped {
            'c' => {
                if let Some(&control) = chars.get(index + 2) {
                    if in_class && (control.is_ascii_digit() || control == '_') {
                        write!(normalized, r"\u{:04X}", u32::from(control) % 32)
                            .expect("writing to String cannot fail");
                        index += 3;
                        continue;
                    }
                    if control.is_ascii_alphabetic() {
                        normalized.push('\\');
                        normalized.push('c');
                        normalized.push(control);
                        index += 3;
                        continue;
                    }
                }
                normalized.push_str(r"\\c");
                index += 2;
            }
            'x' => {
                if has_hex_digits(&chars, index + 2, 2) {
                    normalized.extend(chars[index..index + 4].iter());
                    index += 4;
                } else {
                    push_backend_literal(&mut normalized, escaped, in_class);
                    index += 2;
                }
            }
            'u' => {
                if has_hex_digits(&chars, index + 2, 4) {
                    normalized.extend(chars[index..index + 6].iter());
                    index += 6;
                } else {
                    push_backend_literal(&mut normalized, escaped, in_class);
                    index += 2;
                }
            }
            'k' if chars.get(index + 2) == Some(&'<') => {
                normalized.push('\\');
                normalized.push(escaped);
                index += 2;
            }
            escaped
                if is_legacy_backend_escape(escaped)
                    || escaped.is_ascii_digit()
                    || is_escaped_syntax_character(escaped, in_class) =>
            {
                normalized.push('\\');
                normalized.push(escaped);
                index += 2;
            }
            _ => {
                push_backend_literal(&mut normalized, escaped, in_class);
                index += 2;
            }
        }
    }
    normalized
}

fn normalize_legacy_quantifiable_assertion(
    chars: &[char],
    start: usize,
    normalized: &mut String,
) -> Option<usize> {
    if chars.get(start) != Some(&'(')
        || chars.get(start + 1) != Some(&'?')
        || !matches!(chars.get(start + 2), Some('=') | Some('!'))
    {
        return None;
    }

    let group_end = assertion_group_end(chars, start)?;
    let (quantifier_end, requires_assertion) = legacy_assertion_quantifier(chars, group_end)?;
    if requires_assertion {
        normalized.extend(chars[start..group_end].iter());
    }
    Some(quantifier_end)
}

fn assertion_group_end(chars: &[char], start: usize) -> Option<usize> {
    let mut index = start + 3;
    let mut depth = 1usize;
    let mut in_class = false;
    while index < chars.len() {
        match chars[index] {
            '\\' => index += 2,
            '[' if !in_class => {
                in_class = true;
                index += 1;
            }
            ']' if in_class => {
                in_class = false;
                index += 1;
            }
            '(' if !in_class => {
                depth += 1;
                index += 1;
            }
            ')' if !in_class => {
                depth = depth.saturating_sub(1);
                index += 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => index += 1,
        }
    }
    None
}

fn legacy_assertion_quantifier(chars: &[char], start: usize) -> Option<(usize, bool)> {
    let (mut end, requires_assertion) = match chars.get(start)? {
        '*' | '?' => (start + 1, false),
        '+' => (start + 1, true),
        '{' => (valid_legacy_braced_quantifier_end(chars, start)?, true),
        _ => return None,
    };
    if chars.get(end) == Some(&'?') {
        end += 1;
    }
    Some((end, requires_assertion))
}

fn copy_valid_legacy_braced_quantifier(
    chars: &[char],
    start: usize,
    normalized: &mut String,
) -> Option<usize> {
    let end = valid_legacy_braced_quantifier_end(chars, start)?;
    normalized.extend(chars[start..end].iter());
    Some(end)
}

fn valid_legacy_braced_quantifier_end(chars: &[char], start: usize) -> Option<usize> {
    if chars.get(start) != Some(&'{') {
        return None;
    }
    let mut end = start + 1;
    while end < chars.len() && chars[end] != '}' {
        end += 1;
    }
    if end >= chars.len() {
        return None;
    }

    let inner = &chars[start + 1..end];
    if inner.is_empty() {
        return None;
    }
    let valid = if let Some(comma) = inner.iter().position(|&ch| ch == ',') {
        let lhs = &inner[..comma];
        let rhs = &inner[comma + 1..];
        if lhs.is_empty() || !lhs.iter().all(|ch| ch.is_ascii_digit()) {
            false
        } else if rhs.is_empty() {
            true
        } else if !rhs.iter().all(|ch| ch.is_ascii_digit()) {
            false
        } else {
            let lhs = digits_to_u32(lhs);
            let rhs = digits_to_u32(rhs);
            lhs <= rhs
        }
    } else {
        inner.iter().all(|ch| ch.is_ascii_digit())
    };

    valid.then_some(end + 1)
}

fn digits_to_u32(chars: &[char]) -> u32 {
    chars.iter().fold(0u32, |acc, ch| {
        acc.saturating_mul(10)
            .saturating_add(ch.to_digit(10).unwrap_or(0))
    })
}

fn is_legacy_backend_escape(ch: char) -> bool {
    matches!(
        ch,
        'b' | 'B' | 'd' | 'D' | 's' | 'S' | 'w' | 'W' | 'f' | 'n' | 'r' | 't' | 'v'
    )
}

fn is_escaped_syntax_character(ch: char, in_class: bool) -> bool {
    if in_class {
        matches!(ch, '\\' | ']' | '-' | '^')
    } else {
        matches!(
            ch,
            '^' | '$' | '\\' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|'
        )
    }
}

fn push_backend_literal(normalized: &mut String, ch: char, in_class: bool) {
    match ch {
        '\0'..='\u{001F}' | '\u{007F}' => {
            write!(normalized, r"\u{:04X}", u32::from(ch)).expect("writing to String cannot fail");
        }
        '\\' => normalized.push_str(r"\\"),
        ']' if in_class => normalized.push_str(r"\]"),
        '-' if in_class => normalized.push_str(r"\-"),
        '^' if in_class => normalized.push_str(r"\^"),
        _ => normalized.push(ch),
    }
}

fn has_hex_digits(chars: &[char], start: usize, count: usize) -> bool {
    (0..count).all(|offset| {
        chars
            .get(start + offset)
            .is_some_and(char::is_ascii_hexdigit)
    })
}

fn expand_astral_source_for_ucs2(pattern: &str) -> String {
    if !pattern.chars().any(|ch| u32::from(ch) > 0xFFFF) {
        return pattern.to_owned();
    }

    let chars = pattern.chars().collect::<Vec<_>>();
    let mut expanded = String::with_capacity(pattern.len());
    let mut in_class = false;
    let mut index = 0;
    while index < chars.len() {
        if !in_class {
            if let Some(end) = named_capture_span_end(&chars, index) {
                expanded.extend(chars[index..end].iter());
                index = end;
                continue;
            }
            if let Some(end) = named_reference_span_end(&chars, index) {
                expanded.extend(chars[index..end].iter());
                index = end;
                continue;
            }
        }

        let ch = chars[index];
        if ch == '\\' {
            expanded.push(ch);
            index += 1;
            if let Some(&escaped) = chars.get(index) {
                expanded.push(escaped);
                index += 1;
            }
            continue;
        }

        if ch == '[' && !in_class {
            expanded.push(ch);
            in_class = true;
            index += 1;
            continue;
        }

        if ch == ']' && in_class {
            expanded.push(ch);
            in_class = false;
            index += 1;
            continue;
        }

        let code_point = u32::from(ch);
        if code_point <= 0xFFFF {
            expanded.push(ch);
            index += 1;
            continue;
        }

        let scalar = code_point - 0x1_0000;
        let high = 0xD800 + (scalar >> 10);
        let low = 0xDC00 + (scalar & 0x3FF);
        if in_class {
            write!(&mut expanded, r"\u{high:04X}\u{low:04X}")
                .expect("writing to String cannot fail");
        } else {
            write!(&mut expanded, r"[\u{high:04X}][\u{low:04X}]")
                .expect("writing to String cannot fail");
        }
        index += 1;
    }
    expanded
}

fn named_capture_span_end(chars: &[char], start: usize) -> Option<usize> {
    if chars.get(start) != Some(&'(')
        || chars.get(start + 1) != Some(&'?')
        || chars.get(start + 2) != Some(&'<')
        || matches!(chars.get(start + 3), Some('=') | Some('!'))
    {
        return None;
    }
    angle_name_span_end(chars, start + 3)
}

fn named_reference_span_end(chars: &[char], start: usize) -> Option<usize> {
    if chars.get(start) != Some(&'\\')
        || chars.get(start + 1) != Some(&'k')
        || chars.get(start + 2) != Some(&'<')
    {
        return None;
    }
    angle_name_span_end(chars, start + 3)
}

fn angle_name_span_end(chars: &[char], mut index: usize) -> Option<usize> {
    while index < chars.len() {
        if chars[index] == '>' {
            return Some(index + 1);
        }
        index += 1;
    }
    None
}

impl RegExpPayload {
    pub fn compile(pattern: &str, flags: &str) -> Result<Self, regress::Error> {
        let parsed_flags = RegExpObjectFlags::from_flag_text(flags);
        let backend_pattern = normalize_backend_pattern(pattern, parsed_flags);
        let backend = Regex::with_flags(&backend_pattern, parsed_flags.compile_flags())?;
        let fast_pattern = detect_fast_pattern(pattern, parsed_flags);
        Ok(Self {
            source: pattern.into(),
            source_units: None,
            flags: parsed_flags,
            flag_text: parsed_flags.ordered_flag_text().into_boxed_str(),
            backend,
            fast_pattern,
        })
    }

    pub fn compile_with_source_units(
        pattern: &str,
        source_units: Box<[u16]>,
        flags: &str,
    ) -> Result<Self, regress::Error> {
        let mut payload = Self::compile(pattern, flags)?;
        payload.source_units = Some(source_units);
        Ok(payload)
    }

    #[inline]
    pub fn source(&self) -> &str {
        &self.source
    }

    #[inline]
    pub fn source_units(&self) -> Option<&[u16]> {
        self.source_units.as_deref()
    }

    #[inline]
    pub fn flag_text(&self) -> &str {
        &self.flag_text
    }

    #[inline]
    pub const fn flags(&self) -> RegExpObjectFlags {
        self.flags
    }

    #[inline]
    pub fn payload_bytes(&self) -> usize {
        self.source.len()
            + self
                .source_units
                .as_ref()
                .map_or(0, |units| units.len() * size_of::<u16>())
            + self.flag_text.len()
            + size_of::<Regex>()
    }

    pub fn find_from_code_units(&self, text: &[u16], start: usize) -> Option<RegExpMatchRecord> {
        if let Some(matched) = self.find_fast_from_code_units(text, start) {
            return matched;
        }
        let matched = if self.flags.unicode_aware() {
            self.backend.find_from_utf16(text, start).next()?
        } else {
            self.backend.find_from_ucs2(text, start).next()?
        };
        let named_captures = matched
            .named_groups()
            .map(|(name, range)| RegExpNamedCapture::new(name.into(), range))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let captures = matched.captures.into_boxed_slice();
        Some(RegExpMatchRecord::new(
            matched.range,
            captures,
            named_captures,
        ))
    }

    fn find_fast_from_code_units(
        &self,
        text: &[u16],
        start: usize,
    ) -> Option<Option<RegExpMatchRecord>> {
        match self.fast_pattern? {
            RegExpFastPattern::AsciiDigit => {
                Some(self.find_fast_class(text, start, is_ascii_digit_code_unit))
            }
            RegExpFastPattern::AsciiNonDigit => {
                Some(self.find_fast_class(text, start, |unit| !is_ascii_digit_code_unit(unit)))
            }
            RegExpFastPattern::Whitespace => {
                Some(self.find_fast_class(text, start, is_js_whitespace_or_line_terminator))
            }
            RegExpFastPattern::NonWhitespace => Some(self.find_fast_class(text, start, |unit| {
                !is_js_whitespace_or_line_terminator(unit)
            })),
            RegExpFastPattern::AsciiWord => {
                Some(self.find_fast_class(text, start, is_ascii_word_code_unit))
            }
            RegExpFastPattern::AsciiNonWord => {
                Some(self.find_fast_class(text, start, |unit| !is_ascii_word_code_unit(unit)))
            }
            RegExpFastPattern::AnchoredAsciiDigitRun => {
                Some(self.match_fast_anchored_run(text, start, is_ascii_digit_code_unit))
            }
            RegExpFastPattern::AnchoredAsciiNonDigitRun => Some(self.match_fast_anchored_run(
                text,
                start,
                |unit| !is_ascii_digit_code_unit(unit),
            )),
            RegExpFastPattern::AnchoredWhitespaceRun => {
                Some(self.match_fast_anchored_run(text, start, is_js_whitespace_or_line_terminator))
            }
            RegExpFastPattern::AnchoredNonWhitespaceRun => {
                Some(self.match_fast_anchored_run(text, start, |unit| {
                    !is_js_whitespace_or_line_terminator(unit)
                }))
            }
            RegExpFastPattern::AnchoredAsciiWordRun => {
                Some(self.match_fast_anchored_run(text, start, is_ascii_word_code_unit))
            }
            RegExpFastPattern::AnchoredAsciiNonWordRun => Some(self.match_fast_anchored_run(
                text,
                start,
                |unit| !is_ascii_word_code_unit(unit),
            )),
        }
    }

    fn find_fast_class(
        &self,
        text: &[u16],
        start: usize,
        predicate: impl Fn(u16) -> bool,
    ) -> Option<RegExpMatchRecord> {
        let index = text.get(start..).and_then(|tail| {
            tail.iter()
                .position(|unit| predicate(*unit))
                .map(|offset| start + offset)
        })?;
        Some(simple_match_record(
            index..index + fast_match_code_unit_width(text, index, self.flags.unicode_aware()),
        ))
    }

    fn match_fast_anchored_run(
        &self,
        text: &[u16],
        start: usize,
        predicate: impl Fn(u16) -> bool,
    ) -> Option<RegExpMatchRecord> {
        if start != 0 || text.is_empty() || text.iter().any(|unit| !predicate(*unit)) {
            return None;
        }
        Some(simple_match_record(0..text.len()))
    }
}

fn detect_fast_pattern(pattern: &str, flags: RegExpObjectFlags) -> Option<RegExpFastPattern> {
    let word_classes_are_ascii = !flags.ignore_case();
    match pattern {
        r"\d" => Some(RegExpFastPattern::AsciiDigit),
        r"\D" => Some(RegExpFastPattern::AsciiNonDigit),
        r"\s" => Some(RegExpFastPattern::Whitespace),
        r"\S" => Some(RegExpFastPattern::NonWhitespace),
        r"\w" if word_classes_are_ascii => Some(RegExpFastPattern::AsciiWord),
        r"\W" if word_classes_are_ascii => Some(RegExpFastPattern::AsciiNonWord),
        r"^\d+$" if !flags.multiline() => Some(RegExpFastPattern::AnchoredAsciiDigitRun),
        r"^\D+$" if !flags.multiline() => Some(RegExpFastPattern::AnchoredAsciiNonDigitRun),
        r"^\s+$" if !flags.multiline() => Some(RegExpFastPattern::AnchoredWhitespaceRun),
        r"^\S+$" if !flags.multiline() => Some(RegExpFastPattern::AnchoredNonWhitespaceRun),
        r"^\w+$" if !flags.multiline() && word_classes_are_ascii => {
            Some(RegExpFastPattern::AnchoredAsciiWordRun)
        }
        r"^\W+$" if !flags.multiline() && word_classes_are_ascii => {
            Some(RegExpFastPattern::AnchoredAsciiNonWordRun)
        }
        _ => None,
    }
}

#[inline]
fn is_ascii_digit_code_unit(unit: u16) -> bool {
    (0x30..=0x39).contains(&unit)
}

#[inline]
fn is_ascii_word_code_unit(unit: u16) -> bool {
    is_ascii_digit_code_unit(unit)
        || (0x41..=0x5A).contains(&unit)
        || unit == 0x5F
        || (0x61..=0x7A).contains(&unit)
}

#[inline]
fn is_js_whitespace_or_line_terminator(unit: u16) -> bool {
    matches!(
        unit,
        0x0009
            | 0x000A
            | 0x000B
            | 0x000C
            | 0x000D
            | 0x0020
            | 0x00A0
            | 0x1680
            | 0x2028
            | 0x2029
            | 0x202F
            | 0x205F
            | 0x3000
            | 0xFEFF
    ) || (0x2000..=0x200A).contains(&unit)
}

#[inline]
fn fast_match_code_unit_width(text: &[u16], index: usize, unicode_aware: bool) -> usize {
    if !unicode_aware {
        return 1;
    }
    let Some(unit) = text.get(index).copied() else {
        return 1;
    };
    if !(0xD800..=0xDBFF).contains(&unit) {
        return 1;
    }
    text.get(index + 1)
        .copied()
        .filter(|trail| (0xDC00..=0xDFFF).contains(trail))
        .map_or(1, |_| 2)
}

fn simple_match_record(range: Range<usize>) -> RegExpMatchRecord {
    RegExpMatchRecord::new(
        range,
        Vec::<Option<Range<usize>>>::new().into_boxed_slice(),
        Vec::<RegExpNamedCapture>::new().into_boxed_slice(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags(text: &str) -> RegExpObjectFlags {
        RegExpObjectFlags::from_flag_text(text)
    }

    #[test]
    fn detects_fast_character_class_patterns() {
        assert_eq!(
            detect_fast_pattern(r"\s", flags("")),
            Some(RegExpFastPattern::Whitespace)
        );
        assert_eq!(
            detect_fast_pattern(r"\w", flags("u")),
            Some(RegExpFastPattern::AsciiWord)
        );
        assert_eq!(
            detect_fast_pattern(r"^\S+$", flags("v")),
            Some(RegExpFastPattern::AnchoredNonWhitespaceRun)
        );
        assert_eq!(
            detect_fast_pattern(r"^\W+$", flags("")),
            Some(RegExpFastPattern::AnchoredAsciiNonWordRun)
        );
    }
}
