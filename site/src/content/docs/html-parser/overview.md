---
title: HTML Parser overview
description: A WHATWG-compliant HTML parser with an arena-backed DOM.
---

The Lyng HTML parser implements the WHATWG HTML parsing algorithm and
materialises the result into an arena-backed DOM. It is designed to be
embedded inside a larger browser stack rather than used as a standalone
tool.

This page is a placeholder; deeper notes will land here as the parser
stabilises and the API surface settles.

## Highlights

- **Spec-driven.** Tokenizer and tree-construction states map directly to
  the WHATWG HTML Living Standard.
- **Arena-allocated DOM.** Nodes live in a contiguous arena, keyed by
  index, which keeps allocation cheap and tree traversal cache-friendly.
- **Tested against html5lib.** The repository runs the html5lib test
  suite as part of CI to catch regressions against real-world inputs.

## Where to go next

- [Lyng JS overview](/lyng-js/overview/) — the sibling JavaScript engine.
- [Blog](/blog/) — design notes and progress updates.
