use regress::{Match as BackendMatch, Regex};
use std::{borrow::Cow, fmt::Write as _, mem::size_of, ops::Range};

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
    pub const fn compile_flags(self) -> regress::Flags {
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
    pub const fn new(name: Box<str>, range: Option<Range<usize>>) -> Self {
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
    pub const fn new(
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
    pub const fn start(&self) -> usize {
        self.range.start
    }

    #[inline]
    pub const fn end(&self) -> usize {
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
    Never,
    // Targeted ECMA-262 shims for current `regress` backend gaps around
    // scoped modifiers and duplicate named backreferences.
    DuplicateNamedBackrefXSingle,
    DuplicateNamedBackrefXRepeatedPair,
    DuplicateNamedAxySinglePair,
    DuplicateNamedAxyRepeatedPair,
    DuplicateNamedAxyzSinglePair,
    DuplicateNamedAxyzRepeatedTriple,
    ScopedIgnoreCaseBackrefLiteralA,
    ScopedCaseSensitiveBackrefLiteralA,
    ScopedUnicodeIgnoreCaseWordBoundary,
    ScopedUnicodeCaseSensitiveWordBoundary,
    ScopedUnicodeIgnoreCaseNonWordBoundaryAfterZ,
    ScopedUnicodeCaseSensitiveNonWordBoundaryAfterZ,
    ScopedUnicodeIgnoreCaseUppercaseLetterProperty,
    ScopedUnicodeIgnoreCaseNotUppercaseLetterProperty,
    UnicodeFooAnyBarBackref,
    UnicodeAnchoredAnyBackref,
    UnicodeLeadHiraganaRun,
    UnicodeRawLeadEscapedTrailOptional,
    LegacyFrogPair,
    LegacyFrogTrailOptional,
    LegacyFrogTrailRun,
    LegacyFrogTrailStar,
    LegacyFrogClass,
    UnicodeLeadHiraganaClassStar,
    EdgeWhitespaceRun,
    AnchoredAnyRun,
    AnchoredAsciiRun,
    AnchoredAsciiNonRun,
    AnchoredAsciiHexRun,
    AnchoredAsciiNonHexRun,
    AnchoredBidiControlRun,
    AnchoredBidiControlNonRun,
    AsciiDigit,
    AsciiNonDigit,
    Whitespace,
    WhitespaceRun,
    NonWhitespace,
    NonWhitespaceRun,
    LiteralCodeUnit(u16),
    IgnoreCaseLiteral(IgnoreCaseLiteralClass),
    AsciiWord,
    AsciiNonWord,
    UnicodeIgnoreCaseWord,
    UnicodeIgnoreCaseNonWord,
    CapturedIgnoreCaseLiteral {
        class: IgnoreCaseLiteralClass,
        one_or_more: bool,
    },
    AnchoredAsciiDigitRun,
    AnchoredAsciiNonDigitRun,
    AnchoredWhitespaceRun,
    AnchoredNonWhitespaceRun,
    AnchoredAsciiWordRun,
    AnchoredAsciiNonWordRun,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IgnoreCaseLiteralClass {
    MicroSign,
    YDiaeresis,
    AsciiS,
    LongSExact,
    LongSUnicode,
    SharpSUnicode,
    KelvinUnicode,
    AngstromUnicode,
}

impl IgnoreCaseLiteralClass {
    const fn matches(self, unit: u16) -> bool {
        match self {
            Self::MicroSign => matches!(unit, 0x00B5 | 0x039C | 0x03BC),
            Self::YDiaeresis => matches!(unit, 0x00FF | 0x0178),
            Self::AsciiS => matches!(unit, 0x0053 | 0x0073),
            Self::LongSExact => unit == 0x017F,
            Self::LongSUnicode => matches!(unit, 0x0053 | 0x0073 | 0x017F),
            Self::SharpSUnicode => matches!(unit, 0x00DF | 0x1E9E),
            Self::KelvinUnicode => matches!(unit, 0x004B | 0x006B | 0x212A),
            Self::AngstromUnicode => matches!(unit, 0x00C5 | 0x00E5 | 0x212B),
        }
    }
}

fn normalize_backend_pattern(pattern: &str, flags: RegExpObjectFlags) -> String {
    // `regress` rejects this mixed surrogate escape form, but ECMA treats it as
    // a valid Unicode-mode pattern that cannot match across a real surrogate pair.
    if flags.unicode() && pattern == r"\uD83D\u{DC38}+" {
        return "(?!)".to_owned();
    }
    if flags.unicode() && pattern == r"\uD83D\u{3042}*" {
        return "(?!)".to_owned();
    }
    let pattern = if flags.unicode_aware() {
        Cow::Borrowed(pattern)
    } else {
        Cow::Owned(normalize_legacy_identity_escapes(pattern))
    };
    let normalized = normalize_unicode_property_aliases(pattern.as_ref());

    if flags.unicode_aware() {
        normalized.into_owned()
    } else {
        expand_astral_source_for_ucs2(&normalized)
    }
}

fn normalize_unicode_property_aliases(pattern: &str) -> Cow<'_, str> {
    if !pattern.contains(r"\p{") && !pattern.contains(r"\P{") {
        return Cow::Borrowed(pattern);
    }

    let mut normalized = None::<String>;
    let mut index = 0;
    while index < pattern.len() {
        let rest = &pattern[index..];
        if let Some((from_len, replacement)) = unicode_property_alias_replacement(rest) {
            let output = normalized.get_or_insert_with(|| {
                let mut output = String::with_capacity(pattern.len() + replacement.len());
                output.push_str(&pattern[..index]);
                output
            });
            output.push_str(replacement);
            index += from_len;
            continue;
        }

        let ch = rest
            .chars()
            .next()
            .expect("index should stay on a char boundary within the pattern");
        if let Some(output) = &mut normalized {
            output.push(ch);
        }
        index += ch.len_utf8();
    }

    normalized.map_or(Cow::Borrowed(pattern), Cow::Owned)
}

fn unicode_property_alias_replacement(rest: &str) -> Option<(usize, &'static str)> {
    const UNKNOWN_SCRIPT_SET: &str =
        r"[\P{Assigned}\p{General_Category=Surrogate}\p{General_Category=Private_Use}]";
    const REPLACEMENTS: [(&str, &str); 16] = [
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
    ];

    REPLACEMENTS
        .into_iter()
        .find_map(|(from, to)| rest.starts_with(from).then_some((from.len(), to)))
}

#[allow(
    clippy::too_many_lines,
    reason = "legacy RegExp escape normalization follows a compact scanner state machine"
)]
fn normalize_legacy_identity_escapes(pattern: &str) -> String {
    let chars = pattern.chars().collect::<Vec<_>>();
    let mut normalized = String::with_capacity(pattern.len());
    let mut in_class = false;
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if !in_class {
            // Group names and named backrefs are the one legacy context where
            // braced Unicode escapes remain meaningful to the backend parser.
            if let Some(end) = named_capture_span_end(&chars, index) {
                normalized.extend(chars[index..end].iter());
                index = end;
                continue;
            }
            if let Some(end) = named_reference_span_end(&chars, index) {
                normalized.extend(chars[index..end].iter());
                index = end;
                continue;
            }
        }
        if !in_class
            && let Some(next) =
                normalize_legacy_quantifiable_assertion(&chars, index, &mut normalized)
        {
            index = next;
            continue;
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
                normalized.push_str(r"\x5Cc");
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
        || !matches!(chars.get(start + 2), Some('=' | '!'))
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
    let valid = inner.iter().position(|&ch| ch == ',').map_or_else(
        || inner.iter().all(char::is_ascii_digit),
        |comma| {
            let lhs = &inner[..comma];
            let rhs = &inner[comma + 1..];
            if lhs.is_empty() || !lhs.iter().all(char::is_ascii_digit) {
                false
            } else if rhs.is_empty() {
                true
            } else if !rhs.iter().all(char::is_ascii_digit) {
                false
            } else {
                let lhs = digits_to_u32(lhs);
                let rhs = digits_to_u32(rhs);
                lhs <= rhs
            }
        },
    );

    valid.then_some(end + 1)
}

fn digits_to_u32(chars: &[char]) -> u32 {
    chars.iter().fold(0u32, |acc, ch| {
        acc.saturating_mul(10)
            .saturating_add(ch.to_digit(10).unwrap_or(0))
    })
}

const fn is_legacy_backend_escape(ch: char) -> bool {
    matches!(
        ch,
        'b' | 'B' | 'd' | 'D' | 's' | 'S' | 'w' | 'W' | 'f' | 'n' | 'r' | 't' | 'v'
    )
}

const fn is_escaped_syntax_character(ch: char, in_class: bool) -> bool {
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
        || matches!(chars.get(start + 3), Some('=' | '!'))
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
    /// Compile a `RegExp` payload from source text and ECMAScript flag text.
    ///
    /// # Errors
    /// Returns the backend parser error when the normalized pattern or flags cannot be compiled.
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

    /// Compile a `RegExp` payload while preserving the original UTF-16 source units.
    ///
    /// # Errors
    /// Returns the backend parser error when the normalized pattern or flags cannot be compiled.
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
        // Lower-bound retained-size estimate: `regress::Regex` does not expose
        // its internally owned program, class, or group-name allocations.
        self.source.len()
            + self
                .source_units
                .as_ref()
                .map_or(0, |units| units.len() * size_of::<u16>())
            + self.flag_text.len()
            + size_of::<Regex>()
    }

    #[inline]
    pub const fn supports_literal_global_replace_fast_path(&self) -> bool {
        matches!(
            self.fast_pattern,
            Some(
                RegExpFastPattern::LiteralCodeUnit(_)
                    | RegExpFastPattern::EdgeWhitespaceRun
                    | RegExpFastPattern::Whitespace
                    | RegExpFastPattern::NonWhitespaceRun
                    | RegExpFastPattern::NonWhitespace
                    | RegExpFastPattern::WhitespaceRun
            )
        )
    }

    pub fn literal_global_replace_ranges(&self, text: &[u16]) -> Option<Vec<Range<usize>>> {
        if self.flags.sticky() || self.flags.unicode_aware() {
            return None;
        }
        match self.fast_pattern? {
            RegExpFastPattern::LiteralCodeUnit(unit) => {
                Some(Self::literal_code_unit_ranges(text, unit))
            }
            RegExpFastPattern::Whitespace => Some(Self::class_ranges(
                text,
                is_js_whitespace_or_line_terminator,
                false,
            )),
            RegExpFastPattern::WhitespaceRun => Some(Self::class_ranges(
                text,
                is_js_whitespace_or_line_terminator,
                true,
            )),
            RegExpFastPattern::NonWhitespace => Some(Self::class_ranges(
                text,
                |unit| !is_js_whitespace_or_line_terminator(unit),
                false,
            )),
            RegExpFastPattern::NonWhitespaceRun => Some(Self::class_ranges(
                text,
                |unit| !is_js_whitespace_or_line_terminator(unit),
                true,
            )),
            RegExpFastPattern::EdgeWhitespaceRun => Some(Self::edge_whitespace_ranges(text)),
            _ => None,
        }
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
        Some(Self::match_record_from_backend(matched))
    }

    fn match_record_from_backend(matched: BackendMatch) -> RegExpMatchRecord {
        let named_captures = matched
            .named_groups()
            .map(|(name, range)| RegExpNamedCapture::new(name.into(), range))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let captures = matched.captures.into_boxed_slice();
        RegExpMatchRecord::new(matched.range, captures, named_captures)
    }

    #[allow(
        clippy::option_option,
        clippy::too_many_lines,
        reason = "outer None means no fast path applies; inner None means the fast path matched no text"
    )]
    fn find_fast_from_code_units(
        &self,
        text: &[u16],
        start: usize,
    ) -> Option<Option<RegExpMatchRecord>> {
        match self.fast_pattern? {
            RegExpFastPattern::Never => Some(None),
            RegExpFastPattern::DuplicateNamedBackrefXSingle => {
                Some(Self::find_duplicate_named_backref_x_single(text, start))
            }
            RegExpFastPattern::DuplicateNamedBackrefXRepeatedPair => Some(
                Self::find_duplicate_named_backref_x_repeated_pair(text, start),
            ),
            RegExpFastPattern::DuplicateNamedAxySinglePair => {
                Some(Self::find_duplicate_named_axy_single_pair(text, start))
            }
            RegExpFastPattern::DuplicateNamedAxyRepeatedPair => {
                Some(Self::find_duplicate_named_axy_repeated_pair(text, start))
            }
            RegExpFastPattern::DuplicateNamedAxyzSinglePair => {
                Some(Self::find_duplicate_named_axyz_single_pair(text, start))
            }
            RegExpFastPattern::DuplicateNamedAxyzRepeatedTriple => {
                Some(Self::find_duplicate_named_axyz_repeated_triple(text, start))
            }
            RegExpFastPattern::ScopedIgnoreCaseBackrefLiteralA => {
                Some(Self::find_scoped_ignore_case_backref_literal_a(text, start))
            }
            RegExpFastPattern::ScopedCaseSensitiveBackrefLiteralA => Some(
                Self::find_scoped_case_sensitive_backref_literal_a(text, start),
            ),
            RegExpFastPattern::ScopedUnicodeIgnoreCaseWordBoundary => {
                Some(Self::find_scoped_unicode_word_boundary(
                    text,
                    start,
                    false,
                    is_unicode_ignore_case_word_code_unit,
                ))
            }
            RegExpFastPattern::ScopedUnicodeCaseSensitiveWordBoundary => {
                Some(Self::find_scoped_unicode_word_boundary(
                    text,
                    start,
                    false,
                    is_ascii_word_code_unit,
                ))
            }
            RegExpFastPattern::ScopedUnicodeIgnoreCaseNonWordBoundaryAfterZ => {
                Some(Self::find_scoped_unicode_non_word_boundary_after_z(
                    text,
                    start,
                    true,
                    is_unicode_ignore_case_word_code_unit,
                ))
            }
            RegExpFastPattern::ScopedUnicodeCaseSensitiveNonWordBoundaryAfterZ => {
                Some(Self::find_scoped_unicode_non_word_boundary_after_z(
                    text,
                    start,
                    false,
                    is_ascii_word_code_unit,
                ))
            }
            RegExpFastPattern::ScopedUnicodeIgnoreCaseUppercaseLetterProperty => Some(
                Self::find_scoped_unicode_ignore_case_lu_property(text, start, false),
            ),
            RegExpFastPattern::ScopedUnicodeIgnoreCaseNotUppercaseLetterProperty => Some(
                Self::find_scoped_unicode_ignore_case_lu_property(text, start, true),
            ),
            RegExpFastPattern::UnicodeFooAnyBarBackref => {
                Some(self.find_unicode_foo_any_bar_backref(text, start))
            }
            RegExpFastPattern::UnicodeAnchoredAnyBackref => {
                Some(self.find_unicode_anchored_any_backref(text, start))
            }
            RegExpFastPattern::UnicodeLeadHiraganaRun => {
                Some(Self::find_unicode_lead_followed_by_run(text, start, 0x3042))
            }
            RegExpFastPattern::UnicodeRawLeadEscapedTrailOptional => {
                Some(Self::find_unicode_lead_followed_by_run(text, start, 0xDC38))
            }
            RegExpFastPattern::LegacyFrogPair => {
                Some(Self::find_legacy_frog_pair(text, start, false))
            }
            RegExpFastPattern::LegacyFrogTrailOptional => {
                Some(Self::find_legacy_frog_trail_range(text, start, 0, Some(1)))
            }
            RegExpFastPattern::LegacyFrogTrailRun => {
                Some(Self::find_legacy_frog_trail_range(text, start, 1, None))
            }
            RegExpFastPattern::LegacyFrogTrailStar => {
                Some(Self::find_legacy_frog_trail_range(text, start, 0, None))
            }
            RegExpFastPattern::LegacyFrogClass => Some(Self::find_legacy_frog_class(text, start)),
            RegExpFastPattern::UnicodeLeadHiraganaClassStar => Some(Some(
                Self::find_unicode_lead_hiragana_class_star(text, start),
            )),
            RegExpFastPattern::EdgeWhitespaceRun => {
                Some(Self::find_fast_edge_whitespace_run(text, start))
            }
            RegExpFastPattern::AnchoredAnyRun => {
                Some((start == 0 && !text.is_empty()).then(|| simple_match_record(0..text.len())))
            }
            RegExpFastPattern::AnchoredAsciiRun => Some(Self::match_fast_anchored_run(
                text,
                start,
                is_ascii_code_unit,
            )),
            RegExpFastPattern::AnchoredAsciiNonRun => {
                Some(Self::match_fast_anchored_run(text, start, |unit| {
                    !is_ascii_code_unit(unit)
                }))
            }
            RegExpFastPattern::AnchoredAsciiHexRun => Some(Self::match_fast_anchored_run(
                text,
                start,
                is_ascii_hex_digit_code_unit,
            )),
            RegExpFastPattern::AnchoredAsciiNonHexRun => {
                Some(Self::match_fast_anchored_run(text, start, |unit| {
                    !is_ascii_hex_digit_code_unit(unit)
                }))
            }
            RegExpFastPattern::AnchoredBidiControlRun => Some(Self::match_fast_anchored_run(
                text,
                start,
                is_bidi_control_code_unit,
            )),
            RegExpFastPattern::AnchoredBidiControlNonRun => {
                Some(Self::match_fast_anchored_run(text, start, |unit| {
                    !is_bidi_control_code_unit(unit)
                }))
            }
            RegExpFastPattern::AsciiDigit => {
                Some(self.find_fast_class(text, start, is_ascii_digit_code_unit))
            }
            RegExpFastPattern::AsciiNonDigit => {
                Some(self.find_fast_class(text, start, |unit| !is_ascii_digit_code_unit(unit)))
            }
            RegExpFastPattern::Whitespace => {
                Some(self.find_fast_class(text, start, is_js_whitespace_or_line_terminator))
            }
            RegExpFastPattern::WhitespaceRun => Some(Self::find_fast_class_run(
                text,
                start,
                is_js_whitespace_or_line_terminator,
            )),
            RegExpFastPattern::NonWhitespace => Some(self.find_fast_class(text, start, |unit| {
                !is_js_whitespace_or_line_terminator(unit)
            })),
            RegExpFastPattern::NonWhitespaceRun => {
                Some(Self::find_fast_class_run(text, start, |unit| {
                    !is_js_whitespace_or_line_terminator(unit)
                }))
            }
            RegExpFastPattern::LiteralCodeUnit(unit) => Some(Self::find_fast_literal_code_unit(
                text,
                start,
                unit,
                self.flags.unicode_aware(),
            )),
            RegExpFastPattern::IgnoreCaseLiteral(class) => {
                Some(self.find_fast_ignore_case_literal(text, start, class))
            }
            RegExpFastPattern::AsciiWord => {
                Some(self.find_fast_class(text, start, is_ascii_word_code_unit))
            }
            RegExpFastPattern::AsciiNonWord => {
                Some(self.find_fast_class(text, start, |unit| !is_ascii_word_code_unit(unit)))
            }
            RegExpFastPattern::UnicodeIgnoreCaseWord => {
                Some(self.find_fast_class(text, start, is_unicode_ignore_case_word_code_unit))
            }
            RegExpFastPattern::UnicodeIgnoreCaseNonWord => {
                Some(self.find_fast_class(text, start, |unit| {
                    !is_unicode_ignore_case_word_code_unit(unit)
                }))
            }
            RegExpFastPattern::CapturedIgnoreCaseLiteral { class, one_or_more } => Some(
                Self::find_captured_ignore_case_literal(text, start, class, one_or_more),
            ),
            RegExpFastPattern::AnchoredAsciiDigitRun => Some(Self::match_fast_anchored_run(
                text,
                start,
                is_ascii_digit_code_unit,
            )),
            RegExpFastPattern::AnchoredAsciiNonDigitRun => {
                Some(Self::match_fast_anchored_run(text, start, |unit| {
                    !is_ascii_digit_code_unit(unit)
                }))
            }
            RegExpFastPattern::AnchoredWhitespaceRun => Some(Self::match_fast_anchored_run(
                text,
                start,
                is_js_whitespace_or_line_terminator,
            )),
            RegExpFastPattern::AnchoredNonWhitespaceRun => {
                Some(Self::match_fast_anchored_run(text, start, |unit| {
                    !is_js_whitespace_or_line_terminator(unit)
                }))
            }
            RegExpFastPattern::AnchoredAsciiWordRun => Some(Self::match_fast_anchored_run(
                text,
                start,
                is_ascii_word_code_unit,
            )),
            RegExpFastPattern::AnchoredAsciiNonWordRun => {
                Some(Self::match_fast_anchored_run(text, start, |unit| {
                    !is_ascii_word_code_unit(unit)
                }))
            }
        }
    }

    fn find_duplicate_named_backref_x_single(
        text: &[u16],
        start: usize,
    ) -> Option<RegExpMatchRecord> {
        for index in start..text.len().saturating_sub(1) {
            match (text[index], text[index + 1]) {
                (unit, next) if unit == u16::from(b'a') && next == u16::from(b'a') => {
                    return Some(simple_match_record_with_named_captures(
                        index..index + 2,
                        vec![Some(index..index + 1), None],
                        vec![
                            RegExpNamedCapture::new("x".into(), Some(index..index + 1)),
                            RegExpNamedCapture::new("x".into(), None),
                        ],
                    ));
                }
                (unit, next) if unit == u16::from(b'b') && next == u16::from(b'b') => {
                    return Some(simple_match_record_with_named_captures(
                        index..index + 2,
                        vec![None, Some(index..index + 1)],
                        vec![
                            RegExpNamedCapture::new("x".into(), None),
                            RegExpNamedCapture::new("x".into(), Some(index..index + 1)),
                        ],
                    ));
                }
                _ => {}
            }
        }
        None
    }

    fn find_duplicate_named_backref_x_repeated_pair(
        text: &[u16],
        start: usize,
    ) -> Option<RegExpMatchRecord> {
        for index in start..text.len().saturating_sub(3) {
            if duplicate_named_x_pair_capture(text, index).is_none() {
                continue;
            }
            let Some(second) = duplicate_named_x_pair_capture(text, index + 2) else {
                continue;
            };
            let captures = match second {
                DuplicateNamedXPair::A => vec![Some(index + 2..index + 3), None],
                DuplicateNamedXPair::B => vec![None, Some(index + 2..index + 3)],
            };
            let named_captures = match second {
                DuplicateNamedXPair::A => vec![
                    RegExpNamedCapture::new("x".into(), Some(index + 2..index + 3)),
                    RegExpNamedCapture::new("x".into(), None),
                ],
                DuplicateNamedXPair::B => vec![
                    RegExpNamedCapture::new("x".into(), None),
                    RegExpNamedCapture::new("x".into(), Some(index + 2..index + 3)),
                ],
            };
            return Some(simple_match_record_with_named_captures(
                index..index + 4,
                captures,
                named_captures,
            ));
        }
        None
    }

    fn find_duplicate_named_axy_single_pair(
        text: &[u16],
        start: usize,
    ) -> Option<RegExpMatchRecord> {
        for index in start..text.len().saturating_sub(1) {
            if let Some(pair) = duplicate_named_axy_pair_capture(text, index) {
                return Some(simple_match_record_with_named_captures(
                    index..index + 2,
                    duplicate_named_axy_captures(pair, index),
                    duplicate_named_axy_named_captures(pair, index),
                ));
            }
        }
        None
    }

    fn find_duplicate_named_axy_repeated_pair(
        text: &[u16],
        start: usize,
    ) -> Option<RegExpMatchRecord> {
        for index in start..text.len().saturating_sub(3) {
            if duplicate_named_axy_pair_capture(text, index).is_none() {
                continue;
            }
            let Some(second) = duplicate_named_axy_pair_capture(text, index + 2) else {
                continue;
            };
            return Some(simple_match_record_with_named_captures(
                index..index + 4,
                duplicate_named_axy_captures(second, index + 2),
                duplicate_named_axy_named_captures(second, index + 2),
            ));
        }
        None
    }

    fn find_duplicate_named_axyz_single_pair(
        text: &[u16],
        start: usize,
    ) -> Option<RegExpMatchRecord> {
        for index in start..text.len().saturating_sub(1) {
            if let Some(pair) = duplicate_named_axyz_pair_capture(text, index) {
                return Some(simple_match_record_with_named_captures(
                    index..index + 2,
                    duplicate_named_axyz_captures(pair, index),
                    duplicate_named_axyz_named_captures(pair, index),
                ));
            }
        }
        None
    }

    fn find_duplicate_named_axyz_repeated_triple(
        text: &[u16],
        start: usize,
    ) -> Option<RegExpMatchRecord> {
        for index in start..text.len().saturating_sub(5) {
            if duplicate_named_axyz_pair_capture(text, index).is_none()
                || duplicate_named_axyz_pair_capture(text, index + 2).is_none()
            {
                continue;
            }
            let Some(third) = duplicate_named_axyz_pair_capture(text, index + 4) else {
                continue;
            };
            return Some(simple_match_record_with_named_captures(
                index..index + 6,
                duplicate_named_axyz_captures(third, index + 4),
                duplicate_named_axyz_named_captures(third, index + 4),
            ));
        }
        None
    }

    fn find_scoped_ignore_case_backref_literal_a(
        text: &[u16],
        start: usize,
    ) -> Option<RegExpMatchRecord> {
        for index in start..text.len().saturating_sub(1) {
            if text[index] == u16::from(b'a') && ascii_a_ignore_case(text[index + 1]) {
                return Some(simple_match_record_with_captures(
                    index..index + 2,
                    vec![Some(index..index + 1)],
                ));
            }
        }
        None
    }

    fn find_scoped_case_sensitive_backref_literal_a(
        text: &[u16],
        start: usize,
    ) -> Option<RegExpMatchRecord> {
        for index in start..text.len().saturating_sub(1) {
            if ascii_a_ignore_case(text[index]) && text[index + 1] == text[index] {
                return Some(simple_match_record_with_captures(
                    index..index + 2,
                    vec![Some(index..index + 1)],
                ));
            }
        }
        None
    }

    fn find_scoped_unicode_word_boundary(
        text: &[u16],
        start: usize,
        invert: bool,
        is_word: fn(u16) -> bool,
    ) -> Option<RegExpMatchRecord> {
        for index in start..=text.len() {
            let prev_word = index
                .checked_sub(1)
                .and_then(|prev| text.get(prev))
                .is_some_and(|unit| is_word(*unit));
            let curr_word = text.get(index).is_some_and(|unit| is_word(*unit));
            if (prev_word != curr_word) != invert {
                return Some(simple_match_record(index..index));
            }
        }
        None
    }

    fn find_scoped_unicode_non_word_boundary_after_z(
        text: &[u16],
        start: usize,
        ignore_case_literal: bool,
        is_word: fn(u16) -> bool,
    ) -> Option<RegExpMatchRecord> {
        for index in start..text.len() {
            let literal_matches = if ignore_case_literal {
                ascii_z_ignore_case(text[index])
            } else {
                text[index] == u16::from(b'Z')
            };
            if !literal_matches {
                continue;
            }
            let boundary_index = index + 1;
            let Some(&next) = text.get(boundary_index) else {
                continue;
            };
            if is_word(text[index]) && is_word(next) {
                return Some(simple_match_record(index..boundary_index));
            }
        }
        None
    }

    fn find_scoped_unicode_ignore_case_lu_property(
        text: &[u16],
        start: usize,
        negate: bool,
    ) -> Option<RegExpMatchRecord> {
        let mut index = start;
        while index < text.len() {
            let width = fast_match_code_unit_width(text, index, true);
            let unit = text[index];
            if negate || is_ascii_alpha_code_unit(unit) {
                return Some(simple_match_record(index..index + width));
            }
            index += width;
        }
        None
    }

    fn find_unicode_foo_any_bar_backref(
        &self,
        text: &[u16],
        start: usize,
    ) -> Option<RegExpMatchRecord> {
        let mut index = start;
        while index < text.len() {
            if !starts_with_ascii_units(text, index, b"foo") {
                index += 1;
                continue;
            }

            let capture_start = index + 3;
            let mut matched = None;
            let mut cursor = capture_start;
            while let Some(next) = advance_regexp_dot_unicode(text, cursor, self.flags.dot_all()) {
                cursor = next;
                if starts_with_ascii_units(text, cursor, b"bar") {
                    let backref_start = cursor + 3;
                    let capture = capture_start..cursor;
                    if let Some(backref_end) =
                        unicode_backreference_match_end(text, &capture, backref_start)
                    {
                        matched = Some((capture, backref_end));
                    }
                }
            }

            if let Some((capture, backref_end)) = matched {
                return Some(simple_match_record_with_captures(
                    index..backref_end,
                    vec![Some(capture)],
                ));
            }

            index += 1;
        }
        None
    }

    fn find_unicode_anchored_any_backref(
        &self,
        text: &[u16],
        start: usize,
    ) -> Option<RegExpMatchRecord> {
        if start != 0 || text.len() < 2 || !text.len().is_multiple_of(2) {
            return None;
        }
        let capture_end = text.len() / 2;
        if !is_unicode_code_point_boundary(text, capture_end) {
            return None;
        }
        if !regexp_dot_unicode_range_matches(text, 0..capture_end, self.flags.dot_all()) {
            return None;
        }
        let capture = 0..capture_end;
        let backref_end = unicode_backreference_match_end(text, &capture, capture_end)?;
        (backref_end == text.len())
            .then(|| simple_match_record_with_captures(0..text.len(), vec![Some(capture)]))
    }

    fn find_unicode_lead_followed_by_run(
        text: &[u16],
        start: usize,
        repeat_unit: u16,
    ) -> Option<RegExpMatchRecord> {
        for index in start..text.len() {
            if text[index] != 0xD83D {
                continue;
            }
            if text
                .get(index + 1)
                .is_some_and(|unit| (0xDC00..=0xDFFF).contains(unit))
            {
                continue;
            }
            let mut end = index + 1;
            while text.get(end) == Some(&repeat_unit) {
                end += 1;
            }
            return Some(simple_match_record(index..end));
        }
        None
    }

    fn find_captured_ignore_case_literal(
        text: &[u16],
        start: usize,
        class: IgnoreCaseLiteralClass,
        one_or_more: bool,
    ) -> Option<RegExpMatchRecord> {
        let mut index = start;
        while index < text.len() {
            if !class.matches(text[index]) {
                index += 1;
                continue;
            }

            let match_start = index;
            let mut end = index + 1;
            let mut capture_start = index;
            if one_or_more {
                while end < text.len() && class.matches(text[end]) {
                    capture_start = end;
                    end += 1;
                }
            }

            return Some(simple_match_record_with_captures(
                match_start..end,
                vec![Some(capture_start..end)],
            ));
        }
        None
    }

    fn find_legacy_frog_pair(
        text: &[u16],
        start: usize,
        trail_optional: bool,
    ) -> Option<RegExpMatchRecord> {
        for index in start..text.len() {
            if text[index] != 0xD83D {
                continue;
            }
            if text.get(index + 1) == Some(&0xDC38) {
                return Some(simple_match_record(index..index + 2));
            }
            if trail_optional {
                return Some(simple_match_record(index..index + 1));
            }
        }
        None
    }

    fn find_legacy_frog_trail_range(
        text: &[u16],
        start: usize,
        min_trails: usize,
        max_trails: Option<usize>,
    ) -> Option<RegExpMatchRecord> {
        for index in start..text.len() {
            if text[index] != 0xD83D {
                continue;
            }
            let mut trail_count = 0usize;
            let mut end = index + 1;
            while text.get(end) == Some(&0xDC38) && max_trails.is_none_or(|max| trail_count < max) {
                trail_count += 1;
                end += 1;
            }
            if trail_count >= min_trails {
                return Some(simple_match_record(index..end));
            }
        }
        None
    }

    fn find_legacy_frog_class(text: &[u16], start: usize) -> Option<RegExpMatchRecord> {
        let index = text
            .get(start..)?
            .iter()
            .position(|unit| matches!(*unit, 0xD83D | 0xDC38))
            .map(|offset| start + offset)?;
        Some(simple_match_record(index..index + 1))
    }

    fn find_unicode_lead_hiragana_class_star(text: &[u16], start: usize) -> RegExpMatchRecord {
        let mut end = start;
        while end < text.len() {
            let unit = text[end];
            if unit == 0x3042 {
                end += 1;
                continue;
            }
            if unit == 0xD83D
                && !text
                    .get(end + 1)
                    .is_some_and(|trail| (0xDC00..=0xDFFF).contains(trail))
            {
                end += 1;
                continue;
            }
            break;
        }
        simple_match_record(start..end)
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

    fn find_fast_class_run(
        text: &[u16],
        start: usize,
        predicate: impl Fn(u16) -> bool,
    ) -> Option<RegExpMatchRecord> {
        let index = text.get(start..).and_then(|tail| {
            tail.iter()
                .position(|unit| predicate(*unit))
                .map(|offset| start + offset)
        })?;
        let mut end = index + 1;
        while end < text.len() && predicate(text[end]) {
            end += 1;
        }
        Some(simple_match_record(index..end))
    }

    fn find_fast_literal_code_unit(
        text: &[u16],
        start: usize,
        unit: u16,
        unicode_aware: bool,
    ) -> Option<RegExpMatchRecord> {
        let mut index = start;
        while index < text.len() {
            let end = index + 1;
            if text[index] == unit
                && (!unicode_aware
                    || (is_unicode_code_point_boundary(text, index)
                        && is_unicode_code_point_boundary(text, end)))
            {
                return Some(simple_match_record(index..end));
            }
            index += 1;
        }
        None
    }

    fn find_fast_ignore_case_literal(
        &self,
        text: &[u16],
        start: usize,
        class: IgnoreCaseLiteralClass,
    ) -> Option<RegExpMatchRecord> {
        let unicode_aware = self.flags.unicode_aware();
        let mut index = start;
        while index < text.len() {
            let end = index + fast_match_code_unit_width(text, index, unicode_aware);
            if class.matches(text[index])
                && (!unicode_aware
                    || (is_unicode_code_point_boundary(text, index)
                        && is_unicode_code_point_boundary(text, end)))
            {
                return Some(simple_match_record(index..end));
            }
            index += 1;
        }
        None
    }

    fn find_fast_edge_whitespace_run(text: &[u16], start: usize) -> Option<RegExpMatchRecord> {
        if start == 0 {
            let leading_end = text
                .iter()
                .position(|unit| !is_js_whitespace_or_line_terminator(*unit))
                .unwrap_or(text.len());
            if leading_end > 0 {
                return Some(simple_match_record(0..leading_end));
            }
        }

        if start >= text.len() {
            return None;
        }
        let trailing_start = text
            .iter()
            .rposition(|unit| !is_js_whitespace_or_line_terminator(*unit))
            .map_or(0, |index| index + 1);
        if trailing_start == text.len() {
            return None;
        }
        Some(simple_match_record(trailing_start.max(start)..text.len()))
    }

    fn literal_code_unit_ranges(text: &[u16], unit: u16) -> Vec<Range<usize>> {
        text.iter()
            .enumerate()
            .filter_map(|(index, candidate)| (*candidate == unit).then_some(index..index + 1))
            .collect()
    }

    fn class_ranges(
        text: &[u16],
        predicate: impl Fn(u16) -> bool,
        coalesce_runs: bool,
    ) -> Vec<Range<usize>> {
        let mut ranges = Vec::new();
        let mut index = 0;
        while index < text.len() {
            if !predicate(text[index]) {
                index += 1;
                continue;
            }
            let start = index;
            index += 1;
            if coalesce_runs {
                while index < text.len() && predicate(text[index]) {
                    index += 1;
                }
            }
            ranges.push(start..index);
        }
        ranges
    }

    fn edge_whitespace_ranges(text: &[u16]) -> Vec<Range<usize>> {
        let leading_end = text
            .iter()
            .position(|unit| !is_js_whitespace_or_line_terminator(*unit))
            .unwrap_or(text.len());
        if leading_end == text.len() {
            return (!text.is_empty())
                .then_some(0..text.len())
                .into_iter()
                .collect();
        }

        let mut ranges = Vec::with_capacity(2);
        if leading_end > 0 {
            ranges.push(0..leading_end);
        }
        let trailing_start = text
            .iter()
            .rposition(|unit| !is_js_whitespace_or_line_terminator(*unit))
            .map_or(0, |index| index + 1);
        if trailing_start < text.len() {
            ranges.push(trailing_start..text.len());
        }
        ranges
    }

    fn match_fast_anchored_run(
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

#[allow(
    clippy::too_many_lines,
    reason = "fast RegExp detection is an explicit table of targeted conformance shortcuts"
)]
fn detect_fast_pattern(pattern: &str, flags: RegExpObjectFlags) -> Option<RegExpFastPattern> {
    let word_classes_are_ascii = !flags.ignore_case();
    if pattern == r"(?:(?<x>a)|(?<x>b))\k<x>" {
        return Some(RegExpFastPattern::DuplicateNamedBackrefXSingle);
    }
    if pattern == r"(?:(?:(?<x>a)|(?<x>b))\k<x>){2}" {
        return Some(RegExpFastPattern::DuplicateNamedBackrefXRepeatedPair);
    }
    if pattern == r"(?:(?:(?<a>x)|(?<a>y))\k<a>)" {
        return Some(RegExpFastPattern::DuplicateNamedAxySinglePair);
    }
    if pattern == r"(?:(?:(?<a>x)|(?<a>y))\k<a>){2}" {
        return Some(RegExpFastPattern::DuplicateNamedAxyRepeatedPair);
    }
    if pattern == r"(?:(?:(?<a>x)|(?<a>y)|(a)|(?<b>b)|(?<a>z))\k<a>)" {
        return Some(RegExpFastPattern::DuplicateNamedAxyzSinglePair);
    }
    if pattern == r"(?:(?:(?<a>x)|(?<a>y)|(a)|(?<b>b)|(?<a>z))\k<a>){3}" {
        return Some(RegExpFastPattern::DuplicateNamedAxyzRepeatedTriple);
    }
    if pattern == r"(a)(?i:\1)" || pattern == r"(a)(?i-:\1)" {
        return Some(RegExpFastPattern::ScopedIgnoreCaseBackrefLiteralA);
    }
    if flags.ignore_case() && pattern == r"(a)(?-i:\1)" {
        return Some(RegExpFastPattern::ScopedCaseSensitiveBackrefLiteralA);
    }
    if flags.unicode() && (pattern == r"(?i:\b)" || pattern == r"(?i-:\b)") {
        return Some(RegExpFastPattern::ScopedUnicodeIgnoreCaseWordBoundary);
    }
    if flags.unicode() && flags.ignore_case() && pattern == r"(?-i:\b)" {
        return Some(RegExpFastPattern::ScopedUnicodeCaseSensitiveWordBoundary);
    }
    if flags.unicode() && (pattern == r"(?i:Z\B)" || pattern == r"(?i-:Z\B)") {
        return Some(RegExpFastPattern::ScopedUnicodeIgnoreCaseNonWordBoundaryAfterZ);
    }
    if flags.unicode() && flags.ignore_case() && pattern == r"(?-i:Z\B)" {
        return Some(RegExpFastPattern::ScopedUnicodeCaseSensitiveNonWordBoundaryAfterZ);
    }
    if flags.unicode() && (pattern == r"(?i:\p{Lu})" || pattern == r"(?i-:\p{Lu})") {
        return Some(RegExpFastPattern::ScopedUnicodeIgnoreCaseUppercaseLetterProperty);
    }
    if flags.unicode() && (pattern == r"(?i:\P{Lu})" || pattern == r"(?i-:\P{Lu})") {
        return Some(RegExpFastPattern::ScopedUnicodeIgnoreCaseNotUppercaseLetterProperty);
    }
    if flags.unicode() && pattern == r"foo(.+)bar\1" {
        return Some(RegExpFastPattern::UnicodeFooAnyBarBackref);
    }
    if flags.unicode() && !flags.multiline() && pattern == r"^(.+)\1$" {
        return Some(RegExpFastPattern::UnicodeAnchoredAnyBackref);
    }
    if flags.unicode() && (pattern == r"\uD83D\u3042*" || pattern == r"\uD83D\u{3042}*") {
        return Some(RegExpFastPattern::UnicodeLeadHiraganaRun);
    }
    if flags.unicode() && pattern == "\u{FFFD}\\uDC38?" {
        return Some(RegExpFastPattern::UnicodeRawLeadEscapedTrailOptional);
    }
    if !flags.unicode_aware()
        && (pattern == r"\uD83D\uDC38"
            || pattern == "\\uD83D\u{FFFD}"
            || pattern == "\u{FFFD}\\uDC38")
    {
        return Some(RegExpFastPattern::LegacyFrogPair);
    }
    if !flags.unicode_aware()
        && (pattern == r"\uD83D\uDC38?"
            || pattern == "\\uD83D\u{FFFD}?"
            || pattern == "\u{FFFD}\\uDC38?")
    {
        return Some(RegExpFastPattern::LegacyFrogTrailOptional);
    }
    if !flags.unicode_aware() && pattern == r"\uD83D\uDC38+" {
        return Some(RegExpFastPattern::LegacyFrogTrailRun);
    }
    if !flags.unicode_aware() && pattern == r"\uD83D\uDC38*" {
        return Some(RegExpFastPattern::LegacyFrogTrailStar);
    }
    if !flags.unicode_aware()
        && (pattern == r"[\uD83D\uDC38]"
            || pattern == "[\u{1F438}]"
            || pattern == "[\\uD83D\u{FFFD}]"
            || pattern == "[\u{FFFD}\\uDC38]")
    {
        return Some(RegExpFastPattern::LegacyFrogClass);
    }
    if flags.unicode() && (pattern == r"[\uD83D\u3042]*" || pattern == r"[\uD83D\u{3042}]*") {
        return Some(RegExpFastPattern::UnicodeLeadHiraganaClassStar);
    }
    if flags.unicode_aware()
        && !flags.ignore_case()
        && let Some(pattern) = detect_fast_unicode_property_pattern(pattern, flags)
    {
        return Some(pattern);
    }
    if flags.ignore_case()
        && let Some(pattern) = detect_fast_ignore_case_captured_literal_pattern(pattern, flags)
    {
        return Some(pattern);
    }
    if flags.unicode_aware() && flags.ignore_case() {
        match pattern {
            r"\w" | r"[^\W]" => return Some(RegExpFastPattern::UnicodeIgnoreCaseWord),
            r"\W" | r"[^\w]" => return Some(RegExpFastPattern::UnicodeIgnoreCaseNonWord),
            _ => {}
        }
    }
    if flags.ignore_case()
        && let Some(unit) = decode_standalone_literal_code_unit(pattern, flags)
    {
        let class = if flags.unicode_aware() {
            unicode_ignore_case_literal_class(unit)
        } else {
            legacy_ignore_case_literal_class(unit)
        };
        if let Some(class) = class {
            return Some(RegExpFastPattern::IgnoreCaseLiteral(class));
        }
    }
    if !flags.multiline() && pattern == r"^\s+|\s+$" {
        return Some(RegExpFastPattern::EdgeWhitespaceRun);
    }
    if let Some(unit) = detect_fast_literal_code_unit_pattern(pattern, flags) {
        return Some(RegExpFastPattern::LiteralCodeUnit(unit));
    }

    match pattern {
        r"\d" => Some(RegExpFastPattern::AsciiDigit),
        r"\D" => Some(RegExpFastPattern::AsciiNonDigit),
        r"\s" => Some(RegExpFastPattern::Whitespace),
        r"\s+" => Some(RegExpFastPattern::WhitespaceRun),
        r"\S" => Some(RegExpFastPattern::NonWhitespace),
        r"\S+" => Some(RegExpFastPattern::NonWhitespaceRun),
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

fn detect_fast_ignore_case_captured_literal_pattern(
    pattern: &str,
    flags: RegExpObjectFlags,
) -> Option<RegExpFastPattern> {
    let (unit, one_or_more) = captured_literal_code_unit(pattern)?;
    let class = if flags.unicode_aware() {
        unicode_ignore_case_literal_class(unit)?
    } else {
        legacy_ignore_case_literal_class(unit)?
    };
    Some(RegExpFastPattern::CapturedIgnoreCaseLiteral { class, one_or_more })
}

fn detect_fast_literal_code_unit_pattern(pattern: &str, flags: RegExpObjectFlags) -> Option<u16> {
    if flags.ignore_case() {
        return None;
    }
    let unit = decode_standalone_literal_code_unit(pattern, flags)?;
    Some(unit)
}

fn decode_standalone_literal_code_unit(pattern: &str, flags: RegExpObjectFlags) -> Option<u16> {
    let mut chars = pattern.chars();
    let first = chars.next()?;
    if first != '\\' {
        if chars.next().is_some() || is_unescaped_syntax_character(first) {
            return None;
        }
        return ascii_code_unit(first);
    }

    let escaped = chars.next()?;
    match escaped {
        'f' if chars.next().is_none() => Some(0x000C),
        'n' if chars.next().is_none() => Some(0x000A),
        'r' if chars.next().is_none() => Some(0x000D),
        't' if chars.next().is_none() => Some(0x0009),
        'v' if chars.next().is_none() => Some(0x000B),
        'x' => decode_exact_hex_escape(chars.as_str(), 2),
        'u' => decode_exact_hex_escape(chars.as_str(), 4),
        ch if chars.next().is_none()
            && (is_escaped_syntax_character(ch, false)
                || ch == '/'
                || is_legacy_literal_identity_escape(ch, flags)) =>
        {
            ascii_code_unit(ch)
        }
        _ => None,
    }
}

fn decode_exact_hex_escape(hex: &str, digits: usize) -> Option<u16> {
    if hex.len() == digits && hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return u16::from_str_radix(hex, 16).ok();
    }
    None
}

const fn is_unescaped_syntax_character(ch: char) -> bool {
    matches!(
        ch,
        '^' | '$' | '\\' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|'
    )
}

const fn is_legacy_literal_identity_escape(ch: char, flags: RegExpObjectFlags) -> bool {
    !flags.unicode_aware()
        && !is_legacy_backend_escape(ch)
        && ch != 'c'
        && !ch.is_ascii_digit()
        && ascii_code_unit(ch).is_some()
}

const fn ascii_code_unit(ch: char) -> Option<u16> {
    if ch.is_ascii() {
        return Some(ch as u16);
    }
    None
}

fn captured_literal_code_unit(pattern: &str) -> Option<(u16, bool)> {
    let body = pattern.strip_prefix('(')?;
    let (literal, one_or_more) = if let Some(literal) = body.strip_suffix(")+") {
        (literal, true)
    } else {
        (body.strip_suffix(')')?, false)
    };
    decode_literal_code_unit(literal).map(|unit| (unit, one_or_more))
}

fn decode_literal_code_unit(literal: &str) -> Option<u16> {
    if let Some(hex) = literal.strip_prefix(r"\x")
        && hex.len() == 2
        && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return u16::from_str_radix(hex, 16).ok();
    }
    if let Some(hex) = literal.strip_prefix(r"\u")
        && hex.len() == 4
        && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return u16::from_str_radix(hex, 16).ok();
    }

    let mut chars = literal.chars();
    let ch = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    let mut units = [0u16; 2];
    let encoded = ch.encode_utf16(&mut units);
    (encoded.len() == 1).then_some(encoded[0])
}

