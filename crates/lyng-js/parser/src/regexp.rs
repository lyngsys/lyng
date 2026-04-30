use std::collections::{HashMap, HashSet};

use crate::regexp_tables::is_valid_unicode_property_escape;

#[derive(Clone, Copy, Eq, PartialEq)]
enum AtomKind {
    None,
    Atom,
    LookaheadAssertion,
    LookbehindAssertion,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum GroupKind {
    Atom,
    LookaheadAssertion,
    LookbehindAssertion,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ClassAtomKind {
    Single,
    Set,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AlternativePath {
    segments: Vec<(usize, usize)>,
}

impl AlternativePath {
    fn conflicts_with(&self, other: &Self) -> bool {
        let common = self.segments.len().min(other.segments.len());
        self.segments[..common] == other.segments[..common]
    }
}

#[derive(Clone, Copy)]
struct RegExpFlags {
    unicode: bool,
    unicode_sets: bool,
}

pub fn validate_regexp_literal(pattern: &str, flags: &str) -> Result<(), &'static str> {
    let flags = validate_flags(flags)?;
    let total_captures = count_capturing_groups(pattern);
    validate_pattern(pattern, flags, total_captures, true)
}

pub fn validate_regexp_constructor_pattern(pattern: &str, flags: &str) -> Result<(), &'static str> {
    let flags = validate_flags(flags)?;
    let total_captures = count_capturing_groups(pattern);
    validate_pattern(pattern, flags, total_captures, false)
}

fn validate_flags(flags: &str) -> Result<RegExpFlags, &'static str> {
    let mut seen = HashSet::new();
    let mut saw_unicode_flag = false;
    let mut saw_unicode_sets_flag = false;
    let mut parsed = RegExpFlags {
        unicode: false,
        unicode_sets: false,
    };

    for ch in flags.chars() {
        if !matches!(ch, 'd' | 'g' | 'i' | 'm' | 's' | 'u' | 'v' | 'y') {
            return Err("invalid regular expression flags");
        }
        if !seen.insert(ch) {
            return Err("duplicate regular expression flags");
        }
        if ch == 'u' {
            saw_unicode_flag = true;
            parsed.unicode = true;
        } else if ch == 'v' {
            saw_unicode_sets_flag = true;
            parsed.unicode = true;
            parsed.unicode_sets = true;
        }
    }
    if saw_unicode_flag && saw_unicode_sets_flag {
        return Err("invalid regular expression flags");
    }

    Ok(parsed)
}

fn count_capturing_groups(pattern: &str) -> usize {
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    let mut count = 0;
    let mut in_class = false;

    while i < chars.len() {
        match chars[i] {
            '\\' => i += 2,
            '[' => {
                in_class = true;
                i += 1;
            }
            ']' if in_class => {
                in_class = false;
                i += 1;
            }
            '(' if !in_class => {
                if chars.get(i + 1) != Some(&'?') {
                    count += 1;
                    i += 1;
                    continue;
                }

                match chars.get(i + 2) {
                    Some(':') | Some('=') | Some('!') => i += 1,
                    Some('<') => match chars.get(i + 3) {
                        Some('=') | Some('!') => i += 1,
                        _ => {
                            count += 1;
                            i += 1;
                        }
                    },
                    _ => i += 1,
                }
                i += 1;
            }
            _ => i += 1,
        }
    }

    count
}

