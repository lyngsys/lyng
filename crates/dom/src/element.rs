/// The three namespaces relevant to HTML parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Namespace {
    Html,
    Svg,
    MathML,
}

/// An attribute on an element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    /// The attribute prefix (e.g., "xlink" for xlink:href).
    pub prefix: Option<String>,
    pub name: String,
    pub value: String,
}