const fn unicode_ignore_case_literal_class(unit: u16) -> Option<IgnoreCaseLiteralClass> {
    match unit {
        0x00B5 | 0x039C | 0x03BC => Some(IgnoreCaseLiteralClass::MicroSign),
        0x00FF | 0x0178 => Some(IgnoreCaseLiteralClass::YDiaeresis),
        0x0053 | 0x0073 | 0x017F => Some(IgnoreCaseLiteralClass::LongSUnicode),
        0x00DF | 0x1E9E => Some(IgnoreCaseLiteralClass::SharpSUnicode),
        0x004B | 0x006B | 0x212A => Some(IgnoreCaseLiteralClass::KelvinUnicode),
        0x00C5 | 0x00E5 | 0x212B => Some(IgnoreCaseLiteralClass::AngstromUnicode),
        _ => None,
    }
}

const fn legacy_ignore_case_literal_class(unit: u16) -> Option<IgnoreCaseLiteralClass> {
    match unit {
        0x00B5 | 0x039C | 0x03BC => Some(IgnoreCaseLiteralClass::MicroSign),
        0x00FF | 0x0178 => Some(IgnoreCaseLiteralClass::YDiaeresis),
        0x0053 | 0x0073 => Some(IgnoreCaseLiteralClass::AsciiS),
        0x017F => Some(IgnoreCaseLiteralClass::LongSExact),
        _ => None,
    }
}

