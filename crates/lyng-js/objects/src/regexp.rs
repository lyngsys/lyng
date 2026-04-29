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
    AnchoredAsciiNonDigitRun,
}

fn normalize_backend_pattern(pattern: &str, flags: RegExpObjectFlags) -> String {
    const UNKNOWN_SCRIPT_SET: &str =
        r"[\P{Assigned}\p{General_Category=Surrogate}\p{General_Category=Private_Use}]";

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
    .fold(pattern.to_owned(), |current, (from, to)| {
        current.replace(from, to)
    });

    if flags.unicode_aware() {
        normalized
    } else {
        expand_astral_source_for_ucs2(&normalized)
    }
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
                let matched = text.get(start..).and_then(|tail| {
                    tail.iter()
                        .position(|unit| is_ascii_digit_code_unit(*unit))
                        .map(|offset| start + offset)
                });
                Some(matched.map(|index| simple_match_record(index..index + 1)))
            }
            RegExpFastPattern::AnchoredAsciiNonDigitRun => {
                if start != 0
                    || text.is_empty()
                    || text.iter().any(|unit| is_ascii_digit_code_unit(*unit))
                {
                    return Some(None);
                }
                Some(Some(simple_match_record(0..text.len())))
            }
        }
    }
}

fn detect_fast_pattern(pattern: &str, flags: RegExpObjectFlags) -> Option<RegExpFastPattern> {
    match pattern {
        r"\d" => Some(RegExpFastPattern::AsciiDigit),
        r"^\D+$" if !flags.multiline() => Some(RegExpFastPattern::AnchoredAsciiNonDigitRun),
        _ => None,
    }
}

#[inline]
fn is_ascii_digit_code_unit(unit: u16) -> bool {
    (0x30..=0x39).contains(&unit)
}

fn simple_match_record(range: Range<usize>) -> RegExpMatchRecord {
    RegExpMatchRecord::new(
        range,
        Vec::<Option<Range<usize>>>::new().into_boxed_slice(),
        Vec::<RegExpNamedCapture>::new().into_boxed_slice(),
    )
}
