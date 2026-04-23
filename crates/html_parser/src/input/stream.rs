use std::borrow::Cow;
use std::collections::VecDeque;

use crate::error::{ParseError, ParseErrorCode};

#[derive(Debug)]
struct Segment<'a> {
    data: Cow<'a, str>,
    pos: usize,
}

impl<'a> Segment<'a> {
    fn new(input: impl Into<Cow<'a, str>>) -> Option<Self> {
        let data = preprocess(input);
        if data.is_empty() {
            None
        } else {
            Some(Self { data, pos: 0 })
        }
    }

    fn new_owned(input: impl Into<String>) -> Option<Self> {
        let input = Cow::Owned(input.into());
        let data = preprocess(input);
        if data.is_empty() {
            None
        } else {
            Some(Self { data, pos: 0 })
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn peek(&self) -> Option<char> {
        self.data[self.pos..].chars().next()
    }

    fn is_exhausted(&self) -> bool {
        self.pos >= self.data.len()
    }
}

/// A character-by-character input stream with WHATWG preprocessing (§13.2.3.5).
///
/// Handles:
/// - CR/CRLF → LF normalization
/// - Line/column tracking
/// - Reconsume (push back a character)
/// - Null character and noncharacter error reporting
pub struct InputStream<'a> {
    segments: VecDeque<Segment<'a>>,
    reconsume: bool,
    at_eof: bool,
    last_char: Option<char>,
    line: u32,
    col: u32,
    /// Line/col of the most recently consumed character.
    pub current_line: u32,
    pub current_col: u32,
    pub errors: Vec<ParseError>,
}

impl<'a> InputStream<'a> {
    /// Create a new input stream from a UTF-8 string.
    pub fn new(input: &'a str) -> Self {
        let mut segments = VecDeque::new();
        if let Some(segment) = Segment::new(input) {
            segments.push_back(segment);
        }

        InputStream {
            segments,
            reconsume: false,
            at_eof: false,
            last_char: None,
            line: 1,
            col: 0,
            current_line: 1,
            current_col: 0,
            errors: Vec::new(),
        }
    }

    /// Insert additional HTML at the current read position.
    pub fn insert_html_at_current_position(&mut self, input: impl Into<String>) {
        if let Some(segment) = Segment::new_owned(input) {
            self.segments.push_front(segment);
            self.at_eof = false;
        }
    }

    /// Consume and return the next character, or None at EOF.
    pub fn next_char(&mut self) -> Option<char> {
        if self.reconsume {
            self.reconsume = false;
            if self.at_eof {
                return None;
            }
            // current_line and current_col stay the same on reconsume
            return self.last_char;
        }

        loop {
            let (ch, exhausted) = match self.segments.front_mut() {
                Some(segment) => match segment.next_char() {
                    Some(ch) => (ch, segment.is_exhausted()),
                    None => {
                        self.segments.pop_front();
                        continue;
                    }
                },
                None => {
                    self.at_eof = true;
                    self.last_char = None;
                    return None;
                }
            };

            if exhausted {
                self.segments.pop_front();
            }

            self.at_eof = false;
            self.last_char = Some(ch);
            self.advance_position(ch);
            self.report_input_errors(ch);
            return Some(ch);
        }
    }

    fn advance_position(&mut self, ch: char) {
        if ch == '\n' {
            self.current_line = self.line;
            self.current_col = self.col + 1;
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
            self.current_line = self.line;
            self.current_col = self.col;
        }
    }

    fn report_input_errors(&mut self, ch: char) {
        if is_control_character(ch) {
            self.errors.push(ParseError {
                code: ParseErrorCode::ControlCharacterInInputStream,
                line: self.current_line,
                col: self.current_col,
            });
        }

        if is_noncharacter(ch) {
            self.errors.push(ParseError {
                code: ParseErrorCode::NoncharacterInInputStream,
                line: self.current_line,
                col: self.current_col,
            });
        }
    }

    /// Mark the current character to be re-consumed on the next call to `next_char`.
    pub fn reconsume(&mut self) {
        self.reconsume = true;
    }

    /// Peek at the next character without consuming it.
    pub fn peek(&self) -> Option<char> {
        if self.reconsume && !self.at_eof {
            return self.last_char;
        }

        self.segments.iter().find_map(Segment::peek)
    }

    /// Get the current source position (line, column of the last consumed char).
    pub fn position(&self) -> (u32, u32) {
        (self.current_line, self.current_col)
    }
}

fn preprocess<'a>(input: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
    let input = input.into();
    if !input.contains('\r') {
        return input;
    }

    // Preprocess: normalize CR/CRLF to LF during construction.
    let mut chars = String::with_capacity(input.len());
    let mut raw = input.chars().peekable();

    while let Some(ch) = raw.next() {
        if ch == '\r' {
            chars.push('\n');
            if raw.peek() == Some(&'\n') {
                raw.next();
            }
        } else {
            chars.push(ch);
        }
    }

    Cow::Owned(chars)
}

