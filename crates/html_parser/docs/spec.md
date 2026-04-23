# PRD: HTML Parser — Step 2

## Overview

Build a spec-compliant HTML parser in Rust that transforms raw byte streams into an in-memory DOM tree. The parser implements the WHATWG HTML Living Standard parsing algorithm (§13 "Parsing HTML documents") and targets validation against the html5lib-tests suite.

This is the second component of a from-scratch browser engine, following a working JS engine (Step 1). The parser has no dependency on the JS engine — it produces a standalone DOM tree that will be wired to JS in Step 3.

---

## Goals

1. **Spec compliance** — Implement the WHATWG HTML parsing algorithm faithfully, including error recovery. The spec defines exact behavior for all inputs, valid or not.
2. **Testability** — Pass the html5lib-tests tokenizer and tree construction suites as the primary correctness metric.
3. **Zero or near-zero dependencies** — The parser must be written in pure Rust with no required third-party crates. Exceptions are listed below with justification.
4. **Clean DOM interface** — Produce a well-typed DOM tree that can later be exposed to JS (Step 3) and consumed by CSS style resolution (Step 4).
5. **Performance is secondary** — Correctness first. The architecture should not preclude optimization later, but this phase prioritizes getting the right answer over getting it fast.

---

## Non-Goals (for this phase)

- JavaScript execution or script loading (Step 3)
- CSS parsing or style computation (Step 4)
- Networking / resource fetching (Step 7)
- `<img>`, `<video>`, or any subresource loading
- Shadow DOM (complex, can be layered later)
- Custom elements (requires JS engine integration)
- Full mutation observer support
- `document.write()` (requires JS engine; stub as no-op)

---

## Dependency Policy

**Target: zero crates.**

The HTML parser is a self-contained state machine over byte/character streams. There is no algorithmic reason to pull in external code. The WHATWG spec is the implementation guide.

**Permitted exception — encoding detection and conversion:**

The spec defines a character encoding sniffing algorithm (§13.2.3) and requires support for decoding a set of character encodings defined by the WHATWG Encoding Standard. Implementing all legacy encodings from scratch (Shift_JIS, EUC-KR, Big5, ISO-2022-JP, dozens of single-byte codecs, etc.) is a multi-thousand-line effort orthogonal to the parser itself.

- `encoding_rs` (by Henri Sivonen, the author of the Gecko HTML parser) is permitted. It implements the WHATWG Encoding Standard exactly, is `no_std` compatible, has zero transitive dependencies in its core, and is used by Firefox, Servo, and Deno. Writing your own would be reimplementing the same spec to the same result.

If encoding support is deferred to a later phase, the parser MAY initially assume UTF-8 input only, with a compile-time or runtime flag to enable full encoding support via `encoding_rs` when ready. UTF-8 only is sufficient to pass html5lib-tests (all test inputs are UTF-8).

**Not permitted:**

- HTML parsing crates (`html5ever`, `lol_html`, `scraper`, etc.) — this is the thing we're building
- General-purpose string manipulation crates (`regex`, etc.) — the parser doesn't need regex; it's a character-by-character state machine
- Serialization crates (`serde`, etc.) — test harness can use them in `dev-dependencies` if helpful for parsing html5lib-tests JSON, but they must not appear in the library itself

---

## Architecture

### Module Structure

```
crate::html_parser
├── encoding/        # Encoding sniffing & decoding (wraps encoding_rs or UTF-8 stub)
├── tokenizer/       # Tokenizer state machine
│   ├── states.rs    # All tokenizer states (§13.2.5)
│   ├── tokens.rs    # Token types (DOCTYPE, start tag, end tag, comment, character, EOF)
│   └── mod.rs       # Tokenizer entry point, drives state transitions
├── tree/            # Tree construction
│   ├── builder.rs   # Tree construction dispatcher (§13.2.6)
│   ├── insertion.rs # Insertion modes and the insertion mode state machine
│   ├── active.rs    # List of active formatting elements & adoption agency
│   ├── open.rs      # Stack of open elements
│   └── mod.rs
├── dom/             # DOM node types
│   ├── node.rs      # Node, NodeType, and the tree structure
│   ├── document.rs  # Document node
│   ├── element.rs   # Element, attribute storage, namespace
│   ├── text.rs      # Text, Comment, ProcessingInstruction
│   ├── doctype.rs   # DocumentType node
│   └── mod.rs
├── input/           # Input stream preprocessing
│   ├── stream.rs    # Character-by-character input stream with preprocessing (§13.2.3.5)
│   └── mod.rs
├── error.rs         # Parse error types (§13.2.2)
└── lib.rs           # Public API surface
```

### Component Descriptions

