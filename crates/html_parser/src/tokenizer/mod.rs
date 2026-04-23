pub mod entities;
pub mod states;
pub mod tokens;

use std::collections::{HashSet, VecDeque};

use crate::error::{ParseError, ParseErrorCode};
use crate::input::InputStream;
use states::State;
use tokens::{Attribute, Token};

/// The HTML tokenizer. Consumes characters from an InputStream and emits Tokens.
pub struct Tokenizer<'a> {
    pub input: InputStream<'a>,
    state: State,
    return_state: State,
    current_tag_name: String,
    current_tag_is_end_tag: bool,
    current_tag_self_closing: bool,
    current_tag_attributes: Vec<Attribute>,
    current_tag_attribute_names: Option<HashSet<String>>,
    current_attr_name: String,
    current_attr_value: String,
    current_comment_data: String,
    current_doctype_name: Option<String>,
    current_doctype_public_id: Option<String>,
    current_doctype_system_id: Option<String>,
    current_doctype_force_quirks: bool,
    temp_buffer: String,
    last_start_tag_name: Option<String>,
    character_reference_code: u32,
    pending_tokens: VecDeque<Token>,
    /// Set by tree construction when the adjusted current node is not in the HTML namespace.
    /// Affects CDATA section handling in markup declaration open state.
    pub adjusted_current_node_not_in_html: bool,
    pub errors: Vec<ParseError>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: InputStream<'a>) -> Self {
        Tokenizer {
            input,
            state: State::Data,
            return_state: State::Data,
            current_tag_name: String::new(),
            current_tag_is_end_tag: false,
            current_tag_self_closing: false,
            current_tag_attributes: Vec::new(),
            current_tag_attribute_names: None,
            current_attr_name: String::new(),
            current_attr_value: String::new(),
            current_comment_data: String::new(),
            current_doctype_name: None,
            current_doctype_public_id: None,
            current_doctype_system_id: None,
            current_doctype_force_quirks: false,
            temp_buffer: String::new(),
            last_start_tag_name: None,
            character_reference_code: 0,
            pending_tokens: VecDeque::new(),
            adjusted_current_node_not_in_html: false,
            errors: Vec::new(),
        }
    }

    /// Set the tokenizer state (used by tree construction to switch to RCDATA, RAWTEXT, etc.)
    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    /// Set the last start tag name (needed for tree construction to set up appropriate end tag checks).
    pub fn set_last_start_tag(&mut self, name: &str) {
        self.last_start_tag_name = Some(name.to_string());
    }

    /// Get the next token from the tokenizer.
    pub fn next_token(&mut self) -> Token {
        if let Some(token) = self.pending_tokens.pop_front() {
            return token;
        }

        loop {
            if let Some(token) = self.pending_tokens.pop_front() {
                return token;
            }

            self.step();
        }
    }

    fn step(&mut self) {
        match self.state {
            State::Data => self.state_data(),
            State::TagOpen => self.state_tag_open(),
            State::EndTagOpen => self.state_end_tag_open(),
            State::TagName => self.state_tag_name(),
            State::BeforeAttributeName => self.state_before_attribute_name(),
            State::AttributeName => self.state_attribute_name(),
            State::AfterAttributeName => self.state_after_attribute_name(),
            State::BeforeAttributeValue => self.state_before_attribute_value(),
            State::AttributeValueDoubleQuoted => self.state_attribute_value_double_quoted(),
            State::AttributeValueSingleQuoted => self.state_attribute_value_single_quoted(),
            State::AttributeValueUnquoted => self.state_attribute_value_unquoted(),
            State::AfterAttributeValueQuoted => self.state_after_attribute_value_quoted(),
            State::SelfClosingStartTag => self.state_self_closing_start_tag(),
            State::BogusComment => self.state_bogus_comment(),
            State::MarkupDeclarationOpen => self.state_markup_declaration_open(),
            State::CommentStart => self.state_comment_start(),
            State::CommentStartDash => self.state_comment_start_dash(),
            State::Comment => self.state_comment(),
            State::CommentLessThanSign => self.state_comment_less_than_sign(),
            State::CommentLessThanSignBang => self.state_comment_less_than_sign_bang(),
            State::CommentLessThanSignBangDash => self.state_comment_less_than_sign_bang_dash(),
            State::CommentLessThanSignBangDashDash => {
                self.state_comment_less_than_sign_bang_dash_dash()
            }
            State::CommentEndDash => self.state_comment_end_dash(),
            State::CommentEnd => self.state_comment_end(),
            State::CommentEndBang => self.state_comment_end_bang(),
            State::Doctype => self.state_doctype(),
            State::BeforeDoctypeName => self.state_before_doctype_name(),
            State::DoctypeName => self.state_doctype_name(),
            State::AfterDoctypeName => self.state_after_doctype_name(),
            State::AfterDoctypePublicKeyword => self.state_after_doctype_public_keyword(),
            State::BeforeDoctypePublicIdentifier => self.state_before_doctype_public_identifier(),
            State::DoctypePublicIdentifierDoubleQuoted => {
                self.state_doctype_public_identifier_double_quoted()
            }
            State::DoctypePublicIdentifierSingleQuoted => {
                self.state_doctype_public_identifier_single_quoted()
            }
            State::AfterDoctypePublicIdentifier => self.state_after_doctype_public_identifier(),
            State::BetweenDoctypePublicAndSystemIdentifiers => {
                self.state_between_doctype_public_and_system_identifiers()
            }
            State::AfterDoctypeSystemKeyword => self.state_after_doctype_system_keyword(),
            State::BeforeDoctypeSystemIdentifier => self.state_before_doctype_system_identifier(),
            State::DoctypeSystemIdentifierDoubleQuoted => {
                self.state_doctype_system_identifier_double_quoted()
            }
            State::DoctypeSystemIdentifierSingleQuoted => {
                self.state_doctype_system_identifier_single_quoted()
            }
            State::AfterDoctypeSystemIdentifier => self.state_after_doctype_system_identifier(),
            State::BogusDoctype => self.state_bogus_doctype(),
            State::CharacterReference => self.state_character_reference(),
            State::NamedCharacterReference => self.state_named_character_reference(),
            State::AmbiguousAmpersand => self.state_ambiguous_ampersand(),
            State::NumericCharacterReference => self.state_numeric_character_reference(),
            State::HexadecimalCharacterReferenceStart => {
                self.state_hexadecimal_character_reference_start()
            }
            State::DecimalCharacterReferenceStart => self.state_decimal_character_reference_start(),
            State::HexadecimalCharacterReference => self.state_hexadecimal_character_reference(),
            State::DecimalCharacterReference => self.state_decimal_character_reference(),
            State::NumericCharacterReferenceEnd => self.state_numeric_character_reference_end(),
            State::RcData => self.state_rcdata(),
            State::RcDataLessThanSign => self.state_rcdata_less_than_sign(),
            State::RcDataEndTagOpen => self.state_rcdata_end_tag_open(),
            State::RcDataEndTagName => self.state_rcdata_end_tag_name(),
            State::RawText => self.state_rawtext(),
            State::RawTextLessThanSign => self.state_rawtext_less_than_sign(),
            State::RawTextEndTagOpen => self.state_rawtext_end_tag_open(),
            State::RawTextEndTagName => self.state_rawtext_end_tag_name(),
            State::ScriptData => self.state_script_data(),
            State::ScriptDataLessThanSign => self.state_script_data_less_than_sign(),
            State::ScriptDataEndTagOpen => self.state_script_data_end_tag_open(),
            State::ScriptDataEndTagName => self.state_script_data_end_tag_name(),
            State::ScriptDataEscapeStart => self.state_script_data_escape_start(),
            State::ScriptDataEscapeStartDash => self.state_script_data_escape_start_dash(),
            State::ScriptDataEscaped => self.state_script_data_escaped(),
            State::ScriptDataEscapedDash => self.state_script_data_escaped_dash(),
            State::ScriptDataEscapedDashDash => self.state_script_data_escaped_dash_dash(),
            State::ScriptDataEscapedLessThanSign => self.state_script_data_escaped_less_than_sign(),
            State::ScriptDataEscapedEndTagOpen => self.state_script_data_escaped_end_tag_open(),
            State::ScriptDataEscapedEndTagName => self.state_script_data_escaped_end_tag_name(),
            State::ScriptDataDoubleEscapeStart => self.state_script_data_double_escape_start(),
            State::ScriptDataDoubleEscaped => self.state_script_data_double_escaped(),
            State::ScriptDataDoubleEscapedDash => self.state_script_data_double_escaped_dash(),
            State::ScriptDataDoubleEscapedDashDash => {
                self.state_script_data_double_escaped_dash_dash()
            }
            State::ScriptDataDoubleEscapedLessThanSign => {
                self.state_script_data_double_escaped_less_than_sign()
            }
            State::ScriptDataDoubleEscapeEnd => self.state_script_data_double_escape_end(),
            State::PlainText => self.state_plaintext(),
            State::CdataSection => self.state_cdata_section(),
            State::CdataSectionBracket => self.state_cdata_section_bracket(),
            State::CdataSectionEnd => self.state_cdata_section_end(),
        }
    }

    // -----------------------------------------------------------------------
    // State implementations
    // -----------------------------------------------------------------------

    /// §13.2.5.1 Data state
    fn state_data(&mut self) {
        match self.input.next_char() {
            Some('&') => {
                self.return_state = State::Data;
                self.state = State::CharacterReference;
            }
            Some('<') => {
                self.state = State::TagOpen;
            }
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.emit_char('\0');
            }
            None => {
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => {
                self.emit_char(c);
            }
        }
    }

    /// §13.2.5.6 Tag open state
    fn state_tag_open(&mut self) {
        match self.input.next_char() {
            Some('!') => {
                self.state = State::MarkupDeclarationOpen;
            }
            Some('/') => {
                self.state = State::EndTagOpen;
            }
            Some(c) if c.is_ascii_alphabetic() => {
                self.create_start_tag();
                self.input.reconsume();
                self.state = State::TagName;
            }
            Some('?') => {
                self.emit_error(ParseErrorCode::UnexpectedQuestionMarkInsteadOfTagName);
                self.create_comment("");
                self.input.reconsume();
                self.state = State::BogusComment;
            }
            None => {
                self.emit_error(ParseErrorCode::EofBeforeTagName);
                self.emit_char('<');
                self.emit_token(Token::EndOfFile);
            }
            Some(_) => {
                self.emit_error(ParseErrorCode::InvalidFirstCharacterOfTagName);
                self.emit_char('<');
                self.input.reconsume();
                self.state = State::Data;
            }
        }
    }

    /// §13.2.5.7 End tag open state
    fn state_end_tag_open(&mut self) {
        match self.input.next_char() {
            Some(c) if c.is_ascii_alphabetic() => {
                self.create_end_tag();
                self.input.reconsume();
                self.state = State::TagName;
            }
            Some('>') => {
                self.emit_error(ParseErrorCode::MissingEndTagName);
                self.state = State::Data;
            }
            None => {
                self.emit_error(ParseErrorCode::EofBeforeTagName);
                self.emit_char('<');
                self.emit_char('/');
                self.emit_token(Token::EndOfFile);
            }
            Some(_) => {
                self.emit_error(ParseErrorCode::InvalidFirstCharacterOfTagName);
                self.create_comment("");
                self.input.reconsume();
                self.state = State::BogusComment;
            }
        }
    }

    /// §13.2.5.8 Tag name state
    fn state_tag_name(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = State::BeforeAttributeName;
            }
            Some('/') => {
                self.state = State::SelfClosingStartTag;
            }
            Some('>') => {
                self.state = State::Data;
                self.emit_current_tag();
            }
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.current_tag_name.push('\u{FFFD}');
            }
            None => {
                self.emit_error(ParseErrorCode::EofInTag);
                self.emit_token(Token::EndOfFile);
            }
            Some(c) if c.is_ascii_uppercase() => {
                self.current_tag_name.push(c.to_ascii_lowercase());
            }
            Some(c) => {
                self.current_tag_name.push(c);
            }
        }
    }

    /// §13.2.5.32 Before attribute name state
    fn state_before_attribute_name(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                // Ignore whitespace
            }
            Some('/') | Some('>') | None => {
                self.input.reconsume();
                self.state = State::AfterAttributeName;
            }
            Some('=') => {
                self.emit_error(ParseErrorCode::UnexpectedEqualsSignBeforeAttributeName);
                self.start_new_attribute();
                self.current_attr_name.push('=');
                self.state = State::AttributeName;
            }
            Some(_) => {
                self.start_new_attribute();
                self.input.reconsume();
                self.state = State::AttributeName;
            }
        }
    }

    /// §13.2.5.33 Attribute name state
    fn state_attribute_name(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') | Some('/') | Some('>') | None => {
                self.input.reconsume();
                self.state = State::AfterAttributeName;
            }
            Some('=') => {
                self.state = State::BeforeAttributeValue;
            }
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.current_attr_name.push('\u{FFFD}');
            }
            Some(c @ '"') | Some(c @ '\'') | Some(c @ '<') => {
                self.emit_error(ParseErrorCode::UnexpectedCharacterInAttributeName);
                self.current_attr_name.push(c);
            }
            Some(c) if c.is_ascii_uppercase() => {
                self.current_attr_name.push(c.to_ascii_lowercase());
            }
            Some(c) => {
                self.current_attr_name.push(c);
            }
        }
    }

    /// §13.2.5.34 After attribute name state
    fn state_after_attribute_name(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                // Ignore whitespace
            }
            Some('/') => {
                self.state = State::SelfClosingStartTag;
            }
            Some('=') => {
                self.state = State::BeforeAttributeValue;
            }
            Some('>') => {
                self.state = State::Data;
                self.emit_current_tag();
            }
            None => {
                self.emit_error(ParseErrorCode::EofInTag);
                self.emit_token(Token::EndOfFile);
            }
            Some(_) => {
                self.start_new_attribute();
                self.input.reconsume();
                self.state = State::AttributeName;
            }
        }
    }

    /// §13.2.5.35 Before attribute value state
    fn state_before_attribute_value(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                // Ignore whitespace
            }
            Some('"') => {
                self.state = State::AttributeValueDoubleQuoted;
            }
            Some('\'') => {
                self.state = State::AttributeValueSingleQuoted;
            }
            Some('>') => {
                self.emit_error(ParseErrorCode::MissingAttributeValue);
                self.state = State::Data;
                self.emit_current_tag();
            }
            _ => {
                self.input.reconsume();
                self.state = State::AttributeValueUnquoted;
            }
        }
    }

    /// §13.2.5.36 Attribute value (double-quoted) state
    fn state_attribute_value_double_quoted(&mut self) {
        match self.input.next_char() {
            Some('"') => {
                self.state = State::AfterAttributeValueQuoted;
            }
            Some('&') => {
                self.return_state = State::AttributeValueDoubleQuoted;
                self.state = State::CharacterReference;
            }
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.current_attr_value.push('\u{FFFD}');
            }
            None => {
                self.emit_error(ParseErrorCode::EofInTag);
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => {
                self.current_attr_value.push(c);
            }
        }
    }

    /// §13.2.5.37 Attribute value (single-quoted) state
    fn state_attribute_value_single_quoted(&mut self) {
        match self.input.next_char() {
            Some('\'') => {
                self.state = State::AfterAttributeValueQuoted;
            }
            Some('&') => {
                self.return_state = State::AttributeValueSingleQuoted;
                self.state = State::CharacterReference;
            }
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.current_attr_value.push('\u{FFFD}');
            }
            None => {
                self.emit_error(ParseErrorCode::EofInTag);
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => {
                self.current_attr_value.push(c);
            }
        }
    }

    /// §13.2.5.38 Attribute value (unquoted) state
    fn state_attribute_value_unquoted(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = State::BeforeAttributeName;
            }
            Some('&') => {
                self.return_state = State::AttributeValueUnquoted;
                self.state = State::CharacterReference;
            }
            Some('>') => {
                self.state = State::Data;
                self.emit_current_tag();
            }
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.current_attr_value.push('\u{FFFD}');
            }
            Some(c @ '"') | Some(c @ '\'') | Some(c @ '<') | Some(c @ '=') | Some(c @ '`') => {
                self.emit_error(ParseErrorCode::UnexpectedCharacterInUnquotedAttributeValue);
                self.current_attr_value.push(c);
            }
            None => {
                self.emit_error(ParseErrorCode::EofInTag);
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => {
                self.current_attr_value.push(c);
            }
        }
    }

    /// §13.2.5.39 After attribute value (quoted) state
    fn state_after_attribute_value_quoted(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = State::BeforeAttributeName;
            }
            Some('/') => {
                self.state = State::SelfClosingStartTag;
            }
            Some('>') => {
                self.state = State::Data;
                self.emit_current_tag();
            }
            None => {
                self.emit_error(ParseErrorCode::EofInTag);
                self.emit_token(Token::EndOfFile);
            }
            Some(_) => {
                self.emit_error(ParseErrorCode::MissingWhitespaceBetweenAttributes);
                self.input.reconsume();
                self.state = State::BeforeAttributeName;
            }
        }
    }

    /// §13.2.5.40 Self-closing start tag state
    fn state_self_closing_start_tag(&mut self) {
        match self.input.next_char() {
            Some('>') => {
                self.current_tag_self_closing = true;
                self.state = State::Data;
                self.emit_current_tag();
            }
            None => {
                self.emit_error(ParseErrorCode::EofInTag);
                self.emit_token(Token::EndOfFile);
            }
            Some(_) => {
                self.emit_error(ParseErrorCode::UnexpectedSolidusInTag);
                self.input.reconsume();
                self.state = State::BeforeAttributeName;
            }
        }
    }

    /// §13.2.5.41 Bogus comment state
    fn state_bogus_comment(&mut self) {
        match self.input.next_char() {
            Some('>') => {
                self.state = State::Data;
                self.emit_current_comment();
            }
            None => {
                self.emit_current_comment();
                self.emit_token(Token::EndOfFile);
            }
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.current_comment_data.push('\u{FFFD}');
            }
            Some(c) => {
                self.current_comment_data.push(c);
            }
        }
    }

    /// §13.2.5.42 Markup declaration open state
    fn state_markup_declaration_open(&mut self) {
        // Check for "--" (comment)
        if self.input.peek() == Some('-') {
            self.input.next_char(); // consume first '-'
            if self.input.peek() == Some('-') {
                self.input.next_char(); // consume second '-'
                self.create_comment("");
                self.state = State::CommentStart;
                return;
            }
            // Not a comment, we consumed one '-', need to handle
            self.emit_error(ParseErrorCode::IncorrectlyOpenedComment);
            self.create_comment("-");
            self.state = State::BogusComment;
            return;
        }

        // Check for "DOCTYPE" (case-insensitive)
        let mut buf = String::new();
        let mut matched_doctype = true;
        for expected in ['D', 'O', 'C', 'T', 'Y', 'P', 'E'] {
            match self.input.peek() {
                Some(c) if c.to_ascii_uppercase() == expected => {
                    self.input.next_char();
                    buf.push(c);
                }
                _ => {
                    matched_doctype = false;
                    break;
                }
            }
        }

        if matched_doctype {
            self.state = State::Doctype;
            return;
        }

        // Check for "[CDATA["
        if buf.is_empty() && self.input.peek() == Some('[') {
            self.input.next_char();
            let mut cdata_buf = String::new();
            let mut matched_cdata = true;
            for expected in ['C', 'D', 'A', 'T', 'A', '['] {
                match self.input.next_char() {
                    Some(c) if c == expected => cdata_buf.push(c),
                    _ => {
                        matched_cdata = false;
                        break;
                    }
                }
            }
            if matched_cdata {
                if self.adjusted_current_node_not_in_html {
                    // In foreign content: enter CDATA section state
                    self.state = State::CdataSection;
                } else {
                    // In HTML content: this is a parse error, treat as bogus comment
                    self.emit_error(ParseErrorCode::CdataInHtmlContent);
                    self.create_comment("[CDATA[");
                    self.state = State::BogusComment;
                }
                return;
            }
            // Not CDATA, fall through to bogus comment
            buf.push('[');
            buf.push_str(&cdata_buf);
        }

        self.emit_error(ParseErrorCode::IncorrectlyOpenedComment);
        self.create_comment("");
        for c in buf.chars() {
            self.current_comment_data.push(c);
        }
        self.state = State::BogusComment;
    }

    // -----------------------------------------------------------------------
    // Comment states (§13.2.5.43–§13.2.5.52)
    // -----------------------------------------------------------------------

    /// §13.2.5.43 Comment start state
    fn state_comment_start(&mut self) {
        match self.input.next_char() {
            Some('-') => self.state = State::CommentStartDash,
            Some('>') => {
                self.emit_error(ParseErrorCode::AbruptClosingOfEmptyComment);
                self.state = State::Data;
                self.emit_current_comment();
            }
            _ => {
                self.input.reconsume();
                self.state = State::Comment;
            }
        }
    }

    /// §13.2.5.44 Comment start dash state
    fn state_comment_start_dash(&mut self) {
        match self.input.next_char() {
            Some('-') => self.state = State::CommentEnd,
            Some('>') => {
                self.emit_error(ParseErrorCode::AbruptClosingOfEmptyComment);
                self.state = State::Data;
                self.emit_current_comment();
            }
            None => {
                self.emit_error(ParseErrorCode::EofInComment);
                self.emit_current_comment();
                self.emit_token(Token::EndOfFile);
            }
            _ => {
                self.current_comment_data.push('-');
                self.input.reconsume();
                self.state = State::Comment;
            }
        }
    }

    /// §13.2.5.45 Comment state
    fn state_comment(&mut self) {
        match self.input.next_char() {
            Some('<') => {
                self.current_comment_data.push('<');
                self.state = State::CommentLessThanSign;
            }
            Some('-') => self.state = State::CommentEndDash,
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.current_comment_data.push('\u{FFFD}');
            }
            None => {
                self.emit_error(ParseErrorCode::EofInComment);
                self.emit_current_comment();
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => self.current_comment_data.push(c),
        }
    }

    /// §13.2.5.46 Comment less-than sign state
    fn state_comment_less_than_sign(&mut self) {
        match self.input.next_char() {
            Some('!') => {
                self.current_comment_data.push('!');
                self.state = State::CommentLessThanSignBang;
            }
            Some('<') => self.current_comment_data.push('<'),
            _ => {
                self.input.reconsume();
                self.state = State::Comment;
            }
        }
    }

    /// §13.2.5.47 Comment less-than sign bang state
    fn state_comment_less_than_sign_bang(&mut self) {
        match self.input.next_char() {
            Some('-') => self.state = State::CommentLessThanSignBangDash,
            _ => {
                self.input.reconsume();
                self.state = State::Comment;
            }
        }
    }

    /// §13.2.5.48 Comment less-than sign bang dash state
    fn state_comment_less_than_sign_bang_dash(&mut self) {
        match self.input.next_char() {
            Some('-') => self.state = State::CommentLessThanSignBangDashDash,
            _ => {
                self.input.reconsume();
                self.state = State::CommentEndDash;
            }
        }
    }

    /// §13.2.5.49 Comment less-than sign bang dash dash state
    fn state_comment_less_than_sign_bang_dash_dash(&mut self) {
        match self.input.next_char() {
            Some('>') | None => {
                self.input.reconsume();
                self.state = State::CommentEnd;
            }
            _ => {
                self.emit_error(ParseErrorCode::NestedComment);
                self.input.reconsume();
                self.state = State::CommentEnd;
            }
        }
    }

    /// §13.2.5.50 Comment end dash state
    fn state_comment_end_dash(&mut self) {
        match self.input.next_char() {
            Some('-') => self.state = State::CommentEnd,
            None => {
                self.emit_error(ParseErrorCode::EofInComment);
                self.emit_current_comment();
                self.emit_token(Token::EndOfFile);
            }
            _ => {
                self.current_comment_data.push('-');
                self.input.reconsume();
                self.state = State::Comment;
            }
        }
    }

    /// §13.2.5.51 Comment end state
    fn state_comment_end(&mut self) {
        match self.input.next_char() {
            Some('>') => {
                self.state = State::Data;
                self.emit_current_comment();
            }
            Some('!') => self.state = State::CommentEndBang,
            Some('-') => self.current_comment_data.push('-'),
            None => {
                self.emit_error(ParseErrorCode::EofInComment);
                self.emit_current_comment();
                self.emit_token(Token::EndOfFile);
            }
            _ => {
                self.current_comment_data.push('-');
                self.current_comment_data.push('-');
                self.input.reconsume();
                self.state = State::Comment;
            }
        }
    }

    /// §13.2.5.52 Comment end bang state
    fn state_comment_end_bang(&mut self) {
        match self.input.next_char() {
            Some('-') => {
                self.current_comment_data.push('-');
                self.current_comment_data.push('-');
                self.current_comment_data.push('!');
                self.state = State::CommentEndDash;
            }
            Some('>') => {
                self.emit_error(ParseErrorCode::IncorrectlyClosedComment);
                self.state = State::Data;
                self.emit_current_comment();
            }
            None => {
                self.emit_error(ParseErrorCode::EofInComment);
                self.emit_current_comment();
                self.emit_token(Token::EndOfFile);
            }
            _ => {
                self.current_comment_data.push('-');
                self.current_comment_data.push('-');
                self.current_comment_data.push('!');
                self.input.reconsume();
                self.state = State::Comment;
            }
        }
    }

    // -----------------------------------------------------------------------
    // DOCTYPE states (§13.2.5.53–§13.2.5.68)
    // -----------------------------------------------------------------------

    /// §13.2.5.53 DOCTYPE state
    fn state_doctype(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = State::BeforeDoctypeName;
            }
            Some('>') => {
                self.input.reconsume();
                self.state = State::BeforeDoctypeName;
            }
            None => {
                self.emit_error(ParseErrorCode::EofInDoctype);
                self.create_doctype();
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            _ => {
                self.emit_error(ParseErrorCode::MissingWhitespaceBeforeDoctypeName);
                self.input.reconsume();
                self.state = State::BeforeDoctypeName;
            }
        }
    }

    /// §13.2.5.54 Before DOCTYPE name state
    fn state_before_doctype_name(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                // Ignore whitespace
            }
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.create_doctype();
                self.current_doctype_name = Some("\u{FFFD}".to_string());
                self.state = State::DoctypeName;
            }
            Some('>') => {
                self.emit_error(ParseErrorCode::MissingDoctypeName);
                self.create_doctype();
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.state = State::Data;
            }
            None => {
                self.emit_error(ParseErrorCode::EofInDoctype);
                self.create_doctype();
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => {
                self.create_doctype();
                self.current_doctype_name = Some(c.to_ascii_lowercase().to_string());
                self.state = State::DoctypeName;
            }
        }
    }

    /// §13.2.5.55 DOCTYPE name state
    fn state_doctype_name(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = State::AfterDoctypeName;
            }
            Some('>') => {
                self.state = State::Data;
                self.emit_current_doctype();
            }
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                if let Some(ref mut name) = self.current_doctype_name {
                    name.push('\u{FFFD}');
                }
            }
            None => {
                self.emit_error(ParseErrorCode::EofInDoctype);
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => {
                if let Some(ref mut name) = self.current_doctype_name {
                    name.push(c.to_ascii_lowercase());
                }
            }
        }
    }

    /// §13.2.5.56 After DOCTYPE name state
    fn state_after_doctype_name(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                // Ignore
            }
            Some('>') => {
                self.state = State::Data;
                self.emit_current_doctype();
            }
            None => {
                self.emit_error(ParseErrorCode::EofInDoctype);
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => {
                // Check for PUBLIC or SYSTEM
                let upper = c.to_ascii_uppercase();
                if upper == 'P' {
                    // Try to match "UBLIC"
                    let mut matched = true;
                    for expected in ['U', 'B', 'L', 'I', 'C'] {
                        match self.input.next_char() {
                            Some(ch) if ch.to_ascii_uppercase() == expected => {}
                            _ => {
                                matched = false;
                                break;
                            }
                        }
                    }
                    if matched {
                        self.state = State::AfterDoctypePublicKeyword;
                        return;
                    }
                } else if upper == 'S' {
                    let mut matched = true;
                    for expected in ['Y', 'S', 'T', 'E', 'M'] {
                        match self.input.next_char() {
                            Some(ch) if ch.to_ascii_uppercase() == expected => {}
                            _ => {
                                matched = false;
                                break;
                            }
                        }
                    }
                    if matched {
                        self.state = State::AfterDoctypeSystemKeyword;
                        return;
                    }
                }
                self.emit_error(ParseErrorCode::InvalidCharacterSequenceAfterDoctypeName);
                self.current_doctype_force_quirks = true;
                self.input.reconsume();
                self.state = State::BogusDoctype;
            }
        }
    }

    /// §13.2.5.57 After DOCTYPE public keyword state
    fn state_after_doctype_public_keyword(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = State::BeforeDoctypePublicIdentifier;
            }
            Some('"') => {
                self.emit_error(ParseErrorCode::MissingWhitespaceAfterDoctypePublicKeyword);
                self.current_doctype_public_id = Some(String::new());
                self.state = State::DoctypePublicIdentifierDoubleQuoted;
            }
            Some('\'') => {
                self.emit_error(ParseErrorCode::MissingWhitespaceAfterDoctypePublicKeyword);
                self.current_doctype_public_id = Some(String::new());
                self.state = State::DoctypePublicIdentifierSingleQuoted;
            }
            Some('>') => {
                self.emit_error(ParseErrorCode::MissingDoctypePublicIdentifier);
                self.current_doctype_force_quirks = true;
                self.state = State::Data;
                self.emit_current_doctype();
            }
            None => {
                self.emit_error(ParseErrorCode::EofInDoctype);
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            _ => {
                self.emit_error(ParseErrorCode::MissingQuoteBeforeDoctypePublicIdentifier);
                self.current_doctype_force_quirks = true;
                self.input.reconsume();
                self.state = State::BogusDoctype;
            }
        }
    }

    /// §13.2.5.58 Before DOCTYPE public identifier state
    fn state_before_doctype_public_identifier(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {}
            Some('"') => {
                self.current_doctype_public_id = Some(String::new());
                self.state = State::DoctypePublicIdentifierDoubleQuoted;
            }
            Some('\'') => {
                self.current_doctype_public_id = Some(String::new());
                self.state = State::DoctypePublicIdentifierSingleQuoted;
            }
            Some('>') => {
                self.emit_error(ParseErrorCode::MissingDoctypePublicIdentifier);
                self.current_doctype_force_quirks = true;
                self.state = State::Data;
                self.emit_current_doctype();
            }
            None => {
                self.emit_error(ParseErrorCode::EofInDoctype);
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            _ => {
                self.emit_error(ParseErrorCode::MissingQuoteBeforeDoctypePublicIdentifier);
                self.current_doctype_force_quirks = true;
                self.input.reconsume();
                self.state = State::BogusDoctype;
            }
        }
    }

    /// §13.2.5.59 DOCTYPE public identifier (double-quoted) state
    fn state_doctype_public_identifier_double_quoted(&mut self) {
        match self.input.next_char() {
            Some('"') => self.state = State::AfterDoctypePublicIdentifier,
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                if let Some(ref mut id) = self.current_doctype_public_id {
                    id.push('\u{FFFD}');
                }
            }
            Some('>') => {
                self.emit_error(ParseErrorCode::AbruptDoctypePublicIdentifier);
                self.current_doctype_force_quirks = true;
                self.state = State::Data;
                self.emit_current_doctype();
            }
            None => {
                self.emit_error(ParseErrorCode::EofInDoctype);
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => {
                if let Some(ref mut id) = self.current_doctype_public_id {
                    id.push(c);
                }
            }
        }
    }

    /// §13.2.5.60 DOCTYPE public identifier (single-quoted) state
    fn state_doctype_public_identifier_single_quoted(&mut self) {
        match self.input.next_char() {
            Some('\'') => self.state = State::AfterDoctypePublicIdentifier,
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                if let Some(ref mut id) = self.current_doctype_public_id {
                    id.push('\u{FFFD}');
                }
            }
            Some('>') => {
                self.emit_error(ParseErrorCode::AbruptDoctypePublicIdentifier);
                self.current_doctype_force_quirks = true;
                self.state = State::Data;
                self.emit_current_doctype();
            }
            None => {
                self.emit_error(ParseErrorCode::EofInDoctype);
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => {
                if let Some(ref mut id) = self.current_doctype_public_id {
                    id.push(c);
                }
            }
        }
    }

    /// §13.2.5.61 After DOCTYPE public identifier state
    fn state_after_doctype_public_identifier(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = State::BetweenDoctypePublicAndSystemIdentifiers;
            }
            Some('>') => {
                self.state = State::Data;
                self.emit_current_doctype();
            }
            Some('"') => {
                self.emit_error(
                    ParseErrorCode::MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers,
                );
                self.current_doctype_system_id = Some(String::new());
                self.state = State::DoctypeSystemIdentifierDoubleQuoted;
            }
            Some('\'') => {
                self.emit_error(
                    ParseErrorCode::MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers,
                );
                self.current_doctype_system_id = Some(String::new());
                self.state = State::DoctypeSystemIdentifierSingleQuoted;
            }
            None => {
                self.emit_error(ParseErrorCode::EofInDoctype);
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            _ => {
                self.emit_error(ParseErrorCode::MissingQuoteBeforeDoctypeSystemIdentifier);
                self.current_doctype_force_quirks = true;
                self.input.reconsume();
                self.state = State::BogusDoctype;
            }
        }
    }

    /// §13.2.5.62 Between DOCTYPE public and system identifiers state
    fn state_between_doctype_public_and_system_identifiers(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {}
            Some('>') => {
                self.state = State::Data;
                self.emit_current_doctype();
            }
            Some('"') => {
                self.current_doctype_system_id = Some(String::new());
                self.state = State::DoctypeSystemIdentifierDoubleQuoted;
            }
            Some('\'') => {
                self.current_doctype_system_id = Some(String::new());
                self.state = State::DoctypeSystemIdentifierSingleQuoted;
            }
            None => {
                self.emit_error(ParseErrorCode::EofInDoctype);
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            _ => {
                self.emit_error(ParseErrorCode::MissingQuoteBeforeDoctypeSystemIdentifier);
                self.current_doctype_force_quirks = true;
                self.input.reconsume();
                self.state = State::BogusDoctype;
            }
        }
    }

    /// §13.2.5.63 After DOCTYPE system keyword state
    fn state_after_doctype_system_keyword(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = State::BeforeDoctypeSystemIdentifier;
            }
            Some('"') => {
                self.emit_error(ParseErrorCode::MissingWhitespaceAfterDoctypeSystemKeyword);
                self.current_doctype_system_id = Some(String::new());
                self.state = State::DoctypeSystemIdentifierDoubleQuoted;
            }
            Some('\'') => {
                self.emit_error(ParseErrorCode::MissingWhitespaceAfterDoctypeSystemKeyword);
                self.current_doctype_system_id = Some(String::new());
                self.state = State::DoctypeSystemIdentifierSingleQuoted;
            }
            Some('>') => {
                self.emit_error(ParseErrorCode::MissingDoctypeSystemIdentifier);
                self.current_doctype_force_quirks = true;
                self.state = State::Data;
                self.emit_current_doctype();
            }
            None => {
                self.emit_error(ParseErrorCode::EofInDoctype);
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            _ => {
                self.emit_error(ParseErrorCode::MissingQuoteBeforeDoctypeSystemIdentifier);
                self.current_doctype_force_quirks = true;
                self.input.reconsume();
                self.state = State::BogusDoctype;
            }
        }
    }

    /// §13.2.5.64 Before DOCTYPE system identifier state
    fn state_before_doctype_system_identifier(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {}
            Some('"') => {
                self.current_doctype_system_id = Some(String::new());
                self.state = State::DoctypeSystemIdentifierDoubleQuoted;
            }
            Some('\'') => {
                self.current_doctype_system_id = Some(String::new());
                self.state = State::DoctypeSystemIdentifierSingleQuoted;
            }
            Some('>') => {
                self.emit_error(ParseErrorCode::MissingDoctypeSystemIdentifier);
                self.current_doctype_force_quirks = true;
                self.state = State::Data;
                self.emit_current_doctype();
            }
            None => {
                self.emit_error(ParseErrorCode::EofInDoctype);
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            _ => {
                self.emit_error(ParseErrorCode::MissingQuoteBeforeDoctypeSystemIdentifier);
                self.current_doctype_force_quirks = true;
                self.input.reconsume();
                self.state = State::BogusDoctype;
            }
        }
    }

    /// §13.2.5.65 DOCTYPE system identifier (double-quoted) state
    fn state_doctype_system_identifier_double_quoted(&mut self) {
        match self.input.next_char() {
            Some('"') => self.state = State::AfterDoctypeSystemIdentifier,
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                if let Some(ref mut id) = self.current_doctype_system_id {
                    id.push('\u{FFFD}');
                }
            }
            Some('>') => {
                self.emit_error(ParseErrorCode::AbruptDoctypeSystemIdentifier);
                self.current_doctype_force_quirks = true;
                self.state = State::Data;
                self.emit_current_doctype();
            }
            None => {
                self.emit_error(ParseErrorCode::EofInDoctype);
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => {
                if let Some(ref mut id) = self.current_doctype_system_id {
                    id.push(c);
                }
            }
        }
    }

    /// §13.2.5.66 DOCTYPE system identifier (single-quoted) state
    fn state_doctype_system_identifier_single_quoted(&mut self) {
        match self.input.next_char() {
            Some('\'') => self.state = State::AfterDoctypeSystemIdentifier,
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                if let Some(ref mut id) = self.current_doctype_system_id {
                    id.push('\u{FFFD}');
                }
            }
            Some('>') => {
                self.emit_error(ParseErrorCode::AbruptDoctypeSystemIdentifier);
                self.current_doctype_force_quirks = true;
                self.state = State::Data;
                self.emit_current_doctype();
            }
            None => {
                self.emit_error(ParseErrorCode::EofInDoctype);
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => {
                if let Some(ref mut id) = self.current_doctype_system_id {
                    id.push(c);
                }
            }
        }
    }

    /// §13.2.5.67 After DOCTYPE system identifier state
    fn state_after_doctype_system_identifier(&mut self) {
        match self.input.next_char() {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {}
            Some('>') => {
                self.state = State::Data;
                self.emit_current_doctype();
            }
            None => {
                self.emit_error(ParseErrorCode::EofInDoctype);
                self.current_doctype_force_quirks = true;
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            _ => {
                self.emit_error(ParseErrorCode::UnexpectedCharacterAfterDoctypeSystemIdentifier);
                self.input.reconsume();
                self.state = State::BogusDoctype;
            }
        }
    }

    /// §13.2.5.68 Bogus DOCTYPE state
    fn state_bogus_doctype(&mut self) {
        match self.input.next_char() {
            Some('>') => {
                self.state = State::Data;
                self.emit_current_doctype();
            }
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
            }
            None => {
                self.emit_current_doctype();
                self.emit_token(Token::EndOfFile);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // RCDATA states (§13.2.5.2, §13.2.5.9–§13.2.5.11)
    // -----------------------------------------------------------------------

    fn state_rcdata(&mut self) {
        match self.input.next_char() {
            Some('&') => {
                self.return_state = State::RcData;
                self.state = State::CharacterReference;
            }
            Some('<') => self.state = State::RcDataLessThanSign,
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.emit_char('\u{FFFD}');
            }
            None => self.emit_token(Token::EndOfFile),
            Some(c) => self.emit_char(c),
        }
    }

    fn state_rcdata_less_than_sign(&mut self) {
        match self.input.next_char() {
            Some('/') => {
                self.temp_buffer.clear();
                self.state = State::RcDataEndTagOpen;
            }
            _ => {
                self.emit_char('<');
                self.input.reconsume();
                self.state = State::RcData;
            }
        }
    }

    fn state_rcdata_end_tag_open(&mut self) {
        match self.input.next_char() {
            Some(c) if c.is_ascii_alphabetic() => {
                self.create_end_tag();
                self.input.reconsume();
                self.state = State::RcDataEndTagName;
            }
            _ => {
                self.emit_char('<');
                self.emit_char('/');
                self.input.reconsume();
                self.state = State::RcData;
            }
        }
    }

    fn state_rcdata_end_tag_name(&mut self) {
        match self.input.next_char() {
            Some(c @ '\t') | Some(c @ '\n') | Some(c @ '\x0C') | Some(c @ ' ') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::BeforeAttributeName;
                } else {
                    self.emit_end_tag_name_chars_and_reconsume(c);
                    self.state = State::RcData;
                }
            }
            Some('/') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::SelfClosingStartTag;
                } else {
                    self.emit_end_tag_name_chars_and_reconsume('/');
                    self.state = State::RcData;
                }
            }
            Some('>') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::Data;
                    self.emit_current_tag();
                } else {
                    self.emit_end_tag_name_chars_and_reconsume('>');
                    self.state = State::RcData;
                }
            }
            Some(c) if c.is_ascii_alphabetic() => {
                self.current_tag_name.push(c.to_ascii_lowercase());
                self.temp_buffer.push(c);
            }
            _ => {
                self.input.reconsume();
                self.emit_end_tag_name_chars_no_current();
                self.state = State::RcData;
            }
        }
    }

    // -----------------------------------------------------------------------
    // RAWTEXT states (§13.2.5.3, §13.2.5.12–§13.2.5.14)
    // -----------------------------------------------------------------------

    fn state_rawtext(&mut self) {
        match self.input.next_char() {
            Some('<') => self.state = State::RawTextLessThanSign,
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.emit_char('\u{FFFD}');
            }
            None => self.emit_token(Token::EndOfFile),
            Some(c) => self.emit_char(c),
        }
    }

    fn state_rawtext_less_than_sign(&mut self) {
        match self.input.next_char() {
            Some('/') => {
                self.temp_buffer.clear();
                self.state = State::RawTextEndTagOpen;
            }
            _ => {
                self.emit_char('<');
                self.input.reconsume();
                self.state = State::RawText;
            }
        }
    }

    fn state_rawtext_end_tag_open(&mut self) {
        match self.input.next_char() {
            Some(c) if c.is_ascii_alphabetic() => {
                self.create_end_tag();
                self.input.reconsume();
                self.state = State::RawTextEndTagName;
            }
            _ => {
                self.emit_char('<');
                self.emit_char('/');
                self.input.reconsume();
                self.state = State::RawText;
            }
        }
    }

    fn state_rawtext_end_tag_name(&mut self) {
        match self.input.next_char() {
            Some(c @ '\t') | Some(c @ '\n') | Some(c @ '\x0C') | Some(c @ ' ') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::BeforeAttributeName;
                } else {
                    self.emit_end_tag_name_chars_and_reconsume(c);
                    self.state = State::RawText;
                }
            }
            Some('/') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::SelfClosingStartTag;
                } else {
                    self.emit_end_tag_name_chars_and_reconsume('/');
                    self.state = State::RawText;
                }
            }
            Some('>') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::Data;
                    self.emit_current_tag();
                } else {
                    self.emit_end_tag_name_chars_and_reconsume('>');
                    self.state = State::RawText;
                }
            }
            Some(c) if c.is_ascii_alphabetic() => {
                self.current_tag_name.push(c.to_ascii_lowercase());
                self.temp_buffer.push(c);
            }
            _ => {
                self.input.reconsume();
                self.emit_end_tag_name_chars_no_current();
                self.state = State::RawText;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Script data states (§13.2.5.4, §13.2.5.15–§13.2.5.31)
    // -----------------------------------------------------------------------

    fn state_script_data(&mut self) {
        match self.input.next_char() {
            Some('<') => self.state = State::ScriptDataLessThanSign,
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.emit_char('\u{FFFD}');
            }
            None => self.emit_token(Token::EndOfFile),
            Some(c) => self.emit_char(c),
        }
    }

    fn state_script_data_less_than_sign(&mut self) {
        match self.input.next_char() {
            Some('/') => {
                self.temp_buffer.clear();
                self.state = State::ScriptDataEndTagOpen;
            }
            Some('!') => {
                self.state = State::ScriptDataEscapeStart;
                self.emit_char('<');
                self.emit_char('!');
            }
            _ => {
                self.emit_char('<');
                self.input.reconsume();
                self.state = State::ScriptData;
            }
        }
    }

    fn state_script_data_end_tag_open(&mut self) {
        match self.input.next_char() {
            Some(c) if c.is_ascii_alphabetic() => {
                self.create_end_tag();
                self.input.reconsume();
                self.state = State::ScriptDataEndTagName;
            }
            _ => {
                self.emit_char('<');
                self.emit_char('/');
                self.input.reconsume();
                self.state = State::ScriptData;
            }
        }
    }

    fn state_script_data_end_tag_name(&mut self) {
        match self.input.next_char() {
            Some(c @ '\t') | Some(c @ '\n') | Some(c @ '\x0C') | Some(c @ ' ') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::BeforeAttributeName;
                } else {
                    self.emit_end_tag_name_chars_and_reconsume(c);
                    self.state = State::ScriptData;
                }
            }
            Some('/') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::SelfClosingStartTag;
                } else {
                    self.emit_end_tag_name_chars_and_reconsume('/');
                    self.state = State::ScriptData;
                }
            }
            Some('>') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::Data;
                    self.emit_current_tag();
                } else {
                    self.emit_end_tag_name_chars_and_reconsume('>');
                    self.state = State::ScriptData;
                }
            }
            Some(c) if c.is_ascii_alphabetic() => {
                self.current_tag_name.push(c.to_ascii_lowercase());
                self.temp_buffer.push(c);
            }
            _ => {
                self.input.reconsume();
                self.emit_end_tag_name_chars_no_current();
                self.state = State::ScriptData;
            }
        }
    }

    fn state_script_data_escape_start(&mut self) {
        match self.input.next_char() {
            Some('-') => {
                self.state = State::ScriptDataEscapeStartDash;
                self.emit_char('-');
            }
            _ => {
                self.input.reconsume();
                self.state = State::ScriptData;
            }
        }
    }

    fn state_script_data_escape_start_dash(&mut self) {
        match self.input.next_char() {
            Some('-') => {
                self.state = State::ScriptDataEscapedDashDash;
                self.emit_char('-');
            }
            _ => {
                self.input.reconsume();
                self.state = State::ScriptData;
            }
        }
    }

    fn state_script_data_escaped(&mut self) {
        match self.input.next_char() {
            Some('-') => {
                self.state = State::ScriptDataEscapedDash;
                self.emit_char('-');
            }
            Some('<') => self.state = State::ScriptDataEscapedLessThanSign,
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.emit_char('\u{FFFD}');
            }
            None => {
                self.emit_error(ParseErrorCode::EofInScriptHtmlCommentLikeText);
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => self.emit_char(c),
        }
    }

    fn state_script_data_escaped_dash(&mut self) {
        match self.input.next_char() {
            Some('-') => {
                self.state = State::ScriptDataEscapedDashDash;
                self.emit_char('-');
            }
            Some('<') => self.state = State::ScriptDataEscapedLessThanSign,
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.state = State::ScriptDataEscaped;
                self.emit_char('\u{FFFD}');
            }
            None => {
                self.emit_error(ParseErrorCode::EofInScriptHtmlCommentLikeText);
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => {
                self.state = State::ScriptDataEscaped;
                self.emit_char(c);
            }
        }
    }

    fn state_script_data_escaped_dash_dash(&mut self) {
        match self.input.next_char() {
            Some('-') => self.emit_char('-'),
            Some('<') => self.state = State::ScriptDataEscapedLessThanSign,
            Some('>') => {
                self.state = State::ScriptData;
                self.emit_char('>');
            }
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.state = State::ScriptDataEscaped;
                self.emit_char('\u{FFFD}');
            }
            None => {
                self.emit_error(ParseErrorCode::EofInScriptHtmlCommentLikeText);
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => {
                self.state = State::ScriptDataEscaped;
                self.emit_char(c);
            }
        }
    }

    fn state_script_data_escaped_less_than_sign(&mut self) {
        match self.input.next_char() {
            Some('/') => {
                self.temp_buffer.clear();
                self.state = State::ScriptDataEscapedEndTagOpen;
            }
            Some(c) if c.is_ascii_alphabetic() => {
                self.temp_buffer.clear();
                self.emit_char('<');
                self.input.reconsume();
                self.state = State::ScriptDataDoubleEscapeStart;
            }
            _ => {
                self.emit_char('<');
                self.input.reconsume();
                self.state = State::ScriptDataEscaped;
            }
        }
    }

    fn state_script_data_escaped_end_tag_open(&mut self) {
        match self.input.next_char() {
            Some(c) if c.is_ascii_alphabetic() => {
                self.create_end_tag();
                self.input.reconsume();
                self.state = State::ScriptDataEscapedEndTagName;
            }
            _ => {
                self.emit_char('<');
                self.emit_char('/');
                self.input.reconsume();
                self.state = State::ScriptDataEscaped;
            }
        }
    }

    fn state_script_data_escaped_end_tag_name(&mut self) {
        match self.input.next_char() {
            Some(c @ '\t') | Some(c @ '\n') | Some(c @ '\x0C') | Some(c @ ' ') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::BeforeAttributeName;
                } else {
                    self.emit_end_tag_name_chars_and_reconsume(c);
                    self.state = State::ScriptDataEscaped;
                }
            }
            Some('/') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::SelfClosingStartTag;
                } else {
                    self.emit_end_tag_name_chars_and_reconsume('/');
                    self.state = State::ScriptDataEscaped;
                }
            }
            Some('>') => {
                if self.is_appropriate_end_tag() {
                    self.state = State::Data;
                    self.emit_current_tag();
                } else {
                    self.emit_end_tag_name_chars_and_reconsume('>');
                    self.state = State::ScriptDataEscaped;
                }
            }
            Some(c) if c.is_ascii_alphabetic() => {
                self.current_tag_name.push(c.to_ascii_lowercase());
                self.temp_buffer.push(c);
            }
            _ => {
                self.input.reconsume();
                self.emit_end_tag_name_chars_no_current();
                self.state = State::ScriptDataEscaped;
            }
        }
    }

    fn state_script_data_double_escape_start(&mut self) {
        match self.input.next_char() {
            Some(c @ '\t') | Some(c @ '\n') | Some(c @ '\x0C') | Some(c @ ' ') | Some(c @ '/')
            | Some(c @ '>') => {
                if self.temp_buffer == "script" {
                    self.state = State::ScriptDataDoubleEscaped;
                } else {
                    self.state = State::ScriptDataEscaped;
                }
                self.emit_char(c);
            }
            Some(c) if c.is_ascii_alphabetic() => {
                self.temp_buffer.push(c.to_ascii_lowercase());
                self.emit_char(c);
            }
            _ => {
                self.input.reconsume();
                self.state = State::ScriptDataEscaped;
            }
        }
    }

    fn state_script_data_double_escaped(&mut self) {
        match self.input.next_char() {
            Some('-') => {
                self.state = State::ScriptDataDoubleEscapedDash;
                self.emit_char('-');
            }
            Some('<') => {
                self.state = State::ScriptDataDoubleEscapedLessThanSign;
                self.emit_char('<');
            }
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.emit_char('\u{FFFD}');
            }
            None => {
                self.emit_error(ParseErrorCode::EofInScriptHtmlCommentLikeText);
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => self.emit_char(c),
        }
    }

    fn state_script_data_double_escaped_dash(&mut self) {
        match self.input.next_char() {
            Some('-') => {
                self.state = State::ScriptDataDoubleEscapedDashDash;
                self.emit_char('-');
            }
            Some('<') => {
                self.state = State::ScriptDataDoubleEscapedLessThanSign;
                self.emit_char('<');
            }
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.state = State::ScriptDataDoubleEscaped;
                self.emit_char('\u{FFFD}');
            }
            None => {
                self.emit_error(ParseErrorCode::EofInScriptHtmlCommentLikeText);
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => {
                self.state = State::ScriptDataDoubleEscaped;
                self.emit_char(c);
            }
        }
    }

    fn state_script_data_double_escaped_dash_dash(&mut self) {
        match self.input.next_char() {
            Some('-') => self.emit_char('-'),
            Some('<') => {
                self.state = State::ScriptDataDoubleEscapedLessThanSign;
                self.emit_char('<');
            }
            Some('>') => {
                self.state = State::ScriptData;
                self.emit_char('>');
            }
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.state = State::ScriptDataDoubleEscaped;
                self.emit_char('\u{FFFD}');
            }
            None => {
                self.emit_error(ParseErrorCode::EofInScriptHtmlCommentLikeText);
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => {
                self.state = State::ScriptDataDoubleEscaped;
                self.emit_char(c);
            }
        }
    }

    fn state_script_data_double_escaped_less_than_sign(&mut self) {
        match self.input.next_char() {
            Some('/') => {
                self.temp_buffer.clear();
                self.state = State::ScriptDataDoubleEscapeEnd;
                self.emit_char('/');
            }
            _ => {
                self.input.reconsume();
                self.state = State::ScriptDataDoubleEscaped;
            }
        }
    }

    fn state_script_data_double_escape_end(&mut self) {
        match self.input.next_char() {
            Some(c @ '\t') | Some(c @ '\n') | Some(c @ '\x0C') | Some(c @ ' ') | Some(c @ '/')
            | Some(c @ '>') => {
                if self.temp_buffer == "script" {
                    self.state = State::ScriptDataEscaped;
                } else {
                    self.state = State::ScriptDataDoubleEscaped;
                }
                self.emit_char(c);
            }
            Some(c) if c.is_ascii_alphabetic() => {
                self.temp_buffer.push(c.to_ascii_lowercase());
                self.emit_char(c);
            }
            _ => {
                self.input.reconsume();
                self.state = State::ScriptDataDoubleEscaped;
            }
        }
    }

    // -----------------------------------------------------------------------
    // PLAINTEXT state (§13.2.5.5)
    // -----------------------------------------------------------------------

    fn state_plaintext(&mut self) {
        match self.input.next_char() {
            Some('\0') => {
                self.emit_error(ParseErrorCode::UnexpectedNullCharacter);
                self.emit_char('\u{FFFD}');
            }
            None => self.emit_token(Token::EndOfFile),
            Some(c) => self.emit_char(c),
        }
    }

    // -----------------------------------------------------------------------
    // CDATA section states (§13.2.5.69–§13.2.5.71)
    // -----------------------------------------------------------------------

    fn state_cdata_section(&mut self) {
        match self.input.next_char() {
            Some(']') => self.state = State::CdataSectionBracket,
            None => {
                self.emit_error(ParseErrorCode::EofInCdata);
                self.emit_token(Token::EndOfFile);
            }
            Some(c) => self.emit_char(c),
        }
    }

    fn state_cdata_section_bracket(&mut self) {
        match self.input.next_char() {
            Some(']') => self.state = State::CdataSectionEnd,
            _ => {
                self.emit_char(']');
                self.input.reconsume();
                self.state = State::CdataSection;
            }
        }
    }

    fn state_cdata_section_end(&mut self) {
        match self.input.next_char() {
            Some(']') => self.emit_char(']'),
            Some('>') => self.state = State::Data,
            _ => {
                self.emit_char(']');
                self.emit_char(']');
                self.input.reconsume();
                self.state = State::CdataSection;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Character reference states (§13.2.5.72–§13.2.5.80)
    // -----------------------------------------------------------------------

    /// §13.2.5.72 Character reference state
    fn state_character_reference(&mut self) {
        self.temp_buffer.clear();
        self.temp_buffer.push('&');

        match self.input.next_char() {
            Some(c) if c.is_ascii_alphanumeric() => {
                self.input.reconsume();
                self.state = State::NamedCharacterReference;
            }
            Some('#') => {
                self.temp_buffer.push('#');
                self.state = State::NumericCharacterReference;
            }
            _ => {
                self.flush_code_points_consumed_as_character_reference();
                self.input.reconsume();
                self.state = self.return_state;
            }
        }
    }

    /// §13.2.5.73 Named character reference state
    fn state_named_character_reference(&mut self) {
        // Consume characters and try to match against the entity table.
        // We need to find the longest match.
        let mut consumed = String::new();
        let mut best_match_len: usize = 0;
        let mut best_match_chars: &[char] = &[];
        let table = entities::NAMED_CHAR_REFS;

        loop {
            match self.input.next_char() {
                Some(c) if c.is_ascii_alphanumeric() || c == ';' => {
                    consumed.push(c);

                    // Check for exact match
                    if let Ok(idx) = table.binary_search_by_key(&consumed.as_str(), |&(k, _)| k) {
                        best_match_len = consumed.len();
                        best_match_chars = table[idx].1;
                        if c == ';' {
                            break;
                        }
                    } else {
                        // Check if any entries still start with this prefix
                        let has_prefix = table
                            .binary_search_by(|&(k, _)| {
                                if k.starts_with(&consumed) {
                                    std::cmp::Ordering::Equal
                                } else {
                                    k.cmp(consumed.as_str())
                                }
                            })
                            .is_ok();

                        if !has_prefix || c == ';' {
                            break;
                        }
                    }
                }
                _ => {
                    self.input.reconsume();
                    break;
                }
            }
        }

        if best_match_len > 0 {
            let matched_name = &consumed[..best_match_len];
            let extra = &consumed[best_match_len..];
            let ends_with_semicolon = matched_name.ends_with(';');

            let in_attribute = matches!(
                self.return_state,
                State::AttributeValueDoubleQuoted
                    | State::AttributeValueSingleQuoted
                    | State::AttributeValueUnquoted
            );

            if !ends_with_semicolon && in_attribute {
                // Check what comes after the matched portion.
                // "after" is either the first char of `extra`, or the next input char.
                let next_after = extra.chars().next().or_else(|| self.input.peek());
                if next_after == Some('=') || next_after.is_some_and(|c| c.is_ascii_alphanumeric())
                {
                    // Not a character reference — flush everything as text
                    self.temp_buffer.push_str(&consumed);
                    self.flush_code_points_consumed_as_character_reference();
                    self.state = self.return_state;
                    return;
                }
            }

            if !ends_with_semicolon {
                self.emit_error(ParseErrorCode::MissingSemicolonAfterCharacterReference);
            }

            // Replace temp_buffer with the matched replacement characters
            self.temp_buffer.clear();
            for &c in best_match_chars {
                self.temp_buffer.push(c);
            }
            self.flush_code_points_consumed_as_character_reference();

            // Emit extra characters that were consumed but not part of the match
            let in_attribute = matches!(
                self.return_state,
                State::AttributeValueDoubleQuoted
                    | State::AttributeValueSingleQuoted
                    | State::AttributeValueUnquoted
            );
            if in_attribute {
                self.current_attr_value.push_str(extra);
            } else {
                for c in extra.chars() {
                    self.emit_char(c);
                }
            }

            self.state = self.return_state;
        } else {
            // No match found
            self.temp_buffer.push_str(&consumed);
            self.flush_code_points_consumed_as_character_reference();
            self.state = State::AmbiguousAmpersand;
        }
    }

    /// §13.2.5.74 Ambiguous ampersand state
    fn state_ambiguous_ampersand(&mut self) {
        match self.input.next_char() {
            Some(c) if c.is_ascii_alphanumeric() => {
                let in_attribute = matches!(
                    self.return_state,
                    State::AttributeValueDoubleQuoted
                        | State::AttributeValueSingleQuoted
                        | State::AttributeValueUnquoted
                );
                if in_attribute {
                    self.current_attr_value.push(c);
                } else {
                    self.emit_char(c);
                }
            }
            Some(';') => {
                self.emit_error(ParseErrorCode::UnknownNamedCharacterReference);
                self.input.reconsume();
                self.state = self.return_state;
            }
            _ => {
                self.input.reconsume();
                self.state = self.return_state;
            }
        }
    }

    /// §13.2.5.75 Numeric character reference state
    fn state_numeric_character_reference(&mut self) {
        self.character_reference_code = 0;
        match self.input.next_char() {
            Some(c @ 'x') | Some(c @ 'X') => {
                self.temp_buffer.push(c);
                self.state = State::HexadecimalCharacterReferenceStart;
            }
            _ => {
                self.input.reconsume();
                self.state = State::DecimalCharacterReferenceStart;
            }
        }
    }

    /// §13.2.5.76 Hexadecimal character reference start state
    fn state_hexadecimal_character_reference_start(&mut self) {
        match self.input.next_char() {
            Some(c) if c.is_ascii_hexdigit() => {
                self.input.reconsume();
                self.state = State::HexadecimalCharacterReference;
            }
            _ => {
                self.emit_error(ParseErrorCode::AbsenceOfDigitsInNumericCharacterReference);
                self.flush_code_points_consumed_as_character_reference();
                self.input.reconsume();
                self.state = self.return_state;
            }
        }
    }

    /// §13.2.5.77 Decimal character reference start state
    fn state_decimal_character_reference_start(&mut self) {
        match self.input.next_char() {
            Some(c) if c.is_ascii_digit() => {
                self.input.reconsume();
                self.state = State::DecimalCharacterReference;
            }
            _ => {
                self.emit_error(ParseErrorCode::AbsenceOfDigitsInNumericCharacterReference);
                self.flush_code_points_consumed_as_character_reference();
                self.input.reconsume();
                self.state = self.return_state;
            }
        }
    }

    /// §13.2.5.78 Hexadecimal character reference state
    fn state_hexadecimal_character_reference(&mut self) {
        match self.input.next_char() {
            Some(c) if c.is_ascii_hexdigit() => {
                self.character_reference_code = self
                    .character_reference_code
                    .wrapping_mul(16)
                    .wrapping_add(c.to_digit(16).unwrap());
                // Cap at a value beyond Unicode range to detect overflow
                if self.character_reference_code > 0x10FFFF {
                    self.character_reference_code = 0x110000;
                }
            }
            Some(';') => {
                self.state = State::NumericCharacterReferenceEnd;
            }
            _ => {
                self.emit_error(ParseErrorCode::MissingSemicolonAfterCharacterReference);
                self.input.reconsume();
                self.state = State::NumericCharacterReferenceEnd;
            }
        }
    }

    /// §13.2.5.79 Decimal character reference state
    fn state_decimal_character_reference(&mut self) {
        match self.input.next_char() {
            Some(c) if c.is_ascii_digit() => {
                self.character_reference_code = self
                    .character_reference_code
                    .wrapping_mul(10)
                    .wrapping_add(c.to_digit(10).unwrap());
                if self.character_reference_code > 0x10FFFF {
                    self.character_reference_code = 0x110000;
                }
            }
            Some(';') => {
                self.state = State::NumericCharacterReferenceEnd;
            }
            _ => {
                self.emit_error(ParseErrorCode::MissingSemicolonAfterCharacterReference);
                self.input.reconsume();
                self.state = State::NumericCharacterReferenceEnd;
            }
        }
    }

    /// §13.2.5.80 Numeric character reference end state
    fn state_numeric_character_reference_end(&mut self) {
        let code = self.character_reference_code;

        let c = if code == 0 {
            self.emit_error(ParseErrorCode::NullCharacterReference);
            '\u{FFFD}'
        } else if code > 0x10FFFF {
            self.emit_error(ParseErrorCode::CharacterReferenceOutsideUnicodeRange);
            '\u{FFFD}'
        } else if is_surrogate_codepoint(code) {
            self.emit_error(ParseErrorCode::SurrogateCharacterReference);
            '\u{FFFD}'
        } else if is_noncharacter_codepoint(code) {
            self.emit_error(ParseErrorCode::NoncharacterCharacterReference);
            // Return the character as-is per spec
            char::from_u32(code).unwrap_or('\u{FFFD}')
        } else if code == 0x0D
            || (is_control_codepoint(code) && !is_ascii_whitespace_codepoint(code))
        {
            self.emit_error(ParseErrorCode::ControlCharacterReference);
            // Apply the replacement table from the spec
            match code {
                0x80 => '\u{20AC}',
                0x82 => '\u{201A}',
                0x83 => '\u{0192}',
                0x84 => '\u{201E}',
                0x85 => '\u{2026}',
                0x86 => '\u{2020}',
                0x87 => '\u{2021}',
                0x88 => '\u{02C6}',
                0x89 => '\u{2030}',
                0x8A => '\u{0160}',
                0x8B => '\u{2039}',
                0x8C => '\u{0152}',
                0x8E => '\u{017D}',
                0x91 => '\u{2018}',
                0x92 => '\u{2019}',
                0x93 => '\u{201C}',
                0x94 => '\u{201D}',
                0x95 => '\u{2022}',
                0x96 => '\u{2013}',
                0x97 => '\u{2014}',
                0x98 => '\u{02DC}',
                0x99 => '\u{2122}',
                0x9A => '\u{0161}',
                0x9B => '\u{203A}',
                0x9C => '\u{0153}',
                0x9E => '\u{017E}',
                0x9F => '\u{0178}',
                _ => char::from_u32(code).unwrap_or('\u{FFFD}'),
            }
        } else {
            char::from_u32(code).unwrap_or('\u{FFFD}')
        };

        self.temp_buffer.clear();
        self.temp_buffer.push(c);
        self.flush_code_points_consumed_as_character_reference();
        self.state = self.return_state;
    }

    /// Flush code points consumed as a character reference.
    /// If the return state is an attribute value state, append to the current attribute value.
    /// Otherwise, emit as character tokens.
    fn flush_code_points_consumed_as_character_reference(&mut self) {
        let in_attribute = matches!(
            self.return_state,
            State::AttributeValueDoubleQuoted
                | State::AttributeValueSingleQuoted
                | State::AttributeValueUnquoted
        );

        if in_attribute {
            self.current_attr_value.push_str(&self.temp_buffer);
            self.temp_buffer.clear();
        } else {
            for c in std::mem::take(&mut self.temp_buffer).chars() {
                self.emit_char(c);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn emit_token(&mut self, token: Token) {
        self.pending_tokens.push_back(token);
    }

    fn emit_char(&mut self, ch: char) {
        self.pending_tokens.push_back(Token::character(ch));
    }

    fn emit_error(&mut self, code: ParseErrorCode) {
        let (line, col) = self.input.position();
        self.errors.push(ParseError { code, line, col });
    }

    fn create_start_tag(&mut self) {
        self.current_tag_name.clear();
        self.current_tag_is_end_tag = false;
        self.current_tag_self_closing = false;
        self.current_tag_attributes.clear();
        self.current_tag_attribute_names = None;
        self.current_attr_name.clear();
        self.current_attr_value.clear();
    }

    fn create_end_tag(&mut self) {
        self.current_tag_name.clear();
        self.current_tag_is_end_tag = true;
        self.current_tag_self_closing = false;
        self.current_tag_attributes.clear();
        self.current_tag_attribute_names = None;
        self.current_attr_name.clear();
        self.current_attr_value.clear();
    }

    fn emit_current_tag(&mut self) {
        self.finish_current_attribute();
        if self.current_tag_is_end_tag {
            if !self.current_tag_attributes.is_empty() {
                self.emit_error(ParseErrorCode::EndTagWithAttributes);
            }
            if self.current_tag_self_closing {
                self.emit_error(ParseErrorCode::EndTagWithTrailingSolidus);
            }
            let name = std::mem::take(&mut self.current_tag_name);
            self.emit_token(Token::EndTag { name });
        } else {
            let name = std::mem::take(&mut self.current_tag_name);
            let attributes = std::mem::take(&mut self.current_tag_attributes);
            self.last_start_tag_name = Some(name.clone());
            self.emit_token(Token::StartTag {
                name,
                attributes,
                self_closing: self.current_tag_self_closing,
            });
        }
    }

    fn start_new_attribute(&mut self) {
        self.finish_current_attribute();
        self.current_attr_name.clear();
        self.current_attr_value.clear();
    }

    fn finish_current_attribute(&mut self) {
        const ATTRIBUTE_NAME_SET_THRESHOLD: usize = 8;

        if self.current_attr_name.is_empty() {
            return;
        }

        let is_duplicate = if let Some(names) = &mut self.current_tag_attribute_names {
            !names.insert(self.current_attr_name.clone())
        } else {
            let is_duplicate = self
                .current_tag_attributes
                .iter()
                .any(|a| a.name == self.current_attr_name);

            if !is_duplicate
                && self.current_tag_attributes.len() + 1 >= ATTRIBUTE_NAME_SET_THRESHOLD
            {
                let mut names = HashSet::with_capacity(self.current_tag_attributes.len() + 1);
                for attr in &self.current_tag_attributes {
                    names.insert(attr.name.clone());
                }
                names.insert(self.current_attr_name.clone());
                self.current_tag_attribute_names = Some(names);
            }

            is_duplicate
        };

        if is_duplicate {
            self.emit_error(ParseErrorCode::DuplicateAttribute);
        } else {
            self.current_tag_attributes.push(Attribute {
                name: std::mem::take(&mut self.current_attr_name),
                value: std::mem::take(&mut self.current_attr_value),
            });
        }

        self.current_attr_name.clear();
        self.current_attr_value.clear();
    }

    fn create_comment(&mut self, initial: &str) {
        self.current_comment_data.clear();
        self.current_comment_data.push_str(initial);
    }

    fn emit_current_comment(&mut self) {
        let data = std::mem::take(&mut self.current_comment_data);
        self.emit_token(Token::Comment { data });
    }

    fn create_doctype(&mut self) {
        self.current_doctype_name = None;
        self.current_doctype_public_id = None;
        self.current_doctype_system_id = None;
        self.current_doctype_force_quirks = false;
    }

    fn emit_current_doctype(&mut self) {
        let name = self.current_doctype_name.take();
        let public_id = self.current_doctype_public_id.take();
        let system_id = self.current_doctype_system_id.take();
        self.emit_token(Token::Doctype {
            name,
            public_id,
            system_id,
            force_quirks: self.current_doctype_force_quirks,
        });
    }

    /// Helper for RCDATA/RAWTEXT/Script end tag name states:
    /// when the end tag is not appropriate, emit "</" + temp_buffer contents as characters,
    /// then reconsume the current character.
    fn emit_end_tag_name_chars_and_reconsume(&mut self, current: char) {
        self.emit_char('<');
        self.emit_char('/');
        for c in std::mem::take(&mut self.temp_buffer).chars() {
            self.emit_char(c);
        }
        self.emit_char(current);
    }

    /// Like emit_end_tag_name_chars_and_reconsume but without emitting the current char
    /// (caller will reconsume).
    fn emit_end_tag_name_chars_no_current(&mut self) {
        self.emit_char('<');
        self.emit_char('/');
        for c in std::mem::take(&mut self.temp_buffer).chars() {
            self.emit_char(c);
        }
    }

    fn is_appropriate_end_tag(&self) -> bool {
        if let Some(ref last) = self.last_start_tag_name {
            *last == self.current_tag_name
        } else {
            false
        }
    }
}

fn is_surrogate_codepoint(cp: u32) -> bool {
    (0xD800..=0xDFFF).contains(&cp)
}

fn is_noncharacter_codepoint(cp: u32) -> bool {
    (0xFDD0..=0xFDEF).contains(&cp) || (cp & 0xFFFF == 0xFFFE) || (cp & 0xFFFF == 0xFFFF)
}

fn is_control_codepoint(cp: u32) -> bool {
    // C0 controls (except HT, LF, FF which are whitespace) and C1 controls
    (0x00..=0x1F).contains(&cp) || (0x7F..=0x9F).contains(&cp)
}

fn is_ascii_whitespace_codepoint(cp: u32) -> bool {
    matches!(cp, 0x09 | 0x0A | 0x0C | 0x20)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<Token> {
        let stream = InputStream::new(input);
        let mut tokenizer = Tokenizer::new(stream);
        let mut tokens = Vec::new();
        loop {
            let token = tokenizer.next_token();
            let is_eof = token == Token::EndOfFile;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn tokenize_no_eof(input: &str) -> Vec<Token> {
        let mut tokens = tokenize(input);
        tokens.pop(); // remove EOF
        tokens
    }

    #[test]
    fn empty_input() {
        let tokens = tokenize("");
        assert_eq!(tokens, vec![Token::EndOfFile]);
    }

    #[test]
    fn text_only() {
        let tokens = tokenize("hello");
        assert_eq!(
            tokens,
            vec![
                Token::Character { data: 'h' },
                Token::Character { data: 'e' },
                Token::Character { data: 'l' },
                Token::Character { data: 'l' },
                Token::Character { data: 'o' },
                Token::EndOfFile,
            ]
        );
    }

    #[test]
    fn null_in_data() {
        let stream = InputStream::new("\0");
        let mut tokenizer = Tokenizer::new(stream);
        let token = tokenizer.next_token();
        assert_eq!(token, Token::Character { data: '\0' });
        assert_eq!(tokenizer.errors.len(), 1);
        assert_eq!(
            tokenizer.errors[0].code,
            ParseErrorCode::UnexpectedNullCharacter
        );
    }

    #[test]
    fn simple_start_tag() {
        let tokens = tokenize_no_eof("<div>");
        assert_eq!(
            tokens,
            vec![Token::StartTag {
                name: "div".to_string(),
                attributes: vec![],
                self_closing: false,
            }]
        );
    }

    #[test]
    fn simple_end_tag() {
        let tokens = tokenize_no_eof("</div>");
        assert_eq!(
            tokens,
            vec![Token::EndTag {
                name: "div".to_string(),
            }]
        );
    }

    #[test]
    fn self_closing_tag() {
        let tokens = tokenize_no_eof("<br/>");
        assert_eq!(
            tokens,
            vec![Token::StartTag {
                name: "br".to_string(),
                attributes: vec![],
                self_closing: true,
            }]
        );
    }

    #[test]
    fn tag_with_attributes() {
        let tokens = tokenize_no_eof(r#"<div class="foo" id='bar' hidden>"#);
        assert_eq!(
            tokens,
            vec![Token::StartTag {
                name: "div".to_string(),
                attributes: vec![
                    Attribute {
                        name: "class".to_string(),
                        value: "foo".to_string(),
                    },
                    Attribute {
                        name: "id".to_string(),
                        value: "bar".to_string(),
                    },
                    Attribute {
                        name: "hidden".to_string(),
                        value: String::new(),
                    },
                ],
                self_closing: false,
            }]
        );
    }

    #[test]
    fn unquoted_attribute() {
        let tokens = tokenize_no_eof("<div class=foo>");
        assert_eq!(
            tokens,
            vec![Token::StartTag {
                name: "div".to_string(),
                attributes: vec![Attribute {
                    name: "class".to_string(),
                    value: "foo".to_string(),
                }],
                self_closing: false,
            }]
        );
    }

    #[test]
    fn uppercase_tag_lowercased() {
        let tokens = tokenize_no_eof("<DIV>");
        assert_eq!(
            tokens,
            vec![Token::StartTag {
                name: "div".to_string(),
                attributes: vec![],
                self_closing: false,
            }]
        );
    }

    #[test]
    fn duplicate_attributes() {
        let stream = InputStream::new(r#"<div class="a" class="b">"#);
        let mut tokenizer = Tokenizer::new(stream);
        let token = tokenizer.next_token();
        assert_eq!(
            token,
            Token::StartTag {
                name: "div".to_string(),
                attributes: vec![Attribute {
                    name: "class".to_string(),
                    value: "a".to_string(),
                }],
                self_closing: false,
            }
        );
        assert!(tokenizer
            .errors
            .iter()
            .any(|e| e.code == ParseErrorCode::DuplicateAttribute));
    }

    #[test]
    fn bogus_comment_from_question_mark() {
        let tokens = tokenize_no_eof("<?xml version=\"1.0\"?>");
        // Should produce a comment token
        assert!(matches!(tokens[0], Token::Comment { .. }));
    }

    #[test]
    fn tag_and_text() {
        let tokens = tokenize_no_eof("<p>hello</p>");
        assert_eq!(
            tokens,
            vec![
                Token::StartTag {
                    name: "p".to_string(),
                    attributes: vec![],
                    self_closing: false,
                },
                Token::Character { data: 'h' },
                Token::Character { data: 'e' },
                Token::Character { data: 'l' },
                Token::Character { data: 'l' },
                Token::Character { data: 'o' },
                Token::EndTag {
                    name: "p".to_string(),
                },
            ]
        );
    }
}
