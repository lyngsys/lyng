//! Comprehensive tests for the lyng-js lexer.
#![allow(clippy::cast_lossless, clippy::float_cmp, clippy::manual_let_else)]

use lyng_js_common::{AtomTable, SourceId};

use crate::{Lexer, LexerMode, LiteralId, Token, TokenKind, TokenPayload};

/// Lex all tokens from source.
fn lex_all(source: &str) -> Vec<Token> {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new(source, source_id, &mut atoms);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        let is_eof = tok.kind == TokenKind::Eof;
        tokens.push(tok);
        if is_eof {
            break;
        }
    }
    tokens
}

fn lex_single(source: &str) -> Token {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new(source, source_id, &mut atoms);
    lexer.next_token()
}

fn num_value(tok: &Token) -> f64 {
    match tok.payload {
        TokenPayload::Number(bits) => f64::from_bits(bits),
        _ => panic!("expected Number payload, got {:?}", tok.payload),
    }
}

fn literal_id(tok: &Token) -> LiteralId {
    match tok.payload {
        TokenPayload::Literal(id) => id,
        _ => panic!("expected Literal payload, got {:?}", tok.payload),
    }
}

// =========================================================================
// Keywords
// =========================================================================

#[test]
fn keywords() {
    let cases = [
        ("await", TokenKind::Await),
        ("break", TokenKind::Break),
        ("case", TokenKind::Case),
        ("catch", TokenKind::Catch),
        ("class", TokenKind::Class),
        ("const", TokenKind::Const),
        ("continue", TokenKind::Continue),
        ("debugger", TokenKind::Debugger),
        ("default", TokenKind::Default),
        ("delete", TokenKind::Delete),
        ("do", TokenKind::Do),
        ("else", TokenKind::Else),
        ("enum", TokenKind::Enum),
        ("export", TokenKind::Export),
        ("extends", TokenKind::Extends),
        ("false", TokenKind::False),
        ("finally", TokenKind::Finally),
        ("for", TokenKind::For),
        ("function", TokenKind::Function),
        ("if", TokenKind::If),
        ("import", TokenKind::Import),
        ("in", TokenKind::In),
        ("instanceof", TokenKind::Instanceof),
        ("new", TokenKind::New),
        ("null", TokenKind::Null),
        ("return", TokenKind::Return),
        ("super", TokenKind::Super),
        ("switch", TokenKind::Switch),
        ("this", TokenKind::This),
        ("throw", TokenKind::Throw),
        ("true", TokenKind::True),
        ("try", TokenKind::Try),
        ("typeof", TokenKind::Typeof),
        ("var", TokenKind::Var),
        ("void", TokenKind::Void),
        ("while", TokenKind::While),
        ("with", TokenKind::With),
        ("yield", TokenKind::Yield),
    ];
    for (src, expected_kind) in cases {
        let tok = lex_single(src);
        assert_eq!(tok.kind, expected_kind, "keyword: {src}");
    }
}

#[test]
fn contextual_keywords_are_identifiers() {
    // These should lex as Identifier, not as keywords
    let contextual = ["let", "async", "of", "get", "set", "from", "static", "as"];
    for src in contextual {
        let tok = lex_single(src);
        assert_eq!(tok.kind, TokenKind::Identifier, "contextual: {src}");
    }
}

// =========================================================================
// Identifiers
// =========================================================================

#[test]
fn simple_identifiers() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("foo bar _private $dollar", source_id, &mut atoms);

    // Collect all tokens first, then drop the lexer to release the mutable borrow.
    let tok1 = lexer.next_token();
    let tok2 = lexer.next_token();
    let tok3 = lexer.next_token();
    let tok4 = lexer.next_token();
    drop(lexer);

    assert_eq!(tok1.kind, TokenKind::Identifier);
    assert_eq!(
        atoms.resolve(match tok1.payload {
            TokenPayload::Atom(a) => a,
            _ => panic!(),
        }),
        "foo"
    );

    assert_eq!(tok2.kind, TokenKind::Identifier);
    assert_eq!(
        atoms.resolve(match tok2.payload {
            TokenPayload::Atom(a) => a,
            _ => panic!(),
        }),
        "bar"
    );

    assert_eq!(tok3.kind, TokenKind::Identifier);
    assert_eq!(
        atoms.resolve(match tok3.payload {
            TokenPayload::Atom(a) => a,
            _ => panic!(),
        }),
        "_private"
    );

    assert_eq!(tok4.kind, TokenKind::Identifier);
    assert_eq!(
        atoms.resolve(match tok4.payload {
            TokenPayload::Atom(a) => a,
            _ => panic!(),
        }),
        "$dollar"
    );
}