fn validate_pattern(
    pattern: &str,
    flags: RegExpFlags,
    total_captures: usize,
    allow_annex_b_invalid_braced_quantifier: bool,
) -> Result<(), &'static str> {
    let chars: Vec<char> = pattern.chars().collect();
    validate_duplicate_named_groups(&chars)?;

    let mut i = 0;
    let mut last_atom = AtomKind::None;
    let mut groups = Vec::new();
    let mut named_groups = HashSet::new();
    let all_named_groups = collect_named_group_names(pattern);
    let mut named_references = Vec::new();

    while i < chars.len() {
        let ch = chars[i];

        if is_line_terminator(ch) {
            return Err("invalid regular expression pattern");
        }

        match ch {
            '\\' => {
                let (next, kind) = parse_escape(
                    &chars,
                    i,
                    flags,
                    false,
                    false,
                    total_captures,
                    &all_named_groups,
                    &mut named_references,
                )?;
                i = next;
                last_atom = match kind {
                    ClassAtomKind::Single | ClassAtomKind::Set => AtomKind::Atom,
                };
            }
            '[' => {
                i = parse_character_class(&chars, i, flags, total_captures, &all_named_groups)?;
                last_atom = AtomKind::Atom;
            }
            '(' => {
                let (next, kind) = parse_group_start(&chars, i, &mut named_groups)?;
                groups.push(kind);
                i = next;
                last_atom = AtomKind::None;
            }
            ')' => {
                let kind = groups.pop().unwrap_or(GroupKind::Atom);
                last_atom = match kind {
                    GroupKind::Atom => AtomKind::Atom,
                    GroupKind::LookaheadAssertion => AtomKind::LookaheadAssertion,
                    GroupKind::LookbehindAssertion => AtomKind::LookbehindAssertion,
                };
                i += 1;
            }
            '|' => {
                last_atom = AtomKind::None;
                i += 1;
            }
            '*' | '+' | '?' => {
                if last_atom != AtomKind::Atom
                    && !(last_atom == AtomKind::LookaheadAssertion && !flags.unicode)
                {
                    return Err("invalid regular expression pattern");
                }
                i += 1;
                if chars.get(i) == Some(&'?') {
                    i += 1;
                }
                last_atom = AtomKind::None;
            }
            '{' => {
                let quantifier = parse_braced_quantifier(&chars, i);
                if let Some((next, valid)) = quantifier {
                    if !valid {
                        if flags.unicode || !allow_annex_b_invalid_braced_quantifier {
                            return Err("invalid regular expression pattern");
                        }
                        last_atom = AtomKind::Atom;
                        i += 1;
                        continue;
                    }
                    if last_atom != AtomKind::Atom
                        && !(last_atom == AtomKind::LookaheadAssertion && !flags.unicode)
                    {
                        return Err("invalid regular expression pattern");
                    }
                    i = next;
                    if chars.get(i) == Some(&'?') {
                        i += 1;
                    }
                    last_atom = AtomKind::None;
                } else if flags.unicode {
                    return Err("invalid regular expression pattern");
                } else {
                    last_atom = AtomKind::Atom;
                    i += 1;
                }
            }
            _ => {
                last_atom = AtomKind::Atom;
                i += 1;
            }
        }
    }

    if named_references
        .iter()
        .any(|name| !named_groups.contains(name))
    {
        return Err("invalid regular expression pattern");
    }

    Ok(())
}

fn parse_group_start(
    chars: &[char],
    start: usize,
    named_groups: &mut HashSet<String>,
) -> Result<(usize, GroupKind), &'static str> {
    if chars.get(start + 1) != Some(&'?') {
        return Ok((start + 1, GroupKind::Atom));
    }

    match chars.get(start + 2) {
        Some(':') => Ok((start + 3, GroupKind::Atom)),
        Some('=') | Some('!') => Ok((start + 3, GroupKind::LookaheadAssertion)),
        Some('i' | 'm' | 's' | '-') => parse_modifier_group_start(chars, start),
        Some('<') => match chars.get(start + 3) {
            Some('=') | Some('!') => Ok((start + 4, GroupKind::LookbehindAssertion)),
            _ => {
                let mut end = start + 3;
                while end < chars.len() && chars[end] != '>' {
                    if is_line_terminator(chars[end]) {
                        return Err("invalid regular expression pattern");
                    }
                    end += 1;
                }

                if end >= chars.len() {
                    return Err("invalid regular expression pattern");
                }

                let Some(name) = parse_group_name_contents(&chars[start + 3..end]) else {
                    return Err("invalid regular expression pattern");
                };
                named_groups.insert(name);

                Ok((end + 1, GroupKind::Atom))
            }
        },
        _ => Ok((start + 1, GroupKind::Atom)),
    }
}