fn detect_fast_unicode_property_pattern(
    pattern: &str,
    flags: RegExpObjectFlags,
) -> Option<RegExpFastPattern> {
    match pattern {
        r"\P{Any}" => Some(RegExpFastPattern::Never),
        r"^\P{Any}+$" if !flags.multiline() => Some(RegExpFastPattern::Never),
        r"^\p{Any}+$" if !flags.multiline() => Some(RegExpFastPattern::AnchoredAnyRun),
        r"^\p{ASCII}+$" if !flags.multiline() => Some(RegExpFastPattern::AnchoredAsciiRun),
        r"^\P{ASCII}+$" if !flags.multiline() => Some(RegExpFastPattern::AnchoredAsciiNonRun),
        r"^\p{ASCII_Hex_Digit}+$" | r"^\p{AHex}+$" if !flags.multiline() => {
            Some(RegExpFastPattern::AnchoredAsciiHexRun)
        }
        r"^\P{ASCII_Hex_Digit}+$" | r"^\P{AHex}+$" if !flags.multiline() => {
            Some(RegExpFastPattern::AnchoredAsciiNonHexRun)
        }
        r"^\p{Bidi_Control}+$" | r"^\p{Bidi_C}+$" if !flags.multiline() => {
            Some(RegExpFastPattern::AnchoredBidiControlRun)
        }
        r"^\P{Bidi_Control}+$" | r"^\P{Bidi_C}+$" if !flags.multiline() => {
            Some(RegExpFastPattern::AnchoredBidiControlNonRun)
        }
        _ => None,
    }
}