#[test]
fn escaped_keyword_is_identifier() {
    // \u0069\u0066 spells "if" but with escapes, so it must be an identifier
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("\\u0069\\u0066", source_id, &mut atoms);
    let tok = lexer.next_token();
    drop(lexer);
    assert_eq!(tok.kind, TokenKind::Identifier);
    assert!(tok.contains_escape());
    let atom = match tok.payload {
        TokenPayload::Atom(a) => a,
        _ => panic!(),
    };
    assert_eq!(atoms.resolve(atom), "if");
}

#[test]
fn unicode_escape_in_identifier() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("h\\u0065llo", source_id, &mut atoms);
    let tok = lexer.next_token();
    drop(lexer);
    assert_eq!(tok.kind, TokenKind::Identifier);
    assert!(tok.contains_escape());
    let atom = match tok.payload {
        TokenPayload::Atom(a) => a,
        _ => panic!(),
    };
    assert_eq!(atoms.resolve(atom), "hello");
}

#[test]
fn unicode_braced_escape_in_identifier() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("\\u{61}bc", source_id, &mut atoms);
    let tok = lexer.next_token();
    drop(lexer);
    assert_eq!(tok.kind, TokenKind::Identifier);
    assert!(tok.contains_escape());
    let atom = match tok.payload {
        TokenPayload::Atom(a) => a,
        _ => panic!(),
    };
    assert_eq!(atoms.resolve(atom), "abc");
}

#[test]
fn raw_multibyte_identifier_start_and_continue_are_lexed_as_one_identifier() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("πβ", source_id, &mut atoms);
    let tok = lexer.next_token();
    let had_errors = lexer.diagnostics.has_errors();
    drop(lexer);

    assert_eq!(tok.kind, TokenKind::Identifier);
    let atom = match tok.payload {
        TokenPayload::Atom(a) => a,
        _ => panic!(),
    };
    assert_eq!(atoms.resolve(atom), "πβ");
    assert!(!had_errors);
}

// =========================================================================
// Numeric literals
// =========================================================================

#[test]
fn decimal_integers() {
    let tok = lex_single("42");
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert_eq!(num_value(&tok), 42.0);
}

#[test]
fn decimal_float() {
    let tok = lex_single("2.5");
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert_eq!(num_value(&tok), 2.5);
}

#[test]
fn decimal_exponent() {
    let tok = lex_single("1e10");
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert_eq!(num_value(&tok), 1e10);
}

#[test]
fn decimal_negative_exponent() {
    let tok = lex_single("5e-3");
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert_eq!(num_value(&tok), 5e-3);
}

#[test]
fn hex_literal() {
    let tok = lex_single("0xFF");
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert_eq!(num_value(&tok), 255.0);
}

#[test]
fn octal_literal() {
    let tok = lex_single("0o77");
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert_eq!(num_value(&tok), 63.0);
}

#[test]
fn binary_literal() {
    let tok = lex_single("0b1010");
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert_eq!(num_value(&tok), 10.0);
}

#[test]
fn numeric_separators() {
    let tok = lex_single("1_000_000");
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert_eq!(num_value(&tok), 1_000_000.0);
}

#[test]
fn hex_with_separators() {
    let tok = lex_single("0xFF_FF");
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert_eq!(num_value(&tok), 0xFFFF as f64);
}

#[test]
fn bigint_decimal() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("123n", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::BigIntLiteral);
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_bigint(id).raw, "123");
}