fn parse_modifier_group_start(
    chars: &[char],
    start: usize,
) -> Result<(usize, GroupKind), &'static str> {
    let mut index = start + 2;
    let mut seen_hyphen = false;
    let mut saw_flag = false;
    let mut ignore_case = None;
    let mut multiline = None;
    let mut dot_all = None;

    while let Some(&ch) = chars.get(index) {
        match ch {
            'i' | 'm' | 's' => {
                let enabled = !seen_hyphen;
                let slot = match ch {
                    'i' => &mut ignore_case,
                    'm' => &mut multiline,
                    's' => &mut dot_all,
                    _ => unreachable!(),
                };
                if slot.replace(enabled).is_some() {
                    return Err("invalid regular expression pattern");
                }
                saw_flag = true;
            }
            '-' => {
                if seen_hyphen {
                    return Err("invalid regular expression pattern");
                }
                seen_hyphen = true;
            }
            ':' => {
                if !saw_flag {
                    return Err("invalid regular expression pattern");
                }
                return Ok((index + 1, GroupKind::Atom));
            }
            _ => return Err("invalid regular expression pattern"),
        }

        index += 1;
    }

    Err("invalid regular expression pattern")
}

fn parse_braced_quantifier(chars: &[char], start: usize) -> Option<(usize, bool)> {
    let mut end = start + 1;
    while end < chars.len() && chars[end] != '}' {
        if is_line_terminator(chars[end]) {
            return Some((end, false));
        }
        end += 1;
    }

    if end >= chars.len() {
        return None;
    }

    let inner: String = chars[start + 1..end].iter().collect();
    if inner.is_empty() {
        return None;
    }

    let valid = if let Some((lhs, rhs)) = inner.split_once(',') {
        if lhs.is_empty() || !lhs.chars().all(|ch| ch.is_ascii_digit()) {
            false
        } else if rhs.is_empty() {
            true
        } else if !rhs.chars().all(|ch| ch.is_ascii_digit()) {
            false
        } else {
            lhs.parse::<u32>().ok() <= rhs.parse::<u32>().ok()
        }
    } else {
        inner.chars().all(|ch| ch.is_ascii_digit())
    };

    Some((end + 1, valid))
}

fn parse_character_class(
    chars: &[char],
    start: usize,
    flags: RegExpFlags,
    total_captures: usize,
    all_named_groups: &HashSet<String>,
) -> Result<usize, &'static str> {
    let mut i = start + 1;
    let mut prev_atom = None;
    let mut pending_range_left = None;
    let mut pending_operator = false;
    let mut first = true;
    let mut class_complemented = false;

    if chars.get(i) == Some(&'^') {
        class_complemented = true;
        i += 1;
        first = false;
    }

    while i < chars.len() {
        if chars[i] == ']' && (!first || !has_unescaped_class_closer(chars, i + 1)) {
            if pending_operator {
                return Err("invalid regular expression pattern");
            }
            return Ok(i + 1);
        }

        if flags.unicode_sets && is_class_set_operator(chars, i) {
            if prev_atom.is_none() || pending_range_left.is_some() {
                return Err("invalid regular expression pattern");
            }
            prev_atom = None;
            pending_operator = true;
            i += 2;
            first = false;
            continue;
        }

        if chars[i] == '-' && prev_atom.is_some() && chars.get(i + 1) != Some(&']') {
            pending_range_left = prev_atom.take();
            i += 1;
            first = false;
            continue;
        }

        let (next, atom_kind) = if flags.unicode_sets && chars[i] == '[' {
            (
                parse_character_class(chars, i, flags, total_captures, all_named_groups)?,
                ClassAtomKind::Set,
            )
        } else if chars[i] == '\\' {
            let mut named_references = Vec::new();
            parse_escape(
                chars,
                i,
                flags,
                true,
                class_complemented,
                total_captures,
                all_named_groups,
                &mut named_references,
            )?
        } else {
            if is_line_terminator(chars[i]) {
                return Err("invalid regular expression pattern");
            }
            if flags.unicode_sets && is_unescaped_class_set_reserved_syntax(chars, i) {
                return Err("invalid regular expression pattern");
            }
            (i + 1, ClassAtomKind::Single)
        };

        if let Some(left) = pending_range_left.take() {
            if flags.unicode
                && (left != ClassAtomKind::Single || atom_kind != ClassAtomKind::Single)
            {
                return Err("invalid regular expression pattern");
            }
        }

        prev_atom = Some(atom_kind);
        pending_operator = false;
        i = next;
        first = false;
    }

    Err("invalid regular expression pattern")
}

