---
title: Getting started
description: Build and run the Lyng JS engine from source.
---

Lyng is a Cargo workspace. To build and run the JavaScript engine from a
local checkout:

```sh
git clone https://github.com/lyngsys/lyng.git
cd lyng
cargo build --release
```

The engine is exposed as a library crate today; integration entry points
and a `lyng` CLI will be documented here as they stabilize.

## Running the test suites

The repository pulls in [Test262](https://github.com/tc39/test262) and the
[html5lib test suites](https://github.com/html5lib/html5lib-tests) as git
submodules. After the initial clone, fetch them with:

```sh
git submodule update --init --recursive
```

Then run the standard test commands documented in the workspace
`README.md`.

## Where to go next

- [Lyng JS overview](/lyng-js/overview/) — the architectural tour.
- [HTML Parser overview](/html-parser/overview/) — the sibling project.
- [Blog](/blog/) — design notes and progress updates.