#[test]
fn bigint_hex() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("0xFFn", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::BigIntLiteral);
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_bigint(id).raw, "0xFF");
}

#[test]
fn zero() {
    let tok = lex_single("0");
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert_eq!(num_value(&tok), 0.0);
}

#[test]
fn dot_number() {
    let tok = lex_single(".5");
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert_eq!(num_value(&tok), 0.5);
}

#[test]
fn legacy_octal_like_decimal_sets_flag() {
    let tok = lex_single("010");
    assert!(tok.has_legacy_octal_like_decimal());
}

#[test]
fn legacy_octal_integer_value_is_octal() {
    let tok = lex_single("010");
    assert!(tok.has_legacy_octal_like_decimal());
    assert_eq!(num_value(&tok), 8.0);

    let tok = lex_single("077");
    assert!(tok.has_legacy_octal_like_decimal());
    assert_eq!(num_value(&tok), 63.0);
}

#[test]
fn non_octal_decimal_integer_value_stays_decimal() {
    let tok = lex_single("08");
    assert!(tok.has_legacy_octal_like_decimal());
    assert_eq!(num_value(&tok), 8.0);

    let tok = lex_single("0708");
    assert!(tok.has_legacy_octal_like_decimal());
    assert_eq!(num_value(&tok), 708.0);
}

#[test]
fn numeric_followed_by_identifier_reports_error() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("3in", source_id, &mut atoms);
    let _ = lexer.next_token();
    assert!(lexer.diagnostics.has_errors());
}

#[test]
fn invalid_bigint_leading_zero_reports_error() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("01n", source_id, &mut atoms);
    let _ = lexer.next_token();
    assert!(lexer.diagnostics.has_errors());
}

#[test]
fn invalid_separator_after_radix_prefix_reports_error() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("0x_1", source_id, &mut atoms);
    let _ = lexer.next_token();
    assert!(lexer.diagnostics.has_errors());
}

#[test]
fn numeric_literal_before_nbsp_is_allowed() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("2\u{00A0};", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert!(!lexer.diagnostics.has_errors());
}

#[test]
fn numeric_literal_before_line_separator_is_allowed() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("1\u{2028};", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert!(!lexer.diagnostics.has_errors());
}

#[test]
fn numeric_literal_before_unicode_identifier_start_reports_error() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("1π", source_id, &mut atoms);
    let _ = lexer.next_token();
    assert!(lexer.diagnostics.has_errors());
}

// =========================================================================
// String literals
// =========================================================================

#[test]
fn double_quoted_string() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("\"hello\"", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::StringLiteral);
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_string(id).as_str(), Some("hello"));
}

#[test]
fn single_quoted_string() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("'world'", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::StringLiteral);
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_string(id).as_str(), Some("world"));
}

#[test]
fn string_escape_sequences() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new(r#""a\nb\tc\\\"""#, source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::StringLiteral);
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_string(id).as_str(), Some("a\nb\tc\\\""));
}

#[test]
fn string_hex_escape() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new(r#""\x41""#, source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::StringLiteral);
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_string(id).as_str(), Some("A"));
}

#[test]
fn string_unicode_escape() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new(r#""\u0041""#, source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::StringLiteral);
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_string(id).as_str(), Some("A"));
}

#[test]
fn string_unicode_braced_escape() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new(r#""\u{1F600}""#, source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::StringLiteral);
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_string(id).as_str(), Some("\u{1F600}"));
}

#[test]
fn string_identity_escape_consumes_full_unicode_character() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("\"\\А\"", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::StringLiteral);
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_string(id).as_str(), Some("А"));
}

#[test]
fn string_line_continuation_accepts_unicode_line_terminators() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("\"\\\u{2028}\\\u{2029}\"", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::StringLiteral);
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_string(id).as_str(), Some(""));
}

#[test]
fn empty_string() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("\"\"", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::StringLiteral);
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_string(id).as_str(), Some(""));
}