fn is_class_set_operator(chars: &[char], index: usize) -> bool {
    matches!(
        (chars.get(index), chars.get(index + 1)),
        (Some('&'), Some('&')) | (Some('-'), Some('-'))
    )
}

fn is_unescaped_class_set_reserved_syntax(chars: &[char], index: usize) -> bool {
    let Some(&ch) = chars.get(index) else {
        return false;
    };

    if is_class_set_syntax_character(ch) || is_class_set_reserved_punctuator(ch) {
        return true;
    }

    chars
        .get(index + 1)
        .is_some_and(|&next| next == ch && is_class_set_reserved_double_punctuator(ch))
}

fn is_class_set_syntax_character(ch: char) -> bool {
    matches!(ch, '(' | ')' | '[' | '{' | '}' | '/' | '-' | '|')
}

fn is_class_set_reserved_punctuator(ch: char) -> bool {
    matches!(
        ch,
        '&' | '!' | '#' | '%' | ',' | ':' | ';' | '<' | '=' | '>' | '@' | '`' | '~'
    )
}

fn is_class_set_reserved_double_punctuator(ch: char) -> bool {
    matches!(
        ch,
        '!' | '#'
            | '$'
            | '%'
            | '&'
            | '*'
            | '+'
            | ','
            | '.'
            | ':'
            | ';'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '^'
            | '`'
            | '~'
    )
}

