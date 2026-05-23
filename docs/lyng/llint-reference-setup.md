# JSC LLInt reference capture — local-build setup

`lyng-bench capture-llint` uses three modes (`system`, `local`, `excerpt`).
This doc covers the `local` mode: building JSC from source so the binary
retains `_llint_op_*` symbols even when the system framework is stripped.

## When you need this

- macOS ships `JavaScriptCore.framework` but the symbols may be stripped.
  Run `nm /System/Library/Frameworks/JavaScriptCore.framework/Versions/Current/Helpers/jsc | grep _llint_op_add`
  to test. If it returns nothing, the system binary won't work for capture.
- Linux: most distributions don't ship `jsc`; you'll need a local build.

## Building WebKit/JSC

Clone the WebKit repository and build JSC in debug mode (symbols retained):
```sh
git clone https://github.com/WebKit/WebKit.git
cd WebKit
Tools/Scripts/build-jsc --debug
```

Output binary: `WebKitBuild/Debug/bin/jsc` (Linux) or
`WebKitBuild/Debug/JavaScriptCore.framework/Helpers/jsc` (macOS).

Verify symbols are present:
```sh
nm WebKitBuild/Debug/bin/jsc | grep _llint_op_add
```
Expected: one or more matches.

## Running capture-llint in local mode

```sh
cargo run --release -p lyng-bench -- capture-llint \
  --source local \
  --jsc-binary /path/to/WebKitBuild/Debug/bin/jsc \
  --opcodes op_add,op_mov,op_jmp,op_get_by_id,op_put_by_id,op_call,op_ret \
  --output reports/lyng/llint-reference
```

## Running capture-llint in excerpt mode (no build required)

Excerpt mode reads the offlineasm source files directly from a WebKit
source checkout — no compilation needed.

```sh
cargo run --release -p lyng-bench -- capture-llint \
  --source excerpt \
  --jsc-source /Users/sondre/dev/WebKit \
  --opcodes op_add,op_mov,op_jmp,op_get_by_id \
  --output reports/lyng/llint-reference
```

This produces source-level (offlineasm pseudo-code) reference rather than
concrete asm. It's always available; the trade-off is one level removed
from the actual machine code.