#[test]
fn string_literal_sets_legacy_escape_flags() {
    let tok = lex_single(r#""\8\1""#);
    assert!(tok.has_non_octal_decimal_escape());
    assert!(tok.has_legacy_octal_escape());
}

#[test]
fn string_literal_sets_non_octal_flag_for_zero_followed_by_eight() {
    let tok = lex_single(r#""\08""#);
    assert!(tok.has_non_octal_decimal_escape());
}

#[test]
fn string_literal_allows_line_separator() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("\"\u{2028}\"", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::StringLiteral);
    assert!(!lexer.diagnostics.has_errors());
}

#[test]
fn skips_ogham_space_mark() {
    let tok = lex_single("\u{1680};");
    assert_eq!(tok.kind, TokenKind::Semicolon);
}

#[test]
fn skips_ideographic_space() {
    let tok = lex_single("\u{3000};");
    assert_eq!(tok.kind, TokenKind::Semicolon);
}

// =========================================================================
// Template literals
// =========================================================================

#[test]
fn no_substitution_template() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("`hello`", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::NoSubstitutionTemplate);
    let id = literal_id(&tok);
    let chunk = lexer.literals.get_template(id);
    assert_eq!(chunk.cooked.as_deref(), Some("hello"));
    assert_eq!(chunk.raw, "hello");
}

#[test]
fn template_with_substitution() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("`hello ${name}`", source_id, &mut atoms);

    let head = lexer.next_token();
    assert_eq!(head.kind, TokenKind::TemplateHead);
    let id = literal_id(&head);
    assert_eq!(
        lexer.literals.get_template(id).cooked.as_deref(),
        Some("hello ")
    );

    let ident = lexer.next_token();
    assert_eq!(ident.kind, TokenKind::Identifier);

    let rbrace = lexer.next_token();
    assert_eq!(rbrace.kind, TokenKind::RBrace);

    lexer.set_mode(LexerMode::TemplateContinuation);
    let tail = lexer.next_token();
    assert_eq!(tail.kind, TokenKind::TemplateTail);
}

#[test]
fn template_with_escape() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("`a\\nb`", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::NoSubstitutionTemplate);
    let id = literal_id(&tok);
    let chunk = lexer.literals.get_template(id);
    assert_eq!(chunk.cooked.as_deref(), Some("a\nb"));
    assert_eq!(chunk.raw, "a\\nb");
}

#[test]
fn template_with_invalid_hex_escape_keeps_closing_backtick() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("`\\xg`", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::NoSubstitutionTemplate);
    let id = literal_id(&tok);
    let chunk = lexer.literals.get_template(id);
    assert_eq!(chunk.cooked, None);
    assert_eq!(chunk.raw, "\\xg");
}

#[test]
fn template_with_invalid_braced_unicode_escape_keeps_closing_backtick() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("`\\u{0`", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::NoSubstitutionTemplate);
    let id = literal_id(&tok);
    let chunk = lexer.literals.get_template(id);
    assert_eq!(chunk.cooked, None);
    assert_eq!(chunk.raw, "\\u{0");
}

// =========================================================================
// RegExp literals
// =========================================================================

#[test]
fn simple_regexp() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("/abc/gi", source_id, &mut atoms);
    lexer.set_mode(LexerMode::RegExp);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::RegExpLiteral);
    let id = literal_id(&tok);
    let re = lexer.literals.get_regexp(id);
    assert_eq!(re.pattern, "abc");
    assert_eq!(re.flags, "gi");
}

#[test]
fn regexp_with_class() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("/[a-z]/", source_id, &mut atoms);
    lexer.set_mode(LexerMode::RegExp);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::RegExpLiteral);
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_regexp(id).pattern, "[a-z]");
}

#[test]
fn regexp_with_escaped_slash() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("/a\\/b/", source_id, &mut atoms);
    lexer.set_mode(LexerMode::RegExp);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::RegExpLiteral);
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_regexp(id).pattern, "a\\/b");
}

#[test]
fn slash_is_division_in_normal_mode() {
    let tok = lex_single("/");
    assert_eq!(tok.kind, TokenKind::Slash);
}

// =========================================================================
// Punctuators
// =========================================================================