fn parse_escape(
    chars: &[char],
    start: usize,
    flags: RegExpFlags,
    in_class: bool,
    class_complemented: bool,
    total_captures: usize,
    all_named_groups: &HashSet<String>,
    named_references: &mut Vec<String>,
) -> Result<(usize, ClassAtomKind), &'static str> {
    let Some(&ch) = chars.get(start + 1) else {
        return Err("invalid regular expression pattern");
    };

    if is_line_terminator(ch) {
        return Err("invalid regular expression pattern");
    }

    match ch {
        'b' | 'B' => {
            if in_class {
                Ok((start + 2, ClassAtomKind::Single))
            } else {
                Ok((start + 2, ClassAtomKind::Set))
            }
        }
        'd' | 'D' | 's' | 'S' | 'w' | 'W' => Ok((start + 2, ClassAtomKind::Set)),
        'f' | 'n' | 'r' | 't' | 'v' => Ok((start + 2, ClassAtomKind::Single)),
        'q' if in_class && flags.unicode_sets => parse_class_string_disjunction(chars, start),
        'c' => {
            let Some(control) = chars.get(start + 2) else {
                if !flags.unicode {
                    return Ok((start + 2, ClassAtomKind::Single));
                }
                return Err("invalid regular expression pattern");
            };
            if !control.is_ascii_alphabetic() {
                if !flags.unicode && in_class && (control.is_ascii_digit() || *control == '_') {
                    return Ok((start + 3, ClassAtomKind::Single));
                }
                if !flags.unicode {
                    return Ok((start + 2, ClassAtomKind::Single));
                }
                return Err("invalid regular expression pattern");
            }
            Ok((start + 3, ClassAtomKind::Single))
        }
        'x' => {
            if !has_hex_digits(chars, start + 2, 2) {
                if !flags.unicode {
                    return Ok((start + 2, ClassAtomKind::Single));
                }
                return Err("invalid regular expression pattern");
            }
            Ok((start + 4, ClassAtomKind::Single))
        }
        'u' => parse_unicode_escape(chars, start, flags.unicode),
        '0' => {
            if flags.unicode && chars.get(start + 2).is_some_and(char::is_ascii_digit) {
                return Err("invalid regular expression pattern");
            }
            Ok((start + 2, ClassAtomKind::Single))
        }
        '1'..='9' => {
            let mut end = start + 2;
            while chars.get(end).is_some_and(char::is_ascii_digit) {
                end += 1;
            }
            if flags.unicode {
                let value: String = chars[start + 1..end].iter().collect();
                let value = value.parse::<usize>().ok().unwrap_or(usize::MAX);
                if value == 0 || value > total_captures {
                    return Err("invalid regular expression pattern");
                }
            }
            Ok((end, ClassAtomKind::Single))
        }
        'k' => {
            let requires_named_reference = flags.unicode || !all_named_groups.is_empty();
            match parse_named_reference(chars, start) {
                Ok(Some((end, name))) if !in_class && all_named_groups.contains(&name) => {
                    named_references.push(name);
                    Ok((end, ClassAtomKind::Single))
                }
                Ok(Some(_)) | Ok(None) | Err(_) if requires_named_reference => {
                    Err("invalid regular expression pattern")
                }
                _ => Ok((start + 2, ClassAtomKind::Single)),
            }
        }
        'p' | 'P' => {
            if !flags.unicode {
                return Ok((start + 2, ClassAtomKind::Single));
            }
            if chars.get(start + 2) != Some(&'{') {
                return Err("invalid regular expression pattern");
            }
            let mut end = start + 3;
            while end < chars.len() && chars[end] != '}' {
                if is_line_terminator(chars[end]) {
                    return Err("invalid regular expression pattern");
                }
                end += 1;
            }
            if end >= chars.len() {
                return Err("invalid regular expression pattern");
            }
            let body: String = chars[start + 3..end].iter().collect();
            if !is_valid_unicode_property_escape(
                &body,
                flags.unicode_sets,
                ch == 'P',
                in_class && class_complemented,
            ) {
                return Err("invalid regular expression pattern");
            }
            Ok((end + 1, ClassAtomKind::Set))
        }
        _ => {
            if flags.unicode && !is_unicode_identity_escape(ch, in_class) {
                return Err("invalid regular expression pattern");
            }
            Ok((start + 2, ClassAtomKind::Single))
        }
    }
}

fn parse_class_string_disjunction(
    chars: &[char],
    start: usize,
) -> Result<(usize, ClassAtomKind), &'static str> {
    if chars.get(start + 2) != Some(&'{') {
        return Err("invalid regular expression pattern");
    }

    let mut index = start + 3;
    while index < chars.len() {
        match chars[index] {
            '}' => return Ok((index + 1, ClassAtomKind::Set)),
            '\\' => {
                if chars.get(index + 1).is_none() {
                    return Err("invalid regular expression pattern");
                }
                index += 2;
            }
            ch if is_line_terminator(ch) => return Err("invalid regular expression pattern"),
            _ => index += 1,
        }
    }

    Err("invalid regular expression pattern")
}

fn parse_unicode_escape(
    chars: &[char],
    start: usize,
    unicode: bool,
) -> Result<(usize, ClassAtomKind), &'static str> {
    if chars.get(start + 2) == Some(&'{') {
        if !unicode {
            return Ok((start + 2, ClassAtomKind::Single));
        }

        let mut end = start + 3;
        while end < chars.len() && chars[end] != '}' {
            if !chars[end].is_ascii_hexdigit() {
                return Err("invalid regular expression pattern");
            }
            end += 1;
        }
        if end == start + 3 || end >= chars.len() {
            return Err("invalid regular expression pattern");
        }

        let value: String = chars[start + 3..end].iter().collect();
        let value =
            u32::from_str_radix(&value, 16).map_err(|_| "invalid regular expression pattern")?;
        if value > 0x10FFFF {
            return Err("invalid regular expression pattern");
        }

        Ok((end + 1, ClassAtomKind::Single))
    } else {
        if !has_hex_digits(chars, start + 2, 4) {
            if !unicode {
                return Ok((start + 2, ClassAtomKind::Single));
            }
            return Err("invalid regular expression pattern");
        }
        Ok((start + 6, ClassAtomKind::Single))
    }
}

