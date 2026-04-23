# Lyng

Lyng is a Rust workspace for browser/runtime infrastructure. The repository currently has two
active implementation tracks:

- **Lyng JS** in [`crates/lyng-js/`](crates/lyng-js/README.md), the only JavaScript engine in
  the repository and the current project focus
- **`lyng-html-parser`** in [`crates/html_parser/`](crates/html_parser/README.md), a WHATWG-style
  HTML parser backed by an arena DOM

Current focus: Lyng JS is in Phase 6 ECMA-262 completion work. The active remaining tail is `6H`,
covering dynamic scope, proper tail calls, Annex B closure, and final conformance burn-down.

## Workspace Shape

- `crates/lyng-js/`: JS3 engine crates, integration tests, and runtime/compiler implementation
- `crates/html_parser/`: HTML tokenizer, tree builder, parse APIs, and html5lib-backed validation
- `crates/dom/`: arena-backed DOM used by the HTML parser
- `tools/`: html5lib, benchmark, and Test262 runners

Several other directories exist as placeholders for future browser subsystems, but they are not
active workspace crates today.

## Quick Check

```sh
cargo test -p lyng-js-tests
```

## Read Next

- [Lyng JS Overview](crates/lyng-js/README.md)
- [Lyng JS Docs Index](docs/lyng-js/README.md)
- [HTML Parser README](crates/html_parser/README.md)