#[test]
fn all_single_char_punctuators() {
    let cases = [
        ("(", TokenKind::LParen),
        (")", TokenKind::RParen),
        ("{", TokenKind::LBrace),
        ("}", TokenKind::RBrace),
        ("[", TokenKind::LBracket),
        ("]", TokenKind::RBracket),
        (";", TokenKind::Semicolon),
        (",", TokenKind::Comma),
        (":", TokenKind::Colon),
        ("~", TokenKind::Tilde),
    ];
    for (src, expected) in cases {
        let tok = lex_single(src);
        assert_eq!(tok.kind, expected, "punctuator: {src}");
    }
}

#[test]
fn multi_char_punctuators() {
    let cases = [
        ("...", TokenKind::Ellipsis),
        ("=>", TokenKind::Arrow),
        ("?.", TokenKind::OptionalChain),
        ("??", TokenKind::QuestionQuestion),
        ("++", TokenKind::PlusPlus),
        ("--", TokenKind::MinusMinus),
        ("**", TokenKind::Exp),
        ("<<", TokenKind::LtLt),
        (">>", TokenKind::GtGt),
        (">>>", TokenKind::GtGtGt),
        ("<=", TokenKind::LtEq),
        (">=", TokenKind::GtEq),
        ("==", TokenKind::EqEq),
        ("!=", TokenKind::NotEq),
        ("===", TokenKind::EqEqEq),
        ("!==", TokenKind::NotEqEq),
        ("&&", TokenKind::AmpAmp),
        ("||", TokenKind::PipePipe),
        ("+=", TokenKind::PlusEq),
        ("-=", TokenKind::MinusEq),
        ("*=", TokenKind::StarEq),
        ("/=", TokenKind::SlashEq),
        ("%=", TokenKind::PercentEq),
        ("**=", TokenKind::ExpEq),
        ("&=", TokenKind::AmpEq),
        ("|=", TokenKind::PipeEq),
        ("^=", TokenKind::CaretEq),
        ("<<=", TokenKind::LtLtEq),
        (">>=", TokenKind::GtGtEq),
        (">>>=", TokenKind::GtGtGtEq),
        ("&&=", TokenKind::AmpAmpEq),
        ("||=", TokenKind::PipePipeEq),
        ("??=", TokenKind::QuestionQuestionEq),
    ];
    for (src, expected) in cases {
        let tok = lex_single(src);
        assert_eq!(tok.kind, expected, "punctuator: {src}");
    }
}

#[test]
fn dot_vs_ellipsis() {
    let tokens = lex_all("a.b...c");
    assert_eq!(tokens[0].kind, TokenKind::Identifier); // a
    assert_eq!(tokens[1].kind, TokenKind::Dot); // .
    assert_eq!(tokens[2].kind, TokenKind::Identifier); // b
    assert_eq!(tokens[3].kind, TokenKind::Ellipsis); // ...
    assert_eq!(tokens[4].kind, TokenKind::Identifier); // c
}

// =========================================================================
// Comments
// =========================================================================

#[test]
fn single_line_comment() {
    let tokens = lex_all("a // comment\nb");
    assert_eq!(tokens[0].kind, TokenKind::Identifier); // a
    assert_eq!(tokens[1].kind, TokenKind::Identifier); // b
    assert!(tokens[1].preceded_by_line_terminator());
}

#[test]
fn multi_line_comment() {
    let tokens = lex_all("a /* comment */ b");
    assert_eq!(tokens[0].kind, TokenKind::Identifier); // a
    assert_eq!(tokens[1].kind, TokenKind::Identifier); // b
    assert!(!tokens[1].preceded_by_line_terminator());
}

#[test]
fn multi_line_comment_with_newline() {
    let tokens = lex_all("a /* comment\n */ b");
    assert_eq!(tokens[0].kind, TokenKind::Identifier);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert!(tokens[1].preceded_by_line_terminator());
}

#[test]
fn html_open_comment_is_lexed_in_script_goal() {
    let tokens = lex_all("a<!-- comment\nb");
    assert_eq!(tokens[0].kind, TokenKind::Identifier);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert!(tokens[1].preceded_by_line_terminator());
}