/// Returns true if the code point is a noncharacter (§13.2.3.5).
fn is_noncharacter(ch: char) -> bool {
    let cp = ch as u32;
    (0xFDD0..=0xFDEF).contains(&cp)
        || matches!(
            cp,
            0xFFFE
                | 0xFFFF
                | 0x1FFFE
                | 0x1FFFF
                | 0x2FFFE
                | 0x2FFFF
                | 0x3FFFE
                | 0x3FFFF
                | 0x4FFFE
                | 0x4FFFF
                | 0x5FFFE
                | 0x5FFFF
                | 0x6FFFE
                | 0x6FFFF
                | 0x7FFFE
                | 0x7FFFF
                | 0x8FFFE
                | 0x8FFFF
                | 0x9FFFE
                | 0x9FFFF
                | 0xAFFFE
                | 0xAFFFF
                | 0xBFFFE
                | 0xBFFFF
                | 0xCFFFE
                | 0xCFFFF
                | 0xDFFFE
                | 0xDFFFF
                | 0xEFFFE
                | 0xEFFFF
                | 0xFFFFE
                | 0xFFFFF
                | 0x10FFFE
                | 0x10FFFF
        )
}

fn is_control_character(ch: char) -> bool {
    matches!(ch, '\u{007F}'..='\u{009F}')
        || matches!(ch, '\u{0001}'..='\u{0008}' | '\u{000B}' | '\u{000E}'..='\u{001F}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_iteration() {
        let mut stream = InputStream::new("abc");
        assert_eq!(stream.next_char(), Some('a'));
        assert_eq!(stream.next_char(), Some('b'));
        assert_eq!(stream.next_char(), Some('c'));
        assert_eq!(stream.next_char(), None);
    }

    #[test]
    fn empty_input() {
        let mut stream = InputStream::new("");
        assert_eq!(stream.next_char(), None);
    }

    #[test]
    fn crlf_normalization() {
        let mut stream = InputStream::new("a\r\nb\rc");
        assert_eq!(stream.next_char(), Some('a'));
        assert_eq!(stream.next_char(), Some('\n'));
        assert_eq!(stream.next_char(), Some('b'));
        assert_eq!(stream.next_char(), Some('\n'));
        assert_eq!(stream.next_char(), Some('c'));
        assert_eq!(stream.next_char(), None);
    }

    #[test]
    fn cr_only_normalization() {
        let mut stream = InputStream::new("a\rb");
        assert_eq!(stream.next_char(), Some('a'));
        assert_eq!(stream.next_char(), Some('\n'));
        assert_eq!(stream.next_char(), Some('b'));
    }

    #[test]
    fn line_column_tracking() {
        let mut stream = InputStream::new("ab\ncd");
        stream.next_char(); // a -> line 1, col 1
        assert_eq!(stream.position(), (1, 1));
        stream.next_char(); // b -> line 1, col 2
        assert_eq!(stream.position(), (1, 2));
        stream.next_char(); // \n -> line 1, col 3
        assert_eq!(stream.position(), (1, 3));
        stream.next_char(); // c -> line 2, col 1
        assert_eq!(stream.position(), (2, 1));
        stream.next_char(); // d -> line 2, col 2
        assert_eq!(stream.position(), (2, 2));
    }

    #[test]
    fn reconsume_char() {
        let mut stream = InputStream::new("abc");
        assert_eq!(stream.next_char(), Some('a'));
        stream.reconsume();
        assert_eq!(stream.next_char(), Some('a'));
        assert_eq!(stream.next_char(), Some('b'));
    }

    #[test]
    fn peek_without_consuming() {
        let mut stream = InputStream::new("ab");
        assert_eq!(stream.peek(), Some('a'));
        assert_eq!(stream.peek(), Some('a'));
        assert_eq!(stream.next_char(), Some('a'));
        assert_eq!(stream.peek(), Some('b'));
    }

    #[test]
    fn peek_with_reconsume() {
        let mut stream = InputStream::new("ab");
        stream.next_char(); // a
        stream.reconsume();
        assert_eq!(stream.peek(), Some('a'));
        assert_eq!(stream.next_char(), Some('a'));
        assert_eq!(stream.next_char(), Some('b'));
    }

    #[test]
    fn noncharacter_error() {
        let input = String::from("\u{FDD0}");
        let mut stream = InputStream::new(&input);
        stream.next_char();
        assert_eq!(stream.errors.len(), 1);
        assert_eq!(
            stream.errors[0].code,
            ParseErrorCode::NoncharacterInInputStream
        );
    }

    #[test]
    fn crlf_at_end_of_input() {
        let mut stream = InputStream::new("a\r\n");
        assert_eq!(stream.next_char(), Some('a'));
        assert_eq!(stream.next_char(), Some('\n'));
        assert_eq!(stream.next_char(), None);
    }

    #[test]
    fn multiple_cr() {
        let mut stream = InputStream::new("\r\r");
        assert_eq!(stream.next_char(), Some('\n'));
        assert_eq!(stream.next_char(), Some('\n'));
        assert_eq!(stream.next_char(), None);
    }

    #[test]
    fn inserts_html_at_current_position_without_reordering_remaining_input() {
        let mut stream = InputStream::new("ab");
        assert_eq!(stream.next_char(), Some('a'));
        stream.insert_html_at_current_position("XY");
        assert_eq!(stream.next_char(), Some('X'));
        assert_eq!(stream.next_char(), Some('Y'));
        assert_eq!(stream.next_char(), Some('b'));
        assert_eq!(stream.next_char(), None);
    }
}
