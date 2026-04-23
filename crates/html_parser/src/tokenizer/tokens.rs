/// An attribute as produced by the tokenizer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

/// A token emitted by the HTML tokenizer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Doctype {
        name: Option<String>,
        public_id: Option<String>,
        system_id: Option<String>,
        force_quirks: bool,
    },
    StartTag {
        name: String,
        attributes: Vec<Attribute>,
        self_closing: bool,
    },
    EndTag {
        name: String,
    },
    Comment {
        data: String,
    },
    Character {
        data: char,
    },
    EndOfFile,
}

impl Token {
    pub fn character(ch: char) -> Self {
        Self::Character { data: ch }
    }
}