#[test]
fn html_close_comment_is_lexed_at_line_start() {
    let tokens = lex_all("a\n--> comment\nb");
    assert_eq!(tokens[0].kind, TokenKind::Identifier);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert!(tokens[1].preceded_by_line_terminator());
}

#[test]
fn html_close_comment_requires_line_start() {
    let tokens = lex_all("a-->b");
    assert_eq!(tokens[0].kind, TokenKind::Identifier);
    assert_eq!(tokens[1].kind, TokenKind::MinusMinus);
    assert_eq!(tokens[2].kind, TokenKind::Gt);
    assert_eq!(tokens[3].kind, TokenKind::Identifier);
}

#[test]
fn html_comments_can_be_disabled() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("<!--x", source_id, &mut atoms);
    lexer.set_allow_html_comments(false);

    assert_eq!(lexer.next_token().kind, TokenKind::Lt);
}

// =========================================================================
// Hashbang
// =========================================================================

#[test]
fn hashbang_comment() {
    let tokens = lex_all("#!/usr/bin/env node\nvar x");
    assert_eq!(tokens[0].kind, TokenKind::Var);
    assert!(tokens[0].preceded_by_line_terminator());
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
}

// =========================================================================
// Line terminators and ASI flags
// =========================================================================

#[test]
fn line_terminator_flag() {
    let tokens = lex_all("a\nb");
    assert!(!tokens[0].preceded_by_line_terminator());
    assert!(tokens[1].preceded_by_line_terminator());
}

#[test]
fn carriage_return() {
    let tokens = lex_all("a\rb");
    assert!(tokens[1].preceded_by_line_terminator());
}

#[test]
fn crlf() {
    let tokens = lex_all("a\r\nb");
    assert!(tokens[1].preceded_by_line_terminator());
}

// =========================================================================
// Private identifiers
// =========================================================================

#[test]
fn private_identifier() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("#foo", source_id, &mut atoms);
    let tok = lexer.next_token();
    drop(lexer);
    assert_eq!(tok.kind, TokenKind::PrivateIdentifier);
    let atom = match tok.payload {
        TokenPayload::Atom(a) => a,
        _ => panic!(),
    };
    assert_eq!(atoms.resolve(atom), "foo");
}

// =========================================================================
// EOF
// =========================================================================

#[test]
fn empty_source_gives_eof() {
    let tok = lex_single("");
    assert_eq!(tok.kind, TokenKind::Eof);
}

#[test]
fn whitespace_only_gives_eof() {
    let tok = lex_single("   \t  ");
    assert_eq!(tok.kind, TokenKind::Eof);
}

// =========================================================================
// Integration: mixed token sequences
// =========================================================================

#[test]
fn simple_var_declaration() {
    let tokens = lex_all("var x = 42;");
    assert_eq!(tokens[0].kind, TokenKind::Var);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert_eq!(tokens[2].kind, TokenKind::Eq);
    assert_eq!(tokens[3].kind, TokenKind::NumericLiteral);
    assert_eq!(num_value(&tokens[3]), 42.0);
    assert_eq!(tokens[4].kind, TokenKind::Semicolon);
    assert_eq!(tokens[5].kind, TokenKind::Eof);
}

#[test]
fn arrow_function() {
    let tokens = lex_all("(x) => x + 1");
    assert_eq!(tokens[0].kind, TokenKind::LParen);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert_eq!(tokens[2].kind, TokenKind::RParen);
    assert_eq!(tokens[3].kind, TokenKind::Arrow);
    assert_eq!(tokens[4].kind, TokenKind::Identifier);
    assert_eq!(tokens[5].kind, TokenKind::Plus);
    assert_eq!(tokens[6].kind, TokenKind::NumericLiteral);
}