fn validate_duplicate_named_groups(chars: &[char]) -> Result<(), &'static str> {
    let mut i = 0;
    let mut paren_depth = 0usize;
    let mut alt_indices = HashMap::from([(0usize, 0usize)]);
    let mut locations: HashMap<String, Vec<AlternativePath>> = HashMap::new();

    while i < chars.len() {
        match chars[i] {
            '\\' => i += 2,
            '[' => {
                i += 1;
                while i < chars.len() {
                    match chars[i] {
                        '\\' => i += 2,
                        ']' => {
                            i += 1;
                            break;
                        }
                        _ => i += 1,
                    }
                }
            }
            '(' => {
                if let Some((end, name)) = named_capture_group_at(chars, i) {
                    let segments = (0..=paren_depth)
                        .map(|depth| (depth, *alt_indices.get(&depth).unwrap_or(&0)))
                        .collect();
                    locations
                        .entry(name)
                        .or_default()
                        .push(AlternativePath { segments });
                    i = end;
                } else {
                    i += 1;
                }
                paren_depth += 1;
                alt_indices.insert(paren_depth, 0);
            }
            ')' => {
                if paren_depth > 0 {
                    alt_indices.remove(&paren_depth);
                    paren_depth -= 1;
                }
                i += 1;
            }
            '|' => {
                *alt_indices.entry(paren_depth).or_insert(0) += 1;
                i += 1;
            }
            _ => i += 1,
        }
    }

    for paths in locations.values() {
        for index in 0..paths.len() {
            if paths[index + 1..]
                .iter()
                .any(|candidate| paths[index].conflicts_with(candidate))
            {
                return Err("invalid regular expression pattern");
            }
        }
    }

    Ok(())
}

fn named_capture_group_at(chars: &[char], start: usize) -> Option<(usize, String)> {
    if chars.get(start + 1) != Some(&'?') || chars.get(start + 2) != Some(&'<') {
        return None;
    }
    if matches!(chars.get(start + 3), Some('=') | Some('!')) {
        return None;
    }

    let mut end = start + 3;
    while end < chars.len() && chars[end] != '>' {
        if is_line_terminator(chars[end]) {
            return None;
        }
        end += 1;
    }
    if end >= chars.len() {
        return None;
    }

    parse_group_name_contents(&chars[start + 3..end]).map(|name| (end + 1, name))
}

fn collect_named_group_names(pattern: &str) -> HashSet<String> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    let mut in_class = false;
    let mut names = HashSet::new();

    while i < chars.len() {
        match chars[i] {
            '\\' => i += 2,
            '[' => {
                in_class = true;
                i += 1;
            }
            ']' if in_class => {
                in_class = false;
                i += 1;
            }
            '(' if !in_class => {
                if chars.get(i + 1) != Some(&'?') || chars.get(i + 2) != Some(&'<') {
                    i += 1;
                    continue;
                }

                match chars.get(i + 3) {
                    Some('=') | Some('!') => {
                        i += 1;
                    }
                    _ => {
                        let mut end = i + 3;
                        while end < chars.len() && chars[end] != '>' {
                            if is_line_terminator(chars[end]) {
                                break;
                            }
                            end += 1;
                        }

                        if end < chars.len() && chars[end] == '>' {
                            if let Some(name) = parse_group_name_contents(&chars[i + 3..end]) {
                                names.insert(name);
                            }
                        }

                        i += 1;
                    }
                }
            }
            _ => i += 1,
        }
    }

    names
}