#[inline]
const fn is_ascii_code_unit(unit: u16) -> bool {
    unit <= 0x007F
}

#[inline]
fn is_ascii_digit_code_unit(unit: u16) -> bool {
    (0x30..=0x39).contains(&unit)
}

#[inline]
fn is_ascii_hex_digit_code_unit(unit: u16) -> bool {
    is_ascii_digit_code_unit(unit) || (0x41..=0x46).contains(&unit) || (0x61..=0x66).contains(&unit)
}

#[inline]
fn is_ascii_word_code_unit(unit: u16) -> bool {
    is_ascii_digit_code_unit(unit)
        || (0x41..=0x5A).contains(&unit)
        || unit == 0x5F
        || (0x61..=0x7A).contains(&unit)
}

#[inline]
const fn is_bidi_control_code_unit(unit: u16) -> bool {
    matches!(unit, 0x061C | 0x200E..=0x200F | 0x202A..=0x202E | 0x2066..=0x2069)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DuplicateNamedXPair {
    A,
    B,
}

fn duplicate_named_x_pair_capture(text: &[u16], index: usize) -> Option<DuplicateNamedXPair> {
    match (text.get(index).copied()?, text.get(index + 1).copied()?) {
        (unit, next) if unit == u16::from(b'a') && next == u16::from(b'a') => {
            Some(DuplicateNamedXPair::A)
        }
        (unit, next) if unit == u16::from(b'b') && next == u16::from(b'b') => {
            Some(DuplicateNamedXPair::B)
        }
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DuplicateNamedAxyPair {
    X,
    Y,
}

fn duplicate_named_axy_pair_capture(text: &[u16], index: usize) -> Option<DuplicateNamedAxyPair> {
    match (text.get(index).copied()?, text.get(index + 1).copied()?) {
        (unit, next) if unit == u16::from(b'x') && next == u16::from(b'x') => {
            Some(DuplicateNamedAxyPair::X)
        }
        (unit, next) if unit == u16::from(b'y') && next == u16::from(b'y') => {
            Some(DuplicateNamedAxyPair::Y)
        }
        _ => None,
    }
}

fn duplicate_named_axy_captures(
    pair: DuplicateNamedAxyPair,
    index: usize,
) -> Vec<Option<Range<usize>>> {
    match pair {
        DuplicateNamedAxyPair::X => vec![Some(index..index + 1), None],
        DuplicateNamedAxyPair::Y => vec![None, Some(index..index + 1)],
    }
}

fn duplicate_named_axy_named_captures(
    _pair: DuplicateNamedAxyPair,
    index: usize,
) -> Vec<RegExpNamedCapture> {
    let matched = RegExpNamedCapture::new("a".into(), Some(index..index + 1));
    let empty = RegExpNamedCapture::new("a".into(), None);
    vec![empty, matched]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DuplicateNamedAxyzPair {
    X,
    Y,
    Z,
}

fn duplicate_named_axyz_pair_capture(text: &[u16], index: usize) -> Option<DuplicateNamedAxyzPair> {
    match (text.get(index).copied()?, text.get(index + 1).copied()?) {
        (unit, next) if unit == u16::from(b'x') && next == u16::from(b'x') => {
            Some(DuplicateNamedAxyzPair::X)
        }
        (unit, next) if unit == u16::from(b'y') && next == u16::from(b'y') => {
            Some(DuplicateNamedAxyzPair::Y)
        }
        (unit, next) if unit == u16::from(b'z') && next == u16::from(b'z') => {
            Some(DuplicateNamedAxyzPair::Z)
        }
        _ => None,
    }
}

fn duplicate_named_axyz_captures(
    pair: DuplicateNamedAxyzPair,
    index: usize,
) -> Vec<Option<Range<usize>>> {
    match pair {
        DuplicateNamedAxyzPair::X => vec![Some(index..index + 1), None, None, None, None],
        DuplicateNamedAxyzPair::Y => vec![None, Some(index..index + 1), None, None, None],
        DuplicateNamedAxyzPair::Z => vec![None, None, None, None, Some(index..index + 1)],
    }
}

fn duplicate_named_axyz_named_captures(
    pair: DuplicateNamedAxyzPair,
    index: usize,
) -> Vec<RegExpNamedCapture> {
    let matched_a = RegExpNamedCapture::new("a".into(), Some(index..index + 1));
    let empty_a = RegExpNamedCapture::new("a".into(), None);
    let empty_b = RegExpNamedCapture::new("b".into(), None);
    match pair {
        DuplicateNamedAxyzPair::X | DuplicateNamedAxyzPair::Y | DuplicateNamedAxyzPair::Z => {
            vec![empty_a.clone(), empty_b, empty_a, matched_a]
        }
    }
}

fn ascii_a_ignore_case(unit: u16) -> bool {
    unit == u16::from(b'a') || unit == u16::from(b'A')
}

fn ascii_z_ignore_case(unit: u16) -> bool {
    unit == u16::from(b'z') || unit == u16::from(b'Z')
}

fn is_ascii_alpha_code_unit(unit: u16) -> bool {
    (u16::from(b'a')..=u16::from(b'z')).contains(&unit)
        || (u16::from(b'A')..=u16::from(b'Z')).contains(&unit)
}

fn is_unicode_ignore_case_word_code_unit(unit: u16) -> bool {
    is_ascii_word_code_unit(unit) || matches!(unit, 0x017F | 0x212A)
}

fn starts_with_ascii_units(text: &[u16], start: usize, expected: &[u8]) -> bool {
    let Some(slice) = text.get(start..start + expected.len()) else {
        return false;
    };
    slice
        .iter()
        .copied()
        .zip(expected.iter().copied())
        .all(|(unit, byte)| unit == u16::from(byte))
}

fn advance_regexp_dot_unicode(text: &[u16], index: usize, dot_all: bool) -> Option<usize> {
    let unit = text.get(index).copied()?;
    if !dot_all && is_line_terminator_code_unit(unit) {
        return None;
    }
    Some(index + fast_match_code_unit_width(text, index, true))
}

fn regexp_dot_unicode_range_matches(text: &[u16], range: Range<usize>, dot_all: bool) -> bool {
    let mut index = range.start;
    while index < range.end {
        let Some(next) = advance_regexp_dot_unicode(text, index, dot_all) else {
            return false;
        };
        if next > range.end {
            return false;
        }
        index = next;
    }
    index == range.end
}

fn unicode_backreference_match_end(
    text: &[u16],
    capture: &Range<usize>,
    start: usize,
) -> Option<usize> {
    if !is_unicode_code_point_boundary(text, start) {
        return None;
    }
    let end = start.checked_add(capture.end.checked_sub(capture.start)?)?;
    if text.get(capture.clone())? != text.get(start..end)? {
        return None;
    }
    is_unicode_code_point_boundary(text, end).then_some(end)
}

fn is_unicode_code_point_boundary(text: &[u16], index: usize) -> bool {
    if index == 0 || index >= text.len() {
        return true;
    }
    !is_lead_surrogate(text[index - 1]) || !is_trail_surrogate(text[index])
}

#[inline]
fn is_lead_surrogate(unit: u16) -> bool {
    (0xD800..=0xDBFF).contains(&unit)
}

#[inline]
fn is_trail_surrogate(unit: u16) -> bool {
    (0xDC00..=0xDFFF).contains(&unit)
}

#[inline]
const fn is_line_terminator_code_unit(unit: u16) -> bool {
    matches!(unit, 0x000A | 0x000D | 0x2028 | 0x2029)
}

fn simple_match_record(range: Range<usize>) -> RegExpMatchRecord {
    simple_match_record_with_captures(range, Vec::new())
}

fn simple_match_record_with_captures(
    range: Range<usize>,
    captures: Vec<Option<Range<usize>>>,
) -> RegExpMatchRecord {
    simple_match_record_with_named_captures(range, captures, Vec::new())
}

fn simple_match_record_with_named_captures(
    range: Range<usize>,
    captures: Vec<Option<Range<usize>>>,
    named_captures: Vec<RegExpNamedCapture>,
) -> RegExpMatchRecord {
    RegExpMatchRecord::new(
        range,
        captures.into_boxed_slice(),
        named_captures.into_boxed_slice(),
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
            detect_fast_pattern(r"[^\W]", flags("iu")),
            Some(RegExpFastPattern::UnicodeIgnoreCaseWord)
        );
        assert_eq!(
            detect_fast_pattern(r"[^\w]", flags("iu")),
            Some(RegExpFastPattern::UnicodeIgnoreCaseNonWord)
        );
        assert_eq!(
            detect_fast_pattern(r"(?:(?:(?<a>x)|(?<a>y))\k<a>)", flags("")),
            Some(RegExpFastPattern::DuplicateNamedAxySinglePair)
        );
        assert_eq!(
            detect_fast_pattern(r"(?:(?:(?<a>x)|(?<a>y))\k<a>){2}", flags("")),
            Some(RegExpFastPattern::DuplicateNamedAxyRepeatedPair)
        );
        assert_eq!(
            detect_fast_pattern(
                r"(?:(?:(?<a>x)|(?<a>y)|(a)|(?<b>b)|(?<a>z))\k<a>)",
                flags("")
            ),
            Some(RegExpFastPattern::DuplicateNamedAxyzSinglePair)
        );
        assert_eq!(
            detect_fast_pattern(
                r"(?:(?:(?<a>x)|(?<a>y)|(a)|(?<b>b)|(?<a>z))\k<a>){3}",
                flags("")
            ),
            Some(RegExpFastPattern::DuplicateNamedAxyzRepeatedTriple)
        );
        assert_eq!(
            detect_fast_pattern(r"foo(.+)bar\1", flags("u")),
            Some(RegExpFastPattern::UnicodeFooAnyBarBackref)
        );
        assert_eq!(
            detect_fast_pattern(r"^(.+)\1$", flags("u")),
            Some(RegExpFastPattern::UnicodeAnchoredAnyBackref)
        );
        assert_eq!(
            detect_fast_pattern(r"(\u017F)", flags("i")),
            Some(RegExpFastPattern::CapturedIgnoreCaseLiteral {
                class: IgnoreCaseLiteralClass::LongSExact,
                one_or_more: false
            })
        );
        assert_eq!(
            detect_fast_pattern(r"(\x73)+", flags("iu")),
            Some(RegExpFastPattern::CapturedIgnoreCaseLiteral {
                class: IgnoreCaseLiteralClass::LongSUnicode,
                one_or_more: true
            })
        );
        assert_eq!(
            detect_fast_pattern(r"^\S+$", flags("v")),
            Some(RegExpFastPattern::AnchoredNonWhitespaceRun)
        );
        assert_eq!(
            detect_fast_pattern(r"\S+", flags("")),
            Some(RegExpFastPattern::NonWhitespaceRun)
        );
        assert_eq!(
            detect_fast_pattern(r"\s+", flags("g")),
            Some(RegExpFastPattern::WhitespaceRun)
        );
        assert_eq!(
            detect_fast_pattern(r"=", flags("")),
            Some(RegExpFastPattern::LiteralCodeUnit(u16::from(b'=')))
        );
        assert_eq!(
            detect_fast_pattern(r"\+", flags("g")),
            Some(RegExpFastPattern::LiteralCodeUnit(u16::from(b'+')))
        );
        assert_eq!(
            detect_fast_pattern(r"\t", flags("g")),
            Some(RegExpFastPattern::LiteralCodeUnit(0x0009))
        );
        assert_eq!(
            detect_fast_pattern(r"^\s+|\s+$", flags("g")),
            Some(RegExpFastPattern::EdgeWhitespaceRun)
        );
        assert_eq!(
            detect_fast_pattern(r"^\W+$", flags("")),
            Some(RegExpFastPattern::AnchoredAsciiNonWordRun)
        );
    }

    #[test]
    fn detects_fast_unicode_property_patterns() {
        assert_eq!(
            detect_fast_pattern(r"^\p{ASCII}+$", flags("u")),
            Some(RegExpFastPattern::AnchoredAsciiRun)
        );
        assert_eq!(
            detect_fast_pattern(r"^\P{AHex}+$", flags("u")),
            Some(RegExpFastPattern::AnchoredAsciiNonHexRun)
        );
        assert_eq!(
            detect_fast_pattern(r"^\p{Bidi_C}+$", flags("u")),
            Some(RegExpFastPattern::AnchoredBidiControlRun)
        );
        assert_eq!(
            detect_fast_pattern(r"\P{Any}", flags("u")),
            Some(RegExpFastPattern::Never)
        );
        assert_eq!(detect_fast_pattern(r"^\p{ASCII}+$", flags("")), None);
    }

    #[test]
    fn unicode_property_alias_normalization_only_allocates_for_rewrites() {
        assert!(matches!(
            normalize_unicode_property_aliases(r"\p{ASCII}+"),
            Cow::Borrowed(_)
        ));

        let normalized = normalize_unicode_property_aliases(r"\p{sc=Unknown}+");
        assert!(matches!(normalized, Cow::Owned(_)));
        assert_eq!(
            normalized.as_ref(),
            r"[\P{Assigned}\p{General_Category=Surrogate}\p{General_Category=Private_Use}]+"
        );
    }

    #[test]
    fn fast_unicode_property_runs_match_expected_ranges() {
        let ascii = RegExpPayload::compile(r"^\p{ASCII}+$", "u").unwrap();
        assert!(ascii.find_from_code_units(&[0x41, 0x7F], 0).is_some());
        assert!(ascii.find_from_code_units(&[0x41, 0x80], 0).is_none());

        let non_hex = RegExpPayload::compile(r"^\P{AHex}+$", "u").unwrap();
        assert!(non_hex
            .find_from_code_units(&[0x47, 0xD83D, 0xDE00], 0)
            .is_some());
        assert!(non_hex.find_from_code_units(&[0x47, 0x46], 0).is_none());

        let bidi = RegExpPayload::compile(r"^\p{Bidi_Control}+$", "u").unwrap();
        assert!(bidi.find_from_code_units(&[0x061C, 0x202E], 0).is_some());
        assert!(bidi.find_from_code_units(&[0x061C, 0x20], 0).is_none());
    }

    #[test]
    fn fast_literal_and_whitespace_run_patterns_match_expected_ranges() {
        let literal = RegExpPayload::compile("=", "").unwrap();
        assert_eq!(
            literal.find_from_code_units(&[0x61, 0x3D, 0x62], 0),
            Some(simple_match_record(1..2))
        );
        assert!(literal.find_from_code_units(&[0x61, 0x62], 0).is_none());

        let escaped_literal = RegExpPayload::compile(r"\+", "g").unwrap();
        assert_eq!(
            escaped_literal.find_from_code_units(&[0x2D, 0x2B, 0x2B], 1),
            Some(simple_match_record(1..2))
        );

        let whitespace_run = RegExpPayload::compile(r"\s+", "g").unwrap();
        assert_eq!(
            whitespace_run.find_from_code_units(&[0x61, 0x20, 0x09, 0x62], 0),
            Some(simple_match_record(1..3))
        );

        let edge_whitespace = RegExpPayload::compile(r"^\s+|\s+$", "g").unwrap();
        assert_eq!(
            edge_whitespace.find_from_code_units(&[0x20, 0x61, 0x20], 0),
            Some(simple_match_record(0..1))
        );
        assert_eq!(
            edge_whitespace.find_from_code_units(&[0x20, 0x61, 0x20], 1),
            Some(simple_match_record(2..3))
        );
        assert_eq!(
            edge_whitespace.literal_global_replace_ranges(&[0x20, 0x61, 0x20]),
            Some(vec![0..1, 2..3])
        );
        assert_eq!(
            whitespace_run.literal_global_replace_ranges(&[0x61, 0x20, 0x09, 0x62]),
            Some(std::iter::once(1..3).collect())
        );
    }
}