#[test]
fn class_declaration() {
    let tokens = lex_all("class Foo extends Bar {}");
    assert_eq!(tokens[0].kind, TokenKind::Class);
    assert_eq!(tokens[1].kind, TokenKind::Identifier); // Foo
    assert_eq!(tokens[2].kind, TokenKind::Extends);
    assert_eq!(tokens[3].kind, TokenKind::Identifier); // Bar
    assert_eq!(tokens[4].kind, TokenKind::LBrace);
    assert_eq!(tokens[5].kind, TokenKind::RBrace);
}

#[test]
fn no_diagnostics_for_valid_source() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("var x = 1 + 2;", source_id, &mut atoms);
    loop {
        let tok = lexer.next_token();
        if tok.kind == TokenKind::Eof {
            break;
        }
    }
    assert!(lexer.diagnostics.is_empty());
}

#[test]
fn unterminated_string_reports_error() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("\"hello", source_id, &mut atoms);
    let _ = lexer.next_token();
    assert!(lexer.diagnostics.has_errors());
}

// =========================================================================
// Unicode escapes
// =========================================================================

#[test]
fn string_with_null_escape() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("\"a\\0b\"", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::StringLiteral);
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_string(id).as_str(), Some("a\0b"));
}

#[test]
fn string_with_surrogate_pair_escape() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("\"\\ud801\\udc28\"", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::StringLiteral);
    assert!(lexer.diagnostics.is_empty());
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_string(id).as_str(), Some("𐐨"));
}

#[test]
fn string_with_lone_surrogate_escape_does_not_error() {
    let mut atoms = AtomTable::new();
    let source_id = SourceId::new(0);
    let mut lexer = Lexer::new("\"\\ud801\"", source_id, &mut atoms);
    let tok = lexer.next_token();
    assert_eq!(tok.kind, TokenKind::StringLiteral);
    assert!(lexer.diagnostics.is_empty());
    let id = literal_id(&tok);
    assert_eq!(lexer.literals.get_string(id).as_str(), None);
    assert_eq!(lexer.literals.get_string(id).code_units(), vec![0xD801]);
}

// =========================================================================
// Spans
// =========================================================================

#[test]
fn span_offsets_correct() {
    let tokens = lex_all("if (x)");
    // "if" is at offset 0..2
    assert_eq!(tokens[0].span.range.start.raw(), 0);
    assert_eq!(tokens[0].span.range.end.raw(), 2);
    // "(" at 3..4
    assert_eq!(tokens[1].span.range.start.raw(), 3);
    assert_eq!(tokens[1].span.range.end.raw(), 4);
    // "x" at 4..5
    assert_eq!(tokens[2].span.range.start.raw(), 4);
    assert_eq!(tokens[2].span.range.end.raw(), 5);
    // ")" at 5..6
    assert_eq!(tokens[3].span.range.start.raw(), 5);
    assert_eq!(tokens[3].span.range.end.raw(), 6);
}

// =========================================================================
// Numeric edge cases
// =========================================================================

#[test]
fn float_with_leading_dot() {
    let tok = lex_single(".123");
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert!((num_value(&tok) - 0.123).abs() < f64::EPSILON);
}

#[test]
fn float_with_exponent_and_sign() {
    let tok = lex_single("1.5e+3");
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert_eq!(num_value(&tok), 1500.0);
}

#[test]
fn binary_with_separators() {
    let tok = lex_single("0b1010_0101");
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert_eq!(num_value(&tok), 0b1010_0101 as f64);
}

#[test]
fn octal_with_separators() {
    let tok = lex_single("0o77_77");
    assert_eq!(tok.kind, TokenKind::NumericLiteral);
    assert_eq!(num_value(&tok), 0o7777 as f64);
}

// =========================================================================
// Token is Copy and compact
// =========================================================================

#[test]
fn token_is_copy() {
    let tok = lex_single("42");
    let tok2 = tok; // Copy
    assert_eq!(tok.kind, tok2.kind);
}

#[test]
fn token_size_is_reasonable() {
    // Token should be compact. With Span(12) + kind(1) + flags(1) + payload(8) + padding,
    // it should be at most 32 bytes.
    assert!(
        std::mem::size_of::<Token>() <= 32,
        "Token is {} bytes",
        std::mem::size_of::<Token>()
    );
}