fn parse_named_reference(
    chars: &[char],
    start: usize,
) -> Result<Option<(usize, String)>, &'static str> {
    if chars.get(start + 2) != Some(&'<') {
        return Ok(None);
    }

    let mut end = start + 3;
    while end < chars.len() && chars[end] != '>' {
        if is_line_terminator(chars[end]) {
            return Err("invalid regular expression pattern");
        }
        end += 1;
    }
    if end >= chars.len() {
        return Err("invalid regular expression pattern");
    }

    let Some(name) = parse_group_name_contents(&chars[start + 3..end]) else {
        return Err("invalid regular expression pattern");
    };
    Ok(Some((end + 1, name)))
}

fn has_hex_digits(chars: &[char], start: usize, count: usize) -> bool {
    (0..count).all(|offset| {
        chars
            .get(start + offset)
            .is_some_and(char::is_ascii_hexdigit)
    })
}

fn has_unescaped_class_closer(chars: &[char], mut start: usize) -> bool {
    while start < chars.len() {
        match chars[start] {
            '\\' => start += 2,
            ']' => return true,
            _ => start += 1,
        }
    }

    false
}

fn parse_group_name_contents(chars: &[char]) -> Option<String> {
    let mut index = 0;
    let mut name = String::new();

    while index < chars.len() {
        let ch = parse_group_name_char(chars, &mut index)?;
        if name.is_empty() {
            if !(ch == '$' || ch == '_' || unicode_id_start::is_id_start(ch)) {
                return None;
            }
        } else if !(ch == '$'
            || ch == '\u{200C}'
            || ch == '\u{200D}'
            || unicode_id_start::is_id_continue(ch))
        {
            return None;
        }

        name.push(ch);
    }

    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn parse_group_name_char(chars: &[char], index: &mut usize) -> Option<char> {
    let ch = *chars.get(*index)?;
    if ch != '\\' {
        *index += 1;
        return Some(ch);
    }

    if chars.get(*index + 1) != Some(&'u') {
        return None;
    }

    let (code_point, next_index) = parse_group_name_unicode_escape(chars, *index)?;
    *index = next_index;
    char::from_u32(code_point)
}

fn parse_group_name_unicode_escape(chars: &[char], start: usize) -> Option<(u32, usize)> {
    if chars.get(start + 1) != Some(&'u') {
        return None;
    }

    if chars.get(start + 2) == Some(&'{') {
        let mut end = start + 3;
        while end < chars.len() && chars[end] != '}' {
            if !chars[end].is_ascii_hexdigit() {
                return None;
            }
            end += 1;
        }
        if end == start + 3 || end >= chars.len() {
            return None;
        }

        let value: String = chars[start + 3..end].iter().collect();
        let code_point = u32::from_str_radix(&value, 16).ok()?;
        if code_point > 0x10FFFF {
            return None;
        }
        return Some((code_point, end + 1));
    }

    if !has_hex_digits(chars, start + 2, 4) {
        return None;
    }

    let unit: String = chars[start + 2..start + 6].iter().collect();
    let code_unit = u16::from_str_radix(&unit, 16).ok()?;

    if (0xD800..=0xDBFF).contains(&code_unit) {
        if chars.get(start + 6) != Some(&'\\') || chars.get(start + 7) != Some(&'u') {
            return None;
        }
        if !has_hex_digits(chars, start + 8, 4) {
            return None;
        }

        let low: String = chars[start + 8..start + 12].iter().collect();
        let low_code_unit = u16::from_str_radix(&low, 16).ok()?;
        if !(0xDC00..=0xDFFF).contains(&low_code_unit) {
            return None;
        }

        let high = u32::from(code_unit) - 0xD800;
        let low = u32::from(low_code_unit) - 0xDC00;
        let code_point = 0x10000 + ((high << 10) | low);
        return Some((code_point, start + 12));
    }

    if (0xDC00..=0xDFFF).contains(&code_unit) {
        return None;
    }

    Some((u32::from(code_unit), start + 6))
}

fn is_unicode_identity_escape(ch: char, in_class: bool) -> bool {
    matches!(
        ch,
        '^' | '$' | '\\' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '/'
    ) || (in_class && ch == '-')
}

fn is_line_terminator(ch: char) -> bool {
    matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}