#### 1. Input Stream (`input/`)

**Spec reference:** §13.2.3 (The input byte stream), §13.2.3.5 (Preprocessing the input stream)

Responsibilities:
- Accept input as a byte slice or byte iterator
- Perform encoding sniffing per §13.2.3.1 (BOM sniffing, prescan, declared encoding, fallback) — or assume UTF-8 in the initial phase
- Decode bytes to a stream of Unicode code points
- Preprocess per §13.2.3.5: normalize CR/CRLF to LF, reject or report surrogates and noncharacters
- Track source position (line/column) for error reporting
- Support `reconsume` (push back a character for re-processing in a new state)

#### 2. Tokenizer (`tokenizer/`)

**Spec reference:** §13.2.5 (Tokenization)

The tokenizer is a state machine with 80 named states. It consumes code points from the input stream and emits tokens.

**Token types:**
```rust
enum Token {
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

struct Attribute {
    name: String,
    value: String,
}
```

Key implementation details:
- Each state is a method or match arm that reads characters and either transitions to another state, emits a token, or both
- The tokenizer must support being driven one token at a time (pull-based) so the tree constructor can control the flow — this is necessary because tree construction can change the tokenizer state (e.g., when encountering `<script>`, `<textarea>`, `<title>`, `<style>`, the tokenizer switches to RCDATA, RAWTEXT, or script data states)
- Character reference resolution (named, numeric, hex) must be implemented per §13.2.5.73–§13.2.5.78 — this includes the full named character reference table (~2,200 entries from the spec)
- Duplicate attribute detection: per spec, if a start tag token already has an attribute with a given name, the duplicate is ignored (no error, just dropped)

