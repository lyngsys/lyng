# Implementation Notes

This document records the reasoning behind the current implementation choices.

## Correctness First, Then Optimization

The parser follows the HTML algorithm closely enough that the code can be read against the spec. The project is not trying to compress the algorithm into a minimal abstraction layer at the expense of correctness or debuggability.

That shows up in several places:

- explicit insertion modes rather than a more generic event system
- explicit tokenizer states rather than regex-based tokenization
- dedicated open-elements and active-formatting structures rather than one generic stack type

The codebase is optimized for spec fidelity first and maintainable performance second.

## Why The DOM Uses An Arena

The DOM is stored in an arena with `NodeId` handles instead of `Rc<RefCell<Node>>`.

Reasons:

- tree construction needs frequent reparenting, insertion-before, and ancestor walks
- parser-owned mutation is simpler with indexed nodes than with shared mutable pointers
- arena storage is more cache-friendly
- the structure can later be exposed to higher layers without changing the parser core

`NodeId` is backed by `NonZeroU32`, which keeps `Option<NodeId>` compact and reduces node header size without complicating the API.

## Why The Input Stream Borrows When It Can

`InputStream` preprocesses the source into `Cow<'a, str>` segments.

Reasons:

- most inputs do not need CR normalization
- borrowing avoids an unconditional copy of the whole source
- inserted HTML still works by pushing an owned segment in front of the remaining input

This keeps the parser simple while reducing startup allocation cost.

## Why The Tokenizer Is Pull-Based

The tokenizer is not an independent producer running ahead of tree construction. Instead, the tree builder asks for one token at a time.

Reasons:

- HTML tree construction can change tokenizer state
- `script`, `style`, `title`, `textarea`, and fragment parsing all need that control
- a pull model matches the spec’s feedback loop more directly

This is a better fit than buffering a large token stream up front.

## Why Character Tokens Are Scalar

The tokenizer currently emits scalar character tokens rather than batching text into run tokens.

Reasons:

- the tree builder and many insertion-mode rules are specified in character-oriented terms
- scalar character tokens kept the implementation straightforward and predictable
- an earlier run-token experiment improved some allocation behavior but regressed end-to-end throughput in text-heavy and mixed workloads

The project keeps other memory improvements, but deliberately does not batch character tokens until that path can be reworked without losing performance.

## Why Start-Tag Payloads Move Into The DOM

When tree construction creates an element from a `StartTag`, it moves the tag name and attributes into the DOM node where possible instead of cloning them.

Reasons:

- the tokenizer already owns those strings
- the DOM needs owned strings anyway
- cloning every tag and attribute adds steady allocation overhead on element-heavy documents

This is a low-risk optimization because ownership already transfers at the parser boundary.

## Why Duplicate-Attribute Detection Uses A Thresholded Side Set

Duplicate attribute detection is handled in the tokenizer with a side `HashSet`, but only after a small threshold.

Reasons:

- tiny tags are common, and linear scan is fine there
- very wide or attacker-controlled tags should not become quadratic
- switching after a threshold keeps the common case cheap and fixes the pathological case

This is a targeted optimization with a clear worst-case benefit.

## Why The Project Stays Dependency-Light

The library keeps dependencies minimal on purpose.

Reasons:

- the parser algorithm is fully implementable in ordinary Rust
- avoiding parser-framework dependencies keeps behavior explicit
- fewer dependencies make the crate easier to audit and embed

The main optional exception is `encoding_rs`, because full WHATWG encoding support is a separate problem from HTML parsing and is already solved well there.

## Why Standard Hash Maps And Sets Are Good Enough Here

The project currently uses `std::collections::HashMap` and `HashSet`.

Reasons:

- the crate values simplicity and stability
- the parser’s biggest wins come from algorithmic choices, not from swapping one SwissTable implementation for another
- the hot paths that matter most are dominated by parser behavior, tree mutation, and spec-driven control flow

If hashing ever becomes a measurable bottleneck, the choice of hasher can be revisited separately from the parser design.

## Testing Philosophy

The project leans heavily on html5lib.

Reasons:

- the HTML parser’s hardest cases are malformed input and recovery behavior
- browser-compatible behavior matters more than unit-test-friendly abstractions
- html5lib exercises tokenizer state transitions, tree construction, namespaces, fragments, serialization, and encoding behavior

Unit tests still exist for local invariants, but html5lib compatibility is the main correctness bar.

## Future Work

Areas that would likely pay off next:

- more ergonomic public APIs for DOM traversal
- further targeted allocation reductions in tokenizer temporary buffers
- measured optimization of text handling without reintroducing the earlier throughput regressions
- more documentation around fragment parsing and script-related parser entry points
