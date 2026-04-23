# lyng-html-parser

A WHATWG-style HTML parser in pure Rust.

This crate parses HTML into an arena-backed DOM tree, reports parse errors, and is validated against the html5lib tokenizer, tree-construction, serializer, and encoding suites.

## Status

- Implements the HTML tokenizer and tree-construction pipeline in Rust.
- Supports full-document parsing, fragment parsing, foreign-content handling, and parse error collection.
- Uses an arena-backed DOM with stable node IDs.
- Keeps the library dependency-light. `encoding_rs` is optional and only enabled behind the `encoding` feature.

## Public API

The main entry points are:

- `parse_str(input: &str) -> ParseResult`
- `parse_str_scripting(input: &str, scripting_enabled: bool) -> ParseResult`
- `parse_fragment(input: &str, context: FragmentContext, scripting_enabled: bool) -> ParseResult`

`ParseResult` contains:

- `arena`: the DOM arena
- `document`: the root document node ID
- `errors`: collected parse errors
- fragment metadata when fragment parsing is used

## Example

```rust
use lyng_html_parser::dom::serialize_tree;
use lyng_html_parser::parse_str;

fn main() {
    let result = parse_str("<!doctype html><html><body><p>Hello</p></body></html>");

    assert!(result.errors.is_empty());

    let tree = serialize_tree(&result.arena, result.document);
    println!("{tree}");
}
```

## Running The Project

Run the full test suite:

```bash
cargo test --all-features
```

Run clippy:

```bash
cargo clippy --all-targets --all-features -- -W clippy::pedantic
```

## Project Layout

```text
src/
├── dom/        Arena-backed DOM representation and serializer
├── input/      WHATWG input-stream preprocessing and position tracking
├── tokenizer/  Tokenizer state machine and token definitions
├── tree/       Tree builder, insertion modes, open-elements stack, formatting list
├── error.rs    Parse error types and error codes
└── lib.rs      Public API
```

## Design Docs

- [`docs/architecture.md`](docs/architecture.md): parser pipeline, module responsibilities, and data flow
- [`docs/implementation-notes.md`](docs/implementation-notes.md): reasoning behind the main implementation choices and tradeoffs
- [`docs/spec.md`](docs/spec.md): original project PRD and scope notes

## Validation Approach

The project relies on html5lib as its main compatibility target:

- tokenizer tests
- tree-construction tests
- serializer tests
- encoding tests

That gives a much stronger signal than unit tests alone, especially for error recovery and malformed markup behavior.
