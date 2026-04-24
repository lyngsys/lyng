use regress::Regex;
use std::{mem::size_of, ops::Range};

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
}

fn normalize_backend_pattern(pattern: &str) -> String {
    [
        (r"\p{Script=Unknown}", r"\P{Assigned}"),
        (r"\p{Script=Zzzz}", r"\P{Assigned}"),
        (r"\p{sc=Unknown}", r"\P{Assigned}"),
        (r"\p{sc=Zzzz}", r"\P{Assigned}"),
        (r"\p{Script_Extensions=Unknown}", r"\P{Assigned}"),
        (r"\p{Script_Extensions=Zzzz}", r"\P{Assigned}"),
        (r"\p{scx=Unknown}", r"\P{Assigned}"),
        (r"\p{scx=Zzzz}", r"\P{Assigned}"),
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
    })
}

impl RegExpPayload {
    pub fn compile(pattern: &str, flags: &str) -> Result<Self, regress::Error> {
        let parsed_flags = RegExpObjectFlags::from_flag_text(flags);
        let backend_pattern = normalize_backend_pattern(pattern);
        let backend = Regex::with_flags(&backend_pattern, parsed_flags.compile_flags())?;
        Ok(Self {
            source: pattern.into(),
            source_units: None,
            flags: parsed_flags,
            flag_text: parsed_flags.ordered_flag_text().into_boxed_str(),
            backend,
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
}