**Named character references:** The spec defines a trie-based lookup for named references (e.g., `&amp;` → `&`, `&notin;` → `∉`). This table is large (~2,200 entries) but static. Implementation options:
- A compile-time generated perfect hash map
- A hand-rolled trie (matches the spec's algorithm most closely)
- A sorted array with binary search

The trie approach is recommended because the spec's algorithm (§13.2.5.73) describes a character-by-character trie walk, and matching that structure simplifies verification against the spec.

#### 3. Tree Construction (`tree/`)

**Spec reference:** §13.2.6 (Tree construction)

The tree construction stage consumes tokens from the tokenizer and builds the DOM tree. It maintains:

- **Stack of open elements** — a stack of element node references, used to track nesting. Many operations involve searching this stack (e.g., "has an element in scope").
- **List of active formatting elements** — used by the adoption agency algorithm (§13.2.6.4.7) to handle misnested formatting tags like `<b><i></b></i>`. This is one of the most complex parts of the parser.
- **Insertion mode** — a state enum with ~23 modes (Initial, BeforeHtml, BeforeHead, InHead, InHeadNoscript, AfterHead, InBody, Text, InTable, InTableText, InCaption, InColumnGroup, InTableBody, InRow, InCell, InSelect, InSelectInTable, InTemplate, AfterBody, InFrameset, AfterFrameset, AfterAfterBody, AfterAfterFrameSet).
- **Template insertion modes stack** — for `<template>` element handling
- **`foster_parenting` flag** — for handling misnested content in tables
- **Original insertion mode** — saved and restored during text/script parsing
- **The Document node** — root of the tree being built

Key algorithms to implement:
- **The "in body" insertion mode** — by far the largest mode, handles the majority of HTML elements. Expect this to be hundreds of lines.
- **Adoption agency algorithm** (§13.2.6.4.7) — handles misnested formatting elements. Notoriously fiddly. Follow the spec step-by-step literally.
- **"In table" and related modes** — table parsing has special foster parenting logic where text and elements that aren't valid in table context get reparented outside the table.
- **"Reset the insertion mode appropriately"** — used when re-entering parsing after various operations.
- **"Has an element in scope"** variants — "in scope", "in list item scope", "in button scope", "in table scope", "in select scope" — each with different boundary elements.

#### 4. DOM Tree (`dom/`)

The DOM representation needs to support the operations the tree constructor performs. At this stage it does not need to be a full W3C DOM — it needs to support what the parser does.

**Required operations:**
- Create document, element, text, comment, doctype nodes
- Append child, insert before, remove child
- Set/get attributes on elements
- Reparent nodes (adoption agency, foster parenting)
- Walk ancestors (for "in scope" checks)
- Get/set text content
- Namespace-aware element creation (HTML, SVG, MathML namespaces)

**Node representation:**

A tree with parent/child/sibling pointers. In Rust, the ownership model makes traditional pointer-based trees awkward. Recommended approach:

**Arena allocation with indices.** All nodes live in a `Vec<Node>` (the arena), and "pointers" are `NodeId` (a `usize` or newtype wrapper around `u32`). Parent, first_child, last_child, next_sibling, prev_sibling are all `Option<NodeId>`. This avoids `Rc<RefCell<>>` overhead and is cache-friendly.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct NodeId(u32);

struct Arena {
    nodes: Vec<Node>,
}

struct Node {
    id: NodeId,
    parent: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
    next_sibling: Option<NodeId>,
    prev_sibling: Option<NodeId>,
    data: NodeData,
}

enum NodeData {
    Document { ... },
    Element {
        tag_name: LocalName,
        namespace: Namespace,
        attributes: Vec<Attribute>,
    },
    Text { content: String },
    Comment { content: String },
    Doctype {
        name: String,
        public_id: String,
        system_id: String,
    },
    ProcessingInstruction { ... },
}
```

**Namespaces:** The parser must handle three namespaces — HTML (`http://www.w3.org/1999/xhtml`), SVG (`http://www.w3.org/2000/svg`), and MathML (`http://www.w3.org/1998/Math/MathML`). These arise in the foreign content parsing section (§13.2.6.5). Use an enum rather than strings.

**Tag names:** The spec compares tag names constantly. Interning tag names (mapping them to integer IDs from a fixed table of known HTML/SVG/MathML element names) makes these comparisons `==` on integers rather than string comparisons. Not required, but strongly recommended — it simplifies code and makes future optimization straightforward.

```rust
enum Namespace {
    Html,
    Svg,
    MathML,
}
```

---

## Error Handling

**Spec reference:** §13.2.2 (Parse errors)

The HTML spec defines ~80 named parse errors (e.g., `unexpected-null-character`, `eof-in-doctype`, `missing-end-tag-name`). The parser MUST:
- Never abort on parse errors — HTML parsers are required to recover from all errors and produce a DOM
- Collect parse errors with their position (line/column) and error code
- Allow callers to retrieve the list of parse errors after parsing

```rust
struct ParseError {
    code: ParseErrorCode,
    line: u32,
    col: u32,
}

enum ParseErrorCode {
    UnexpectedNullCharacter,
    EofInDoctype,
    MissingEndTagName,
    // ~80 variants total, all named in the spec
}
```

---

## Quirks Mode

**Spec reference:** §13.2.6.4.1 (The "initial" insertion mode)

The parser determines the document's quirks mode from the DOCTYPE token. There are three modes: no-quirks, quirks, and limited-quirks. The spec defines exact rules for which public/system identifiers trigger which mode. Store this on the Document node:

```rust
enum QuirksMode {
    NoQuirks,
    Quirks,
    LimitedQuirks,
}
```

This does not affect parsing behavior directly but must be set correctly because it affects layout (Step 5) later.

---

## Fragment Parsing

**Spec reference:** §13.2.6.5 (Parsing HTML fragments)

`innerHTML` and other APIs use the parser in "fragment" mode, where a context element is provided and parsing starts in a specific insertion mode. The parser must support this mode — it's necessary for DOM API conformance in Step 3.

```rust
// Full document parse
fn parse_document(input: &[u8]) -> Document;

// Fragment parse (used by innerHTML, insertAdjacentHTML, etc.)
fn parse_fragment(input: &[u8], context: &Element) -> Vec<NodeId>;
```

---

## Public API

```rust
/// Parse a complete HTML document from bytes.
pub fn parse(input: &[u8]) -> ParseResult {
    // encoding sniff → decode → tokenize → tree construct → Document
}

/// Parse a complete HTML document from a string (assumed UTF-8).
pub fn parse_str(input: &str) -> ParseResult {
    // skip encoding, go straight to preprocessing → tokenize → tree construct
}

/// Parse an HTML fragment with the given context element.
pub fn parse_fragment(input: &str, context: &Element) -> FragmentResult;

pub struct ParseResult {
    pub document: Arena,       // the arena holding all nodes
    pub root: NodeId,          // the Document node
    pub errors: Vec<ParseError>,
}

/// Serialize a DOM tree back to HTML (for testing / debugging).
pub fn serialize(arena: &Arena, node: NodeId) -> String;
```

The `serialize` function is listed here because it's essential for testing — html5lib-tests tree construction tests compare against a serialized tree dump.

---

## Testing Strategy

### Primary: html5lib-tests

**Repository:** `github.com/html5lib/html5lib-tests`

The test suite includes:
- `tokenizer/`: JSON files with input → expected token sequences (~4,000 test cases)
- `tree-construction/`: `.dat` files with input → expected tree dump (~3,200 test cases)

**Tokenizer test runner:**

Parse the JSON test files (this is the one place `serde_json` in `dev-dependencies` is justified). For each test: feed input to the tokenizer, collect emitted tokens, compare against expected output. The JSON format is well-documented in the html5lib-tests repo.

**Tree construction test runner:**

Parse the `.dat` files (custom format, trivial to parse). For each test: run the full parser on the input, serialize the resulting tree using the html5lib indented format, compare against expected output.

**Target: 100% pass rate on html5lib-tests** — Every major browser engine achieves this. Partial pass rates indicate bugs, not optional features. Any test you can't pass indicates a misunderstanding of the spec that needs to be resolved.

### Secondary: Manual spec walk-through tests

Write targeted unit tests for the trickiest algorithms:
- Adoption agency (construct specific misnesting scenarios and verify the resulting tree)
- Foster parenting (text inside tables)
- Implicit tag closing (e.g., `<p>` auto-closed by `<div>`)
- Foreign content integration points (SVG in HTML, MathML in HTML)
- Encoding sniffing (BOM detection, meta charset prescan)

### Tertiary: Fuzzing

Once the parser passes html5lib-tests, fuzz it. An HTML parser must handle all inputs without panicking, crashing, or entering infinite loops. Use `cargo-fuzz` with `libFuzzer`. The oracle is simple: the parser should return a valid `ParseResult` for every input, never panic. Also fuzz with a round-trip oracle: parse → serialize → parse → serialize should produce identical output on the second pass.

---

## Implementation Sequence

Suggested ordering to maintain continuous testability:

### Phase 1: Input stream + Tokenizer (weeks 1–3)

1. Implement the input stream with UTF-8 support (skip full encoding for now)
2. Implement tokenizer states, starting with data state and working through tag open, tag name, attribute states
3. After each batch of states, run the tokenizer tests for the states you've implemented
4. Implement character reference parsing
5. Implement remaining states (RCDATA, RAWTEXT, script data, PLAINTEXT, etc.)
6. Target: pass all html5lib tokenizer tests

### Phase 2: DOM arena + Tree construction (weeks 4–7)

1. Implement the node arena and basic tree operations
2. Implement the tree construction dispatcher and insertion modes, starting with: Initial → BeforeHtml → BeforeHead → InHead → AfterHead → InBody (basic elements only) → AfterBody → AfterAfterBody
3. At this point, simple HTML documents like `<html><head><title>x</title></head><body><p>hello</p></body></html>` should produce correct trees
4. Implement the "in body" mode fully — this is the bulk of the work: all element types, implicit closes, formatting elements, the adoption agency algorithm
5. Implement table parsing modes: InTable, InTableText, InTableBody, InRow, InCell, InCaption, InColumnGroup — and foster parenting
6. Implement InSelect, InTemplate, InFrameset
7. Implement foreign content parsing (SVG, MathML)
8. Implement tree serialization in html5lib format
9. Target: pass all html5lib tree construction tests

### Phase 3: Polish (week 8)

1. Implement fragment parsing
2. Add full encoding support via `encoding_rs`
3. Fuzz the parser
4. Clean up public API surface
5. Documentation pass

---

## Spec References

All section numbers refer to the WHATWG HTML Living Standard: https://html.spec.whatwg.org/multipage/parsing.html

| Section | Topic |
|---|---|
| §13.2.2 | Parse errors |
| §13.2.3 | The input byte stream (encoding) |
| §13.2.3.5 | Preprocessing the input stream |
| §13.2.5 | Tokenization (80 states) |
| §13.2.5.73–78 | Character reference parsing |
| §13.2.6 | Tree construction |
| §13.2.6.1 | Creating and inserting nodes |
| §13.2.6.2 | Parsing elements that contain only text |
| §13.2.6.3 | Closing elements that have implied end tags |
| §13.2.6.4 | The rules for parsing tokens in HTML content (all insertion modes) |
| §13.2.6.4.7 | The adoption agency algorithm |
| §13.2.6.5 | The rules for parsing tokens in foreign content |
| §13.2.7 | The end |
| §13.3 | Serializing HTML fragments |
| §13.4 | Parsing HTML fragments |

WHATWG Encoding Standard (for `encoding_rs` reference): https://encoding.spec.whatwg.org/

---

## Success Criteria

1. 100% pass rate on html5lib-tests tokenizer suite
2. 100% pass rate on html5lib-tests tree construction suite
3. Zero panics under fuzzing (minimum 24 hours continuous fuzzing)
4. The DOM tree representation is sufficient to support DOM API bindings in Step 3 (no redesign needed)
5. Fragment parsing mode works correctly
6. All ~80 parse error types are reported with correct positions
7. `cargo build` with default features pulls in zero non-dev dependencies (or only `encoding_rs` if encoding is enabled)