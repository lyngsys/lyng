/// The document quirks mode, determined by the DOCTYPE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuirksMode {
    NoQuirks,
    Quirks,
    LimitedQuirks,
}
