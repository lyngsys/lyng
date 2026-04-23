#![no_main]

use libfuzzer_sys::fuzz_target;

use lyng_js_common::{AtomTable, SourceId};
use lyng_js_lexer::{Lexer, TokenKind};

fuzz_target!(|data: &[u8]| {
    // Only fuzz valid UTF-8 — the lexer expects &str.
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };

    let mut atoms = AtomTable::new();
    let mut lexer = Lexer::new(source, SourceId::new(0), &mut atoms);

    // Consume all tokens until EOF. Must not panic.
    loop {
        let token = lexer.next_token();
        if token.kind == TokenKind::Eof {
            break;
        }
    }
});
