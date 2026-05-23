//! Stateless trivia scanning over a known source range.
//!
//! The parser sometimes needs to look at the whitespace/comment region
//! between two already-lexed tokens — e.g. to detect a line terminator in
//! arrow-cover-grammar lookahead, or to spot a trailing comma before `)`.
//! This module owns that scan so the trivia grammar lives next to the
//! [`Lexer`](crate::Lexer)'s own scanner and cannot silently diverge.

/// Find the first non-trivia character in `source[start..end]`.
///
/// Skips ECMA-262 whitespace and `//` / `/* */` comments. Returns the
/// character, its byte offset, and whether a `LineTerminator` (LF, CR,
/// U+2028, U+2029) was seen in the skipped trivia. Returns `None` if the
/// range contains only trivia.
#[must_use]
pub fn next_non_trivia(source: &str, start: u32, end: u32) -> Option<(char, u32, bool)> {
    let bytes = source.as_bytes();
    let mut pos = start as usize;
    let end = end as usize;
    let mut saw_line_terminator = false;

    while pos < end {
        match bytes[pos] {
            b' ' | b'\t' | 0x0B | 0x0C => pos += 1,
            b'\r' => {
                saw_line_terminator = true;
                pos += 1;
                if pos < end && bytes[pos] == b'\n' {
                    pos += 1;
                }
            }
            b'\n' => {
                saw_line_terminator = true;
                pos += 1;
            }
            b'/' if pos + 1 < end && bytes[pos + 1] == b'/' => {
                pos += 2;
                while pos < end {
                    match bytes[pos] {
                        b'\r' => {
                            saw_line_terminator = true;
                            pos += 1;
                            if pos < end && bytes[pos] == b'\n' {
                                pos += 1;
                            }
                            break;
                        }
                        b'\n' => {
                            saw_line_terminator = true;
                            pos += 1;
                            break;
                        }
                        _ => pos += 1,
                    }
                }
            }
            b'/' if pos + 1 < end && bytes[pos + 1] == b'*' => {
                pos += 2;
                while pos < end {
                    if pos + 1 < end && bytes[pos] == b'*' && bytes[pos + 1] == b'/' {
                        pos += 2;
                        break;
                    }
                    match bytes[pos] {
                        b'\r' => {
                            saw_line_terminator = true;
                            pos += 1;
                            if pos < end && bytes[pos] == b'\n' {
                                pos += 1;
                            }
                        }
                        b'\n' => {
                            saw_line_terminator = true;
                            pos += 1;
                        }
                        0xE2 if pos + 2 < end
                            && bytes[pos + 1] == 0x80
                            && matches!(bytes[pos + 2], 0xA8 | 0xA9) =>
                        {
                            saw_line_terminator = true;
                            pos += 3;
                        }
                        _ => pos += 1,
                    }
                }
            }
            0xE2 if pos + 2 < end
                && bytes[pos + 1] == 0x80
                && matches!(bytes[pos + 2], 0xA8 | 0xA9) =>
            {
                saw_line_terminator = true;
                pos += 3;
            }
            _ => {
                let ch = source[pos..end].chars().next().unwrap_or_default();
                return Some((ch, pos as u32, saw_line_terminator));
            }
        }
    }

    None
}
