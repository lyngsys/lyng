# VM-Internal Time-Attribution Profiler + samply Drill-Down — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a feature-gated statistical sampler that gives per-(opcode × fast/slow-path) *time* attribution on V8 v7 workloads, plus a samply wrapper to replace the broken flamegraph pipeline.

**Architecture:** The engine already keeps per-opcode dispatch/slow-path *counts* in `DispatchCounters` (a `#[repr(C)]` box reached from asm via a fixed `Vm` offset), all behind the `diagnostic-counters` cargo feature. We add a 4th field — `current_opcode: AtomicU64` at offset 6144 — that the asm dispatch prologue publishes on every dispatch (plain opcode = fast lane; `op | 0x100` = slow lane, published at the slow-semantic site). A background sampler thread in the bench tool reads that atomic at a fixed interval into a fast/slow histogram. Combined with the exact dispatch counts, this yields time-share and cost-per-dispatch. samply is wired in as an ad-hoc function-level microscope.

**Tech Stack:** Rust, aarch64 inline-asm DSL (`crates/vm-dsl`, `crates/vm/src/dsl/backend/aarch64`), `lyng-bench` tool, `std::thread` + `AtomicU64`, `serde_json`, external `samply`.

---

## Background the implementer must know

- **Feature gate.** Everything instrumentation-related is behind `#[cfg(feature = "diagnostic-counters")]` (`crates/vm/Cargo.toml:20`, `default = []`). The `lyng` production binary never enables it; `lyng-bench` always does (its `Cargo.toml` sets `lyng-vm = { workspace = true, features = ["diagnostic-counters"] }`). **Hard rule: when the feature is OFF, none of the new code compiles and the asm hot path is byte-identical to today.** Never add a runtime branch where a `#[cfg]` belongs.
- **Why `AtomicU64` (not a plain `u64` like the existing banks).** The existing `dispatch`/`slow_semantic`/`slow_safepoint` banks are plain `[u64; 256]` because they're written by asm and read by Rust only *after* the run finishes (single-threaded). The new `current_opcode` cell is read *concurrently* by the sampler thread while the VM writes it, so it MUST be an `AtomicU64` to avoid a data race (UB). An aligned 64-bit asm `STR` is exactly what `AtomicU64::store(Relaxed)` lowers to, so asm writing the cell + the sampler doing `load(Relaxed)` is the correct, well-defined pattern.
- **Why a raw pointer is needed for the sampler.** `DispatchCounters` is `Box`-allocated, so its heap address is stable for the VM's lifetime. The sampler thread holds a `*const AtomicU64` into that allocation. The borrow checker can't model "asm writes this while a thread reads it," so we capture a raw pointer and uphold the lifetime by **joining the sampler thread before the run's counters are dropped** (`SamplingProfiler::stop`, plus a `Drop` safety net that also joins).
- **aarch64 only.** There is exactly one asm backend (`crates/vm/src/dsl/backend/aarch64/`); the target machine is Apple Silicon. The asm publish is aarch64-only, which is fine — the profiler is a bench-only tool that runs in-process in `lyng-bench` (always aarch64 here).
- **Run with `--release`.** All measurement runs use `cargo run --release -p lyng-bench`. Debug builds print a warning and are meaningless.
- **Deviation from the spec (intentional, less surface).** The design doc proposed a `Vm::with_sampling_profiler` builder hook. This plan does NOT add one. The in-process bench driver already owns the `Vm` and its `DispatchCounters` (boxed, stable address), so it can borrow the `current_opcode` cell and drive the sampler thread itself around the `.run()` call. No new `Vm` field, no new builder hook, no swap plumbing — the only engine change is the 4th `DispatchCounters` field and the asm publish. If a future caller outside the bench tool needs sampling, a builder hook can be added then (YAGNI now).

---

## File Structure

**Engine (crate `lyng-vm`):**
- `crates/vm/src/opcode_counts.rs` (modify) — add `current_opcode: AtomicU64` field + consts + accessor + decode helper to `DispatchCounters`.
- `crates/vm/src/dsl/backend/aarch64/counters.rs` (modify) — publish current opcode in `inc_dispatch_counter!` (fast) and `inc_slow_semantic_counter!` (slow).
- `crates/vm/src/sampling_profiler.rs` (create) — `SamplingProfiler` + `SampleHistogram`, gated.
- `crates/vm/src/lib.rs` (modify) — declare + export the new module under the feature.
- `crates/vm/tests/dispatch_counters_layout.rs` (modify) — pin new size/offset.

**Bench tool (crate `lyng-bench`):**
- `tools/lyng-bench/src/profile.rs` (create) — in-process sampled driver + MD/JSON report + samply hook.
- `tools/lyng-bench/src/v8suite.rs` (modify) — widen visibility of `V8_WORKLOADS`, `build_count_harness`, `read_file`, `ensure_path_exists`, `write_output`, `default_v8_root` to `pub(crate)` for reuse.
- `tools/lyng-bench/src/cli.rs` (modify) — add `Profile` command + parse + help + tests.
- `tools/lyng-bench/src/lib.rs` (modify) — declare module + dispatch.

**Docs:**
- `reports/lyng/llint-parity-state-of-engine.md` (modify) — list the new command + evidence file.
- `reports/lyng/v8-raytrace-profile-2026-05-30.md` (create) — first real artifact from the new tool.

---

## Task 1: Add `current_opcode` cell + consts + accessors to `DispatchCounters`

**Files:**
- Modify: `crates/vm/src/opcode_counts.rs`
- Modify: `crates/vm/tests/dispatch_counters_layout.rs`

- [ ] **Step 1: Update the layout test to the new expected size/offset (failing test first)**

In `crates/vm/tests/dispatch_counters_layout.rs`, replace the two test bodies:

```rust
#[test]
fn dispatch_counters_size_is_expected() {
    // 3 banks * 256 entries * 8 bytes = 6144 bytes, plus the 8-byte
    // current_opcode AtomicU64 cell at offset 6144 = 6152 bytes.
    assert_eq!(size_of::<DispatchCounters>(), 3 * 256 * 8 + 8);
}

#[test]
fn dispatch_counters_field_offsets_are_stable() {
    assert_eq!(offset_of!(DispatchCounters, dispatch), 0);
    assert_eq!(offset_of!(DispatchCounters, slow_semantic), 256 * 8);
    assert_eq!(offset_of!(DispatchCounters, slow_safepoint), 512 * 8);
    // The asm publish stores to [counter_base, #6144]; pin it here so a
    // struct re-order can't silently break it.
    assert_eq!(offset_of!(DispatchCounters, current_opcode), 6144);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p lyng-vm --features diagnostic-counters --test dispatch_counters_layout`
Expected: FAIL — `no field current_opcode on type DispatchCounters` (compile error) / size mismatch.

- [ ] **Step 3: Add the field, consts, accessor, and decode helper**

In `crates/vm/src/opcode_counts.rs`, add to the imports at the top:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
```

Add module-level consts and a free decode helper (place just above `pub struct DispatchCounters`):

```rust
/// Sentinel stored in `DispatchCounters::current_opcode` when no opcode
/// has been dispatched yet (set at construction and on `reset`). Samples
/// that observe this value are attributed to the "non-opcode / native"
/// bucket rather than to any opcode.
pub const CURRENT_OPCODE_IDLE: u64 = u64::MAX;

/// Bit OR-ed into the published opcode byte to mark "currently in this
/// opcode's slow (semantic) path". Opcodes occupy bits 0..=7 (values
/// 0..=255), so bit 8 is a free lane flag.
pub const CURRENT_OPCODE_SLOW_BIT: u64 = 0x100;

/// Decode a raw `current_opcode` cell value into `(opcode, in_slow_path)`.
///
/// Returns `None` for the idle sentinel or any byte that is not a known
/// opcode discriminant (defensive — a torn or stale read should not panic
/// the sampler).
#[must_use]
pub fn decode_current_opcode(raw: u64) -> Option<(Opcode, bool)> {
    if raw == CURRENT_OPCODE_IDLE {
        return None;
    }
    let in_slow = raw & CURRENT_OPCODE_SLOW_BIT != 0;
    let byte = u8::try_from(raw & 0xFF).ok()?;
    Opcode::from_byte(byte).map(|opcode| (opcode, in_slow))
}
```

Add the field to the struct (after `slow_safepoint`), and update the doc comment's bank list to mention it:

```rust
#[repr(C)]
pub struct DispatchCounters {
    pub dispatch: [u64; 256],
    pub slow_semantic: [u64; 256],
    pub slow_safepoint: [u64; 256],
    /// Live opcode published by the asm dispatch prologue (fast lane) and
    /// the slow-semantic counter site (slow lane, `| CURRENT_OPCODE_SLOW_BIT`).
    /// Read concurrently by the sampling profiler, hence `AtomicU64` rather
    /// than a plain `u64` like the count banks above. At offset 6144.
    pub current_opcode: AtomicU64,
}
```

Update `DispatchCounters::new()` to allocate the extra slot and initialize the sentinel. Replace the existing body:

```rust
    pub fn new() -> Box<Self> {
        // 3 * 256 count slots + 1 current_opcode cell. Allocate via a
        // zeroed Vec<u64> -> Box conversion to avoid a 6 KB stack copy.
        // SAFETY: `Box<[u64; 3 * 256 + 1]>` has identical layout to
        // `Box<DispatchCounters>`: both are #[repr(C)] contiguous
        // 6152-byte, 8-byte-aligned allocations (AtomicU64 has the same
        // size/align as u64). Verified in tests/dispatch_counters_layout.rs.
        let zeros: Vec<u64> = vec![0; 3 * 256 + 1];
        let boxed_slice: Box<[u64]> = zeros.into_boxed_slice();
        let raw: *mut u64 = Box::into_raw(boxed_slice).cast::<u64>();
        let mut counters = unsafe { Box::from_raw(raw.cast::<Self>()) };
        counters.current_opcode = AtomicU64::new(CURRENT_OPCODE_IDLE);
        counters
    }
```

Update `DispatchCounters::reset()` to also reset the cell:

```rust
    pub fn reset(&mut self) {
        self.dispatch.fill(0);
        self.slow_semantic.fill(0);
        self.slow_safepoint.fill(0);
        self.current_opcode.store(CURRENT_OPCODE_IDLE, Ordering::Relaxed);
    }
```

Add an accessor method in the `impl DispatchCounters` block (after `slow_safepoint_count`):

```rust
    /// Borrow the live `current_opcode` cell. The sampling profiler turns
    /// this into a raw pointer; the cell stays at a stable heap address for
    /// the lifetime of the owning `DispatchCounters` box.
    #[must_use]
    pub const fn current_opcode_cell(&self) -> &AtomicU64 {
        &self.current_opcode
    }
```

Update `impl Default for DispatchCounters` to construct the new field:

```rust
impl Default for DispatchCounters {
    fn default() -> Self {
        Self {
            dispatch: [0; 256],
            slow_semantic: [0; 256],
            slow_safepoint: [0; 256],
            current_opcode: AtomicU64::new(CURRENT_OPCODE_IDLE),
        }
    }
}
```

- [ ] **Step 4: Run the layout test to verify it passes**

Run: `cargo test -p lyng-vm --features diagnostic-counters --test dispatch_counters_layout`
Expected: PASS (both tests).

- [ ] **Step 5: Add a decode round-trip unit test**

In `crates/vm/src/opcode_counts.rs`, find the existing `#[cfg(test)] mod tests` block (or add one at the end of the file if none exists) and add:

```rust
#[cfg(test)]
mod current_opcode_tests {
    use super::{decode_current_opcode, CURRENT_OPCODE_IDLE, CURRENT_OPCODE_SLOW_BIT};
    use lyng_bytecode::Opcode;

    #[test]
    fn idle_sentinel_decodes_to_none() {
        assert_eq!(decode_current_opcode(CURRENT_OPCODE_IDLE), None);
    }

    #[test]
    fn fast_lane_decodes_to_opcode_without_slow_flag() {
        let op = Opcode::from_byte(0).expect("opcode 0 exists");
        let raw = u64::from(op as u8);
        assert_eq!(decode_current_opcode(raw), Some((op, false)));
    }

    #[test]
    fn slow_bit_decodes_to_slow_lane() {
        let op = Opcode::from_byte(0).expect("opcode 0 exists");
        let raw = u64::from(op as u8) | CURRENT_OPCODE_SLOW_BIT;
        assert_eq!(decode_current_opcode(raw), Some((op, true)));
    }
}
```

- [ ] **Step 6: Run the new unit tests**

Run: `cargo test -p lyng-vm --features diagnostic-counters current_opcode_tests`
Expected: PASS (3 tests).

- [ ] **Step 7: Verify the feature-off build still compiles unchanged**

Run: `cargo build -p lyng-vm`
Expected: success (the new field/consts are not gated, but they add no asm and no cost; they're only *read* by gated code).

- [ ] **Step 8: Commit**

```bash
git add crates/vm/src/opcode_counts.rs crates/vm/tests/dispatch_counters_layout.rs
git commit -m "feat(vm): add current_opcode cell + decode to DispatchCounters

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Publish the current opcode from the asm dispatch prologue

**Files:**
- Modify: `crates/vm/src/dsl/backend/aarch64/counters.rs`

This task has no Rust unit test — the asm is validated by (a) the build succeeding under the feature, and (b) Task 4's statistical end-to-end test. We add an inline integration check that the cell is non-idle after a run.

- [ ] **Step 1: Extend `inc_dispatch_counter!` (fast lane) to publish the opcode**

In `crates/vm/src/dsl/backend/aarch64/counters.rs`, in the `#[cfg(feature = "diagnostic-counters")] macro_rules! inc_dispatch_counter`, replace the `concat!(...)` body with (the first four lines are unchanged; the last two are new):

```rust
        concat!(
            "ldr    x9, [x22, {vm_counter_base}]\n",
            "ldr    x10, [x9, #",
            stringify!($opcode_byte),
            " * 8]\n",
            "add    x10, x10, #1\n",
            "str    x10, [x9, #",
            stringify!($opcode_byte),
            " * 8]\n",
            // Publish the live opcode (fast lane: no slow bit) into the
            // current_opcode cell at offset 6144. x9 still holds the
            // DispatchCounters pointer; x10 is free after the store-back.
            "mov    x10, #",
            stringify!($opcode_byte),
            "\n",
            "str    x10, [x9, #6144]\n",
        )
```

- [ ] **Step 2: Extend `inc_slow_semantic_counter!` (slow lane) to publish `op | 0x100`**

In the same file, in `#[cfg(feature = "diagnostic-counters")] macro_rules! inc_slow_semantic_counter`, replace the `concat!(...)` body with (first four lines unchanged; last two new):

```rust
        concat!(
            "ldr    x16, [x22, {vm_counter_base}]\n",
            "ldr    x17, [x16, #",
            stringify!($opcode_byte),
            " * 8 + 2048]\n",
            "add    x17, x17, #1\n",
            "str    x17, [x16, #",
            stringify!($opcode_byte),
            " * 8 + 2048]\n",
            // Re-publish the live opcode with the slow-lane bit (0x100) so
            // samples taken while the semantic slow path runs are attributed
            // to this opcode's slow lane. x16 = DispatchCounters ptr, x17 free.
            "mov    x17, #",
            stringify!($opcode_byte),
            " + 256\n",
            "str    x17, [x16, #6144]\n",
        )
```

Also update the module doc comment near the top: in the "Emitted shape" section and the scratch-register notes, add a sentence that both increment macros additionally publish the live opcode byte to the `current_opcode` cell at offset 6144 (fast lane plain, slow lane `+ 256`), consumed by the sampling profiler.

- [ ] **Step 3: Build under the feature to validate asm assembles**

Run: `cargo build --release -p lyng-vm --features diagnostic-counters`
Expected: success. If the assembler rejects `mov x17, #5 + 256`, the build fails here — in that case wrap the slow expression as `" + 256"` is already separate; if needed change to a precomputed approach, but LLVM folds `#<int> + <int>` constant expressions (the existing offset operands rely on the same folding), so this should assemble cleanly.

- [ ] **Step 4: Add an end-to-end "publish fired" integration test**

Create the assertion inside the bench tool is premature (profile.rs doesn't exist yet); instead add a focused test in the vm crate that runs a trivial script through the existing counted path. Add to `crates/vm/src/tests/core.rs` (which already has `#[cfg(feature = "diagnostic-counters")]` tests):

```rust
#[cfg(feature = "diagnostic-counters")]
#[test]
fn current_opcode_cell_is_published_after_a_run() {
    use crate::opcode_counts::CURRENT_OPCODE_IDLE;
    use std::sync::atomic::Ordering;

    // Build + run the smallest possible script through the VM, then assert
    // the asm dispatch prologue published at least one opcode (cell moved
    // off the idle sentinel). This proves the `str [x9, #6144]` fired.
    let mut harness = crate::tests::core::CountedRun::new("var x = 1 + 2;");
    harness.run();
    let raw = harness
        .vm()
        .opcode_counters()
        .dispatch_banks()
        .current_opcode_cell()
        .load(Ordering::Relaxed);
    assert_ne!(raw, CURRENT_OPCODE_IDLE, "asm prologue should have published an opcode");
}
```

> **Implementer note:** `crates/vm/src/tests/core.rs` already constructs and runs a VM in its existing `diagnostic-counters` tests (see the tests around lines 238–909). Reuse whatever local harness/helper those tests use to build+run a script rather than introducing `CountedRun` if it does not exist — the assertion that matters is: after running any script, `dispatch_banks().current_opcode_cell().load(Relaxed) != CURRENT_OPCODE_IDLE`. Match the file's existing setup idiom.

- [ ] **Step 5: Run the integration test**

Run: `cargo test -p lyng-vm --features diagnostic-counters current_opcode_cell_is_published`
Expected: PASS.

- [ ] **Step 6: Verify existing counter tests still pass (no regression to counts)**

Run: `cargo test -p lyng-vm --features diagnostic-counters`
Expected: PASS (all). The dispatch/slow counts are unchanged; only an extra publish store was added.

- [ ] **Step 7: Verify feature-off build is unaffected**

Run: `cargo build -p lyng-vm && cargo build --release -p lyng-cli`
Expected: success. The feature-off macro arms are untouched (still empty strings), so the `lyng` binary's hot path is byte-identical.

- [ ] **Step 8: Commit**

```bash
git add crates/vm/src/dsl/backend/aarch64/counters.rs crates/vm/src/tests/core.rs
git commit -m "feat(vm): publish live opcode to current_opcode cell in asm prologue

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: `SamplingProfiler` + `SampleHistogram` (gated module)

**Files:**
- Create: `crates/vm/src/sampling_profiler.rs`
- Modify: `crates/vm/src/lib.rs`

- [ ] **Step 1: Create the module with the histogram, profiler, and unit tests**

Create `crates/vm/src/sampling_profiler.rs`:

```rust
//! Statistical sampling profiler for opcode time-attribution.
//!
//! Gated behind `diagnostic-counters`. A background thread reads the
//! `DispatchCounters::current_opcode` atomic at a fixed interval and bins
//! each observation into a fast/slow per-opcode histogram. Combined with
//! the exact dispatch counts, this gives time-share and cost-per-dispatch.
//!
//! ## Lifetime / safety contract
//!
//! `start` captures a raw `*const AtomicU64` into the (boxed, stable-address)
//! `DispatchCounters` allocation. The borrow checker cannot model "asm writes
//! the cell on one thread while the sampler reads it on another", so the
//! caller MUST keep the owning counters alive until `stop` returns. `stop`
//! joins the sampler thread; `Drop` is a safety net that also signals + joins
//! if `stop` was not called. Construct the profiler AFTER the counters and
//! drop it BEFORE them (natural lexical scope order in the bench driver).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use lyng_bytecode::{Opcode, OPCODE_COUNT};

use crate::opcode_counts::decode_current_opcode;

/// Raw pointer to the shared `current_opcode` cell. `Send` because the cell
/// is an `AtomicU64` at a stable heap address that outlives the thread (see
/// the module-level safety contract).
struct CellPtr(*const AtomicU64);

// SAFETY: the pointee is an AtomicU64 (all accesses are atomic) at a stable
// address kept alive by the caller until the thread is joined.
unsafe impl Send for CellPtr {}

/// Per-opcode fast/slow sample tallies plus a non-opcode bucket.
#[derive(Clone, Debug)]
pub struct SampleHistogram {
    fast: Vec<u64>,
    slow: Vec<u64>,
    non_opcode: u64,
    total: u64,
}

impl SampleHistogram {
    fn zeroed() -> Self {
        let len = OPCODE_COUNT as usize;
        Self {
            fast: vec![0; len],
            slow: vec![0; len],
            non_opcode: 0,
            total: 0,
        }
    }

    /// Merge another histogram into this one (used to sum across samples).
    pub fn merge(&mut self, other: &Self) {
        for index in 0..self.fast.len() {
            self.fast[index] = self.fast[index].saturating_add(other.fast[index]);
            self.slow[index] = self.slow[index].saturating_add(other.slow[index]);
        }
        self.non_opcode = self.non_opcode.saturating_add(other.non_opcode);
        self.total = self.total.saturating_add(other.total);
    }

    #[must_use]
    pub fn fast(&self, opcode: Opcode) -> u64 {
        self.fast.get(usize::from(opcode as u8)).copied().unwrap_or(0)
    }

    #[must_use]
    pub fn slow(&self, opcode: Opcode) -> u64 {
        self.slow.get(usize::from(opcode as u8)).copied().unwrap_or(0)
    }

    /// Total samples attributed to an opcode (fast + slow lanes).
    #[must_use]
    pub fn samples(&self, opcode: Opcode) -> u64 {
        self.fast(opcode).saturating_add(self.slow(opcode))
    }

    /// Samples that observed the idle sentinel (time outside any dispatched
    /// opcode — e.g. before the first dispatch or after the run ended).
    #[must_use]
    pub const fn non_opcode(&self) -> u64 {
        self.non_opcode
    }

    /// Total samples taken (sum of all lanes + non_opcode).
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.total
    }

    /// Iterate opcodes that received at least one sample.
    pub fn iter(&self) -> impl Iterator<Item = (Opcode, u64, u64)> + '_ {
        (0..self.fast.len()).filter_map(move |index| {
            let raw = u8::try_from(index).ok()?;
            let opcode = Opcode::from_byte(raw)?;
            let fast = self.fast[index];
            let slow = self.slow[index];
            if fast == 0 && slow == 0 {
                return None;
            }
            Some((opcode, fast, slow))
        })
    }
}

/// A running sampler. Stop it (or drop it) to collect the histogram.
pub struct SamplingProfiler {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<SampleHistogram>>,
}

impl SamplingProfiler {
    /// Start sampling `cell` every `interval`.
    ///
    /// See the module-level safety contract: `cell` must outlive the profiler,
    /// and the profiler must be stopped/dropped before the cell is freed.
    #[must_use]
    pub fn start(cell: &AtomicU64, interval: Duration) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let ptr = CellPtr(cell as *const AtomicU64);
        let handle = std::thread::spawn(move || {
            let cell_ptr = ptr; // move the Send wrapper into the thread
            let mut hist = SampleHistogram::zeroed();
            while !thread_stop.load(Ordering::Acquire) {
                // SAFETY: pointee is a live AtomicU64 per the safety contract.
                let raw = unsafe { (*cell_ptr.0).load(Ordering::Relaxed) };
                match decode_current_opcode(raw) {
                    Some((opcode, in_slow)) => {
                        let index = usize::from(opcode as u8);
                        if in_slow {
                            hist.slow[index] = hist.slow[index].saturating_add(1);
                        } else {
                            hist.fast[index] = hist.fast[index].saturating_add(1);
                        }
                    }
                    None => hist.non_opcode = hist.non_opcode.saturating_add(1),
                }
                hist.total = hist.total.saturating_add(1);
                std::thread::sleep(interval);
            }
            hist
        });
        Self {
            stop,
            handle: Some(handle),
        }
    }

    /// Stop sampling, join the thread, and return the collected histogram.
    #[must_use]
    pub fn stop(mut self) -> SampleHistogram {
        self.stop.store(true, Ordering::Release);
        self.handle
            .take()
            .map(|handle| handle.join().unwrap_or_else(|_| SampleHistogram::zeroed()))
            .unwrap_or_else(SampleHistogram::zeroed)
    }
}

impl Drop for SamplingProfiler {
    fn drop(&mut self) {
        // Safety net: if stop() was not called, still signal + join so the
        // thread cannot outlive the cell it reads.
        self.stop.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opcode_counts::{CURRENT_OPCODE_IDLE, CURRENT_OPCODE_SLOW_BIT};

    fn sample_constant_cell(value: u64) -> SampleHistogram {
        let cell = AtomicU64::new(value);
        let profiler = SamplingProfiler::start(&cell, Duration::from_millis(1));
        // Let the sampler take several ticks against a constant cell.
        std::thread::sleep(Duration::from_millis(40));
        let hist = profiler.stop();
        // Keep `cell` alive until after stop() joined the thread.
        std::hint::black_box(&cell);
        hist
    }

    #[test]
    fn constant_fast_cell_attributes_all_samples_to_one_fast_lane() {
        let op = Opcode::from_byte(0).expect("opcode 0 exists");
        let hist = sample_constant_cell(u64::from(op as u8));
        assert!(hist.total() > 0, "sampler should have taken ticks");
        assert_eq!(hist.fast(op), hist.total());
        assert_eq!(hist.slow(op), 0);
        assert_eq!(hist.non_opcode(), 0);
    }

    #[test]
    fn constant_slow_cell_attributes_all_samples_to_slow_lane() {
        let op = Opcode::from_byte(0).expect("opcode 0 exists");
        let hist = sample_constant_cell(u64::from(op as u8) | CURRENT_OPCODE_SLOW_BIT);
        assert!(hist.total() > 0);
        assert_eq!(hist.slow(op), hist.total());
        assert_eq!(hist.fast(op), 0);
    }

    #[test]
    fn idle_cell_attributes_all_samples_to_non_opcode() {
        let hist = sample_constant_cell(CURRENT_OPCODE_IDLE);
        assert!(hist.total() > 0);
        assert_eq!(hist.non_opcode(), hist.total());
    }

    #[test]
    fn merge_sums_lanes_and_totals() {
        let op = Opcode::from_byte(0).expect("opcode 0 exists");
        let mut a = sample_constant_cell(u64::from(op as u8));
        let b = sample_constant_cell(u64::from(op as u8));
        let expected = a.total() + b.total();
        a.merge(&b);
        assert_eq!(a.total(), expected);
        assert_eq!(a.fast(op), expected);
    }
}
```

- [ ] **Step 2: Wire the module into `lib.rs` under the feature**

In `crates/vm/src/lib.rs`, add the module declaration near the other `#[cfg(feature = "diagnostic-counters")]` items (e.g. right after `mod opcode_counts;`):

```rust
#[cfg(feature = "diagnostic-counters")]
mod sampling_profiler;
```

And add a re-export next to the existing gated `pub use opcode_counts::{...}` block:

```rust
#[cfg(feature = "diagnostic-counters")]
pub use sampling_profiler::{SampleHistogram, SamplingProfiler};
```

Also make the new `opcode_counts` decode helper + consts reachable. In the existing `#[cfg(feature = "diagnostic-counters")] pub use opcode_counts::{...}` list, add `decode_current_opcode`, `CURRENT_OPCODE_IDLE`, and `CURRENT_OPCODE_SLOW_BIT`:

```rust
#[cfg(feature = "diagnostic-counters")]
pub use opcode_counts::{
    decode_current_opcode, CallArgumentCopyCounts, DispatchCounters, OpcodeCounters,
    OpcodeDispatchCount, OpcodeDispatchCounts, CURRENT_OPCODE_IDLE, CURRENT_OPCODE_SLOW_BIT,
};
```

- [ ] **Step 3: Run the sampler unit tests**

Run: `cargo test -p lyng-vm --features diagnostic-counters sampling_profiler`
Expected: PASS (4 tests). These are deterministic because the cell is constant during each sampling window, so every tick lands in the same bucket.

- [ ] **Step 4: Confirm feature-off build does not compile the module**

Run: `cargo build -p lyng-vm`
Expected: success; `sampling_profiler` is not compiled (gated).

- [ ] **Step 5: Commit**

```bash
git add crates/vm/src/sampling_profiler.rs crates/vm/src/lib.rs
git commit -m "feat(vm): add SamplingProfiler + SampleHistogram (gated)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: `profile` driver + report in `lyng-bench`

**Files:**
- Modify: `tools/lyng-bench/src/v8suite.rs` (widen helper visibility)
- Create: `tools/lyng-bench/src/profile.rs`

- [ ] **Step 1: Widen visibility of reused v8suite helpers**

In `tools/lyng-bench/src/v8suite.rs`, change these item visibilities from private (`fn`) to `pub(crate) fn` (leave bodies unchanged):

- `fn build_count_harness` → `pub(crate) fn build_count_harness`
- `fn read_file` → `pub(crate) fn read_file`
- `fn ensure_path_exists` → `pub(crate) fn ensure_path_exists`
- `fn write_output` → `pub(crate) fn write_output`
- `fn default_v8_root` → `pub(crate) fn default_v8_root`

(`V8_WORKLOADS` and `struct V8Workload` are already `pub(crate)`.)

- [ ] **Step 2: Create `profile.rs` with options parsing + run skeleton + a failing test**

Create `tools/lyng-bench/src/profile.rs`:

```rust
//! `lyng-bench profile` — VM-internal time-attribution profiler.
//!
//! Runs V8 v7 workloads in-process with a statistical sampler that bins
//! samples by (opcode x fast/slow path), then emits a ranked time-attribution
//! report (Markdown + JSON, schema `lyng-bench/profile/v1`). Optionally also
//! captures a samply profile for function-level drill-down (`--samply`).

use std::path::Path;
use std::time::Duration;

use lyng_builtins::BootstrapMode;
use lyng_bytecode::Opcode;
use lyng_common::{AtomTable, SourceId};
use lyng_compiler::compile_script;
use lyng_env::Runtime;
use lyng_host::NoopHostHooks;
use lyng_parser::parse_script;
use lyng_sema::analyze_script;
use lyng_vm::{OpcodeDispatchCounts, SampleHistogram, SamplingProfiler, Vm};
use serde_json::{json, Value};
use std::hint::black_box;

use crate::v8suite::{
    build_count_harness, default_v8_root, ensure_path_exists, read_file, write_output,
    V8Workload, V8_WORKLOADS,
};

const DEFAULT_INTERVAL_US: u64 = 200;
const DEFAULT_SAMPLES: usize = 1;

pub(crate) struct Options {
    pub report_path: String,
    pub json_path: String,
    pub v8_root: String,
    pub samples: usize,
    pub interval_us: u64,
    pub filter: Option<String>,
    pub samply: bool,
    pub lyng_bin: String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            report_path: "reports/lyng/profile.md".to_string(),
            json_path: "reports/lyng/profile.json".to_string(),
            v8_root: default_v8_root(),
            samples: DEFAULT_SAMPLES,
            interval_us: DEFAULT_INTERVAL_US,
            filter: None,
            samply: false,
            lyng_bin: "target/release/lyng".to_string(),
        }
    }
}

pub(crate) fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut iter = args.iter().cloned();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--report" => {
                options.report_path = iter.next().ok_or("--report needs a path")?;
            }
            "--json" => {
                options.json_path = iter.next().ok_or("--json needs a path")?;
            }
            "--v8-root" => {
                options.v8_root = iter.next().ok_or("--v8-root needs a path")?;
            }
            "--filter" => {
                options.filter = Some(iter.next().ok_or("--filter needs a value")?);
            }
            "--samples" => {
                options.samples = iter
                    .next()
                    .ok_or("--samples needs a value")?
                    .parse()
                    .map_err(|_| "--samples must be a positive integer".to_string())?;
            }
            "--interval-us" => {
                options.interval_us = iter
                    .next()
                    .ok_or("--interval-us needs a value")?
                    .parse()
                    .map_err(|_| "--interval-us must be a positive integer".to_string())?;
            }
            "--lyng-bin" => {
                options.lyng_bin = iter.next().ok_or("--lyng-bin needs a path")?;
            }
            "--samply" => options.samply = true,
            "--help" | "-h" => return Err(help_text()),
            other => return Err(format!("unknown profile option: {other}\n\n{}", help_text())),
        }
    }
    if options.samples == 0 {
        return Err("--samples must be >= 1".to_string());
    }
    if options.interval_us == 0 {
        return Err("--interval-us must be >= 1".to_string());
    }
    Ok(options)
}

pub(crate) fn help_text() -> String {
    [
        "Usage: lyng-bench profile [options]",
        "",
        "Options:",
        "  --filter <name>      Only profile the named workload (e.g. RayTrace)",
        "  --samples <n>        Sampled runs to sum per workload (default: 1)",
        "  --interval-us <n>    Sampler tick interval in microseconds (default: 200)",
        "  --samply             Also capture a samply profile per workload",
        "  --lyng-bin <path>    lyng binary for --samply (default: target/release/lyng)",
        "  --v8-root <path>     V8 v7 sources dir",
        "  --report <path>      Markdown report path (default: reports/lyng/profile.md)",
        "  --json <path>        JSON report path (default: reports/lyng/profile.json)",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|p| (*p).to_string()).collect()
    }

    #[test]
    fn defaults_are_applied_when_no_args() {
        let options = parse_options(&[]).unwrap();
        assert_eq!(options.samples, 1);
        assert_eq!(options.interval_us, 200);
        assert!(!options.samply);
    }

    #[test]
    fn parses_filter_interval_and_samply() {
        let options =
            parse_options(&args(&["--filter", "RayTrace", "--interval-us", "100", "--samply"]))
                .unwrap();
        assert_eq!(options.filter.as_deref(), Some("RayTrace"));
        assert_eq!(options.interval_us, 100);
        assert!(options.samply);
    }

    #[test]
    fn zero_samples_is_rejected() {
        assert!(parse_options(&args(&["--samples", "0"])).is_err());
    }
}
```

- [ ] **Step 3: Run the options tests to verify they pass**

Run: `cargo test -p lyng-bench profile::tests`
Expected: PASS (3 tests). (`run` is not implemented yet; that's fine — these test parsing only.)

- [ ] **Step 4: Implement the sampled in-process driver**

Append to `tools/lyng-bench/src/profile.rs`:

```rust
/// Per-workload accumulated result: summed dispatch counts + summed histogram.
struct WorkloadProfile {
    workload: V8Workload,
    dispatch: OpcodeDispatchCounts,
    histogram: SampleHistogram,
}

/// Runs the profile suite and writes Markdown + JSON reports.
///
/// # Errors
/// Returns an error when CLI parsing fails, the V8 root is missing, a
/// workload fails to compile, or a workload errors on every sample.
pub fn run(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;
    if cfg!(debug_assertions) {
        eprintln!("warning: build with --release for meaningful measurements");
    }
    ensure_path_exists(&options.v8_root, "v8 benchmark root")?;
    let base_js = read_file(&Path::new(&options.v8_root).join("base.js"))?;

    let workloads: Vec<&V8Workload> = V8_WORKLOADS
        .iter()
        .filter(|w| {
            options
                .filter
                .as_ref()
                .is_none_or(|needle| w.name.eq_ignore_ascii_case(needle))
        })
        .collect();
    if workloads.is_empty() {
        return Err(format!(
            "no benchmarks matched filter `{}`. known: {}",
            options.filter.as_deref().unwrap_or("<none>"),
            V8_WORKLOADS.iter().map(|w| w.name).collect::<Vec<_>>().join(", ")
        ));
    }

    let mut profiles = Vec::with_capacity(workloads.len());
    for (index, workload) in workloads.iter().enumerate() {
        let benchmark_js = read_file(&Path::new(&options.v8_root).join(workload.file))?;
        let harness = build_count_harness(&base_js, &benchmark_js);
        let source_id = SourceId::new(
            u32::try_from(index + 1).map_err(|_| "workload count exceeds SourceId range".to_string())?,
        );
        let profile = profile_workload(workload, &harness, source_id, &options)?;
        if options.samply {
            samply::capture(workload, &harness, &options)?;
        }
        profiles.push(profile);
    }

    write_output(&options.report_path, &render_markdown(&options, &profiles))?;
    write_output(
        &options.json_path,
        &serde_json::to_string_pretty(&render_json(&options, &profiles))
            .map_err(|e| format!("failed to render profile JSON: {e}"))?,
    )?;
    print_summary(&profiles, &options);
    Ok(())
}

fn profile_workload(
    workload: &V8Workload,
    harness: &str,
    source_id: SourceId,
    options: &Options,
) -> Result<WorkloadProfile, String> {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, source_id, harness);
    if parsed.diagnostics.has_errors() {
        return Err(format!("parse errors for {}: {:?}", workload.name, parsed.diagnostics.as_slice()));
    }
    let sema = analyze_script(&parsed, &atoms);
    if sema.diagnostics.has_errors() {
        return Err(format!("sema errors for {}: {:?}", workload.name, sema.diagnostics.as_slice()));
    }
    let unit = compile_script(&parsed, &sema, &mut atoms)
        .map_err(|e| format!("lowering failed for {}: {e:?}", workload.name))?;

    let mut dispatch = OpcodeDispatchCounts::default();
    let mut histogram = None::<SampleHistogram>;
    let interval = Duration::from_micros(options.interval_us);
    for sample in 0..options.samples {
        let (sample_dispatch, sample_hist) = profile_once(workload, &unit, interval)
            .map_err(|e| format!("sample {} failed for {}: {e}", sample + 1, workload.name))?;
        // Sum dispatch counts across samples.
        let merged_pairs: Vec<(Opcode, u64)> = sample_dispatch
            .iter()
            .filter(|entry| entry.count() != 0)
            .map(|entry| (entry.opcode(), entry.count()))
            .chain(dispatch.iter().filter(|e| e.count() != 0).map(|e| (e.opcode(), e.count())))
            .collect();
        dispatch = OpcodeDispatchCounts::from_counts(merged_pairs);
        match histogram.as_mut() {
            Some(acc) => acc.merge(&sample_hist),
            None => histogram = Some(sample_hist),
        }
    }
    Ok(WorkloadProfile {
        // V8Workload is Copy; `workload` is &V8Workload, so deref once.
        workload: *workload,
        dispatch,
        // parse_options enforces samples >= 1, so the loop ran at least once.
        histogram: histogram.expect("samples >= 1 guarantees one histogram"),
    })
}
```

> **Implementer note:** `profile_workload`'s `workload` parameter is `&V8Workload` (a single reference), and the call site in `run` passes the loop variable `workload` which is `&&V8Workload` — deref coercion handles that, matching the existing `run_workload_opcode_counts(workload, ...)` call in `v8suite.rs`. Inside, `*workload` copies the (Copy) `V8Workload` into the owned `WorkloadProfile`.

Also add a **minimal `samply` submodule stub** now so `run`'s `samply::capture(...)` call compiles in this task. Task 6 replaces this stub with the real implementation. Append to `profile.rs`:

```rust
// Replaced with the real implementation in Task 6.
pub(crate) mod samply {
    use super::{Options, V8Workload};
    pub(crate) fn capture(
        _workload: &V8Workload,
        _harness: &str,
        _options: &Options,
    ) -> Result<(), String> {
        Err("samply capture not yet implemented".to_string())
    }
}
```

- [ ] **Step 5: Implement `profile_once` (single sampled run)**

Append to `tools/lyng-bench/src/profile.rs`:

```rust
fn profile_once(
    workload: &V8Workload,
    unit: &lyng_bytecode::CompiledScriptUnit,
    interval: Duration,
) -> Result<(OpcodeDispatchCounts, SampleHistogram), String> {
    let mut runtime = Runtime::new(NoopHostHooks);
    let agent = runtime.root_agent_mut();
    let realm = agent
        .default_realm()
        .ok_or_else(|| "default realm should exist".to_string())?;
    let realm_id = realm.id();
    let realm_record = realm;
    let mut vm = Vm::new();
    let _ = vm
        .bootstrap_realm(agent, realm_id, BootstrapMode::SpecOnly)
        .map_err(|e| format!("spec bootstrap failed: {e:?}"))?;
    let installed = vm
        .install_script(agent, realm_id, unit)
        .map_err(|e| format!("script install failed for {}: {e:?}", workload.name))?;
    vm.instantiate_global_script(agent, &realm_record, unit.instantiation_plan())
        .map_err(|e| format!("global instantiation failed for {}: {e:?}", workload.name))?;

    // Reset counters (also resets current_opcode to the idle sentinel) BEFORE
    // starting the sampler so no stale opcode is attributed.
    vm.opcode_counters_mut().reset();

    // Capture the cell reference, start the sampler, then run. The profiler is
    // stopped (thread joined) before `vm` is dropped, upholding the safety
    // contract in sampling_profiler.rs.
    let histogram;
    let value;
    {
        // Borrow the cell through the counters; the boxed DispatchCounters
        // address is stable for the VM's lifetime.
        let cell_ptr: *const std::sync::atomic::AtomicU64 =
            vm.opcode_counters().dispatch_banks().current_opcode_cell();
        // SAFETY: the cell lives as long as `vm`; the profiler is stopped
        // before `vm` is dropped at the end of this function.
        let cell: &std::sync::atomic::AtomicU64 = unsafe { &*cell_ptr };
        let profiler = SamplingProfiler::start(cell, interval);

        value = vm
            .evaluate_installed(agent, installed, realm_record.global_env(), realm_record.global_env())
            .run()
            .map_err(|e| format!("execution failed for {}: {e:?}", workload.name))?;

        histogram = profiler.stop();
    }
    black_box(value.bits());

    let dispatch = vm.opcode_counters().dispatch_counts();
    Ok((dispatch, histogram))
}
```

> **Implementer note on the borrow:** taking `&AtomicU64` directly from `dispatch_banks().current_opcode_cell()` and passing it to `SamplingProfiler::start` ties the borrow to `vm`, but the sampler thread holds a raw pointer past that borrow. The explicit `*const` round-trip above is intentional: it documents that we are deliberately extending the cell's reachability past the borrow, justified because `profiler.stop()` joins the thread before `vm` drops. If the borrow checker complains about `vm` being borrowed (the `cell` borrow vs the `&mut`/`&` needed by `evaluate_installed`), capture the raw `*const` first, drop the `&` borrow, then re-`&*` it for `start` — `evaluate_installed` borrows `vm` mutably while the sampler only needs the stable heap address, which the raw pointer already holds.

- [ ] **Step 6: Implement report rendering**

Append to `tools/lyng-bench/src/profile.rs`:

```rust
fn pct(part: u64, whole: u64) -> f64 {
    if whole == 0 { 0.0 } else { (part as f64 / whole as f64) * 100.0 }
}

fn samples_per_mdispatch(samples: u64, dispatches: u64) -> f64 {
    if dispatches == 0 { 0.0 } else { samples as f64 / (dispatches as f64 / 1_000_000.0) }
}

/// Rows sorted by descending total samples, opcodes with >=1 sample only.
fn sorted_rows(profile: &WorkloadProfile) -> Vec<(Opcode, u64, u64, u64)> {
    // (opcode, samples, slow_samples, dispatches)
    let mut rows: Vec<(Opcode, u64, u64, u64)> = profile
        .histogram
        .iter()
        .map(|(op, fast, slow)| (op, fast + slow, slow, profile.dispatch.count(op)))
        .collect();
    rows.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.name().cmp(b.0.name())));
    rows
}

fn render_markdown(options: &Options, profiles: &[WorkloadProfile]) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let _ = writeln!(out, "# Lyng JS Time-Attribution Profile\n");
    let _ = writeln!(
        out,
        "Generated by `cargo run --release -p lyng-bench -- profile`. A background \
         sampler reads the live opcode every `{}us` and bins samples by \
         (opcode x fast/slow path). This is a STATISTICAL view: small-share rows \
         are noise-dominated; judge confidence against total sample count.\n",
        options.interval_us
    );
    let _ = writeln!(out, "- Samples summed per workload: `{}`", options.samples);
    let _ = writeln!(out, "- Sampler interval: `{}us`\n", options.interval_us);

    for profile in profiles {
        let total = profile.histogram.total();
        let total_dispatch = profile.dispatch.total();
        let _ = writeln!(out, "## {}\n", profile.workload.name);
        let _ = writeln!(
            out,
            "Total samples: `{total}` | Total dispatches: `{total_dispatch}` | \
             Non-opcode samples: `{}` ({:.2}%)\n",
            profile.histogram.non_opcode(),
            pct(profile.histogram.non_opcode(), total)
        );
        let _ = writeln!(
            out,
            "| Opcode | Time share | Slow share (of its time) | Dispatches | Samples / Mdispatch |"
        );
        let _ = writeln!(out, "| --- | ---: | ---: | ---: | ---: |");
        for (op, samples, slow, dispatches) in sorted_rows(profile) {
            let _ = writeln!(
                out,
                "| `{}` | {:.2}% | {:.2}% | {} | {:.2} |",
                op.name(),
                pct(samples, total),
                pct(slow, samples),
                dispatches,
                samples_per_mdispatch(samples, dispatches),
            );
        }
        let _ = writeln!(out);
    }
    out
}

fn render_json(options: &Options, profiles: &[WorkloadProfile]) -> Value {
    let workloads: Vec<Value> = profiles
        .iter()
        .map(|profile| {
            let total = profile.histogram.total();
            let rows: Vec<Value> = sorted_rows(profile)
                .into_iter()
                .map(|(op, samples, slow, dispatches)| {
                    json!({
                        "opcode": op.name(),
                        "samples": samples,
                        "slow_samples": slow,
                        "time_share_pct": pct(samples, total),
                        "slow_share_pct": pct(slow, samples),
                        "dispatches": dispatches,
                        "samples_per_mdispatch": samples_per_mdispatch(samples, dispatches),
                    })
                })
                .collect();
            json!({
                "name": profile.workload.name,
                "total_samples": total,
                "total_dispatches": profile.dispatch.total(),
                "non_opcode_samples": profile.histogram.non_opcode(),
                "rows": rows,
            })
        })
        .collect();
    json!({
        "schema": "lyng-bench/profile/v1",
        "interval_us": options.interval_us,
        "samples": options.samples,
        "workloads": workloads,
    })
}

fn print_summary(profiles: &[WorkloadProfile], options: &Options) {
    println!(
        "profile: {} workload(s), {} sample-run(s) each @ {}us interval -> {}",
        profiles.len(),
        options.samples,
        options.interval_us,
        options.report_path
    );
    for profile in profiles {
        if let Some((op, samples, _slow, _d)) = sorted_rows(profile).into_iter().next() {
            println!(
                "  {}: top opcode `{}` at {:.1}% of time",
                profile.workload.name,
                op.name(),
                pct(samples, profile.histogram.total())
            );
        }
    }
}
```

- [ ] **Step 7: Add a statistical end-to-end correctness test**

Append a test to the `#[cfg(test)] mod tests` block in `tools/lyng-bench/src/profile.rs`:

```rust
    #[test]
    fn hot_opcode_dominates_the_profile() {
        // A tight arithmetic loop is dominated by a small set of opcodes; the
        // top profiled opcode should also be among the top dispatched opcodes.
        // Statistical, so assert a weak invariant: the #1 time-share opcode has
        // a nonzero dispatch count and the histogram captured real samples.
        let mut atoms = lyng_common::AtomTable::new();
        let source_id = lyng_common::SourceId::new(1);
        let src = "var s = 0; for (var i = 0; i < 2000000; i++) { s = s + i; } s;";
        let parsed = lyng_parser::parse_script(&mut atoms, source_id, src);
        assert!(!parsed.diagnostics.has_errors());
        let sema = lyng_sema::analyze_script(&parsed, &atoms);
        assert!(!sema.diagnostics.has_errors());
        let unit = lyng_compiler::compile_script(&parsed, &sema, &mut atoms).unwrap();
        let workload = V8Workload { name: "LoopMicro", file: "n/a" };
        let (dispatch, hist) =
            profile_once(&workload, &unit, Duration::from_micros(50)).unwrap();
        assert!(hist.total() > 0, "sampler should capture samples on a 2M-iter loop");
        let profile = WorkloadProfile { workload, dispatch, histogram: hist };
        let top = sorted_rows(&profile).into_iter().next().expect("at least one opcode");
        assert!(top.3 > 0, "top time-share opcode `{}` should have dispatches", top.0.name());
    }
```

> **Implementer note:** `V8Workload` literal construction requires its fields be reachable; they are `pub` within the `pub(crate) struct`. If field construction outside `v8suite` is disallowed, add a `pub(crate) const fn V8Workload::new(name, file)` constructor in `v8suite.rs` and use it here.

- [ ] **Step 8: Run the profile tests (this needs the engine + feature; lyng-bench always has it)**

Run: `cargo test --release -p lyng-bench profile`
Expected: PASS (options tests + the statistical test). Use `--release` so the 2M-iteration loop test is fast.

- [ ] **Step 9: Commit**

```bash
git add tools/lyng-bench/src/profile.rs tools/lyng-bench/src/v8suite.rs
git commit -m "feat(bench): add in-process sampled profile driver + report

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Register the `profile` command in the CLI

**Files:**
- Modify: `tools/lyng-bench/src/cli.rs`
- Modify: `tools/lyng-bench/src/lib.rs`

- [ ] **Step 1: Add a failing parse test for the new command**

In `tools/lyng-bench/src/cli.rs`, add to the `#[cfg(test)] mod tests` block:

```rust
    #[test]
    fn parses_profile_suite_with_passthrough_args() {
        assert_eq!(
            parse_command(&args(&["lyng-bench", "profile", "--filter", "RayTrace"])).unwrap(),
            Command::Profile(vec!["--filter".to_string(), "RayTrace".to_string()])
        );
    }

    #[test]
    fn top_level_help_lists_profile_suite() {
        let help = help_text();
        assert!(help.contains("profile"));
        assert!(help.contains("time-attribution"));
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p lyng-bench cli::tests`
Expected: FAIL — `no variant Profile` / help assertions fail.

- [ ] **Step 3: Add the `Profile` variant, parse arm, and help line**

In `tools/lyng-bench/src/cli.rs`:

Add to the `Command` enum (after `CaptureLlint`):

```rust
    Profile(Vec<String>),
```

Add to the `match` in `parse_command` (after the `capture-llint` arm):

```rust
        Some("profile") => Ok(Command::Profile(args[2..].to_vec())),
```

Update the usage string and add a suite line in `help_text()`. Change the `Usage:` line to include `profile`:

```rust
        "Usage: lyng-bench [runtime|density|test262|compare|v8suite|asm-diff|microbench|capture-llint|profile] [suite-options]",
```

And add this line in the `Suites:` block (after the `capture-llint` line):

```rust
        "  profile       VM-internal time-attribution profile (opcode x fast/slow",
        "                path) with optional samply drill-down",
```

> **Implementer note:** the existing test `top_level_help_lists_external_engine_compare_suite` asserts the exact `Usage:` string. Update that test's expected `Usage:` literal to match the new one that includes `profile`.

- [ ] **Step 4: Wire dispatch in `lib.rs`**

In `tools/lyng-bench/src/lib.rs`, add the module declaration (keep alphabetical-ish with the others):

```rust
pub mod profile;
```

Add the dispatch arm in the `match` (after the `CaptureLlint` arm):

```rust
        cli::Command::Profile(command_args) => profile::run(&command_args),
```

- [ ] **Step 5: Run the CLI tests**

Run: `cargo test -p lyng-bench cli::tests`
Expected: PASS (including the updated `Usage:` assertion).

- [ ] **Step 6: Smoke-test the command end to end**

Run: `cargo run --release -p lyng-bench -- profile --filter RayTrace --samples 1 --report /tmp/lyng-profile-smoke.md --json /tmp/lyng-profile-smoke.json`
Expected: completes, prints a summary line with a top opcode, and writes both files. Open `/tmp/lyng-profile-smoke.md` and confirm the ranked table has `GetNamedProperty` near the top (consistent with the 2026-05-23 RayTrace profile's 22.64% dispatch share).

- [ ] **Step 7: Commit**

```bash
git add tools/lyng-bench/src/cli.rs tools/lyng-bench/src/lib.rs
git commit -m "feat(bench): register 'profile' subcommand

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: samply drill-down integration

**Files:**
- Create: `tools/lyng-bench/src/profile.rs` submodule `samply` (append within the same file)

- [ ] **Step 1: Replace the samply stub with the real capture submodule**

In `tools/lyng-bench/src/profile.rs`, replace the entire `pub(crate) mod samply { ... }` stub added in Task 4 with this full implementation:

```rust
/// Function-level drill-down via the external `samply` profiler. Writes the
/// generated harness to a temp script and records `lyng --shell <script>`.
/// This is the microscope for splitting a single slow handler's internals;
/// the in-process sampler above is the default opcode-level signal.
pub(crate) mod samply {
    use super::{Options, V8Workload};
    use std::io::Write as _;
    use std::process::Command;

    pub(crate) fn capture(
        workload: &V8Workload,
        harness: &str,
        options: &Options,
    ) -> Result<(), String> {
        let script_path = std::env::temp_dir().join(format!("lyng-profile-{}.js", workload.name));
        let mut file = std::fs::File::create(&script_path)
            .map_err(|e| format!("failed to write samply script: {e}"))?;
        file.write_all(harness.as_bytes())
            .map_err(|e| format!("failed to write samply script: {e}"))?;
        let out_path = format!("reports/lyng/samply-{}.json.gz", workload.name);

        let result = Command::new("samply")
            .arg("record")
            .arg("--save-only")
            .arg("-o")
            .arg(&out_path)
            .arg("--")
            .arg(&options.lyng_bin)
            .arg("--shell")
            .arg(&script_path)
            .status();

        match result {
            Ok(status) if status.success() => {
                println!(
                    "  samply: {} profile saved to {out_path} (open with `samply load {out_path}`)",
                    workload.name
                );
                Ok(())
            }
            Ok(status) => Err(format!("samply exited with {status} for {}", workload.name)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(
                "samply not found on PATH. Install with `cargo install samply`, or omit --samply."
                    .to_string(),
            ),
            Err(error) => Err(format!("failed to launch samply: {error}")),
        }
    }
}
```

- [ ] **Step 2: Add a "not installed" path test**

Append to the `#[cfg(test)] mod tests` block in `profile.rs`:

```rust
    #[test]
    fn samply_capture_reports_missing_binary_gracefully() {
        // Point at a lyng-bin/PATH where `samply` does not exist by using a
        // bogus binary name; we only assert the error is actionable, not a panic.
        let options = Options { samply: true, ..Options::default() };
        let workload = V8Workload { name: "Probe", file: "n/a" };
        // If samply IS installed in CI, this will instead fail to run the bogus
        // lyng-bin; either way `capture` must return Err, never panic.
        let result = super::samply::capture(&workload, "1;", &options);
        assert!(result.is_err());
    }
```

> **Implementer note:** this test is intentionally weak (asserts `is_err`, no panic) because samply may or may not be installed in the run environment, and `target/release/lyng` may not exist during `cargo test`. The point is to prove the error paths don't panic. If samply is installed and `target/release/lyng` exists, the test could spuriously *pass the recording* — to keep it deterministic, set `lyng_bin` to a guaranteed-nonexistent path:

Adjust the test to force a deterministic failure regardless of samply presence:

```rust
    #[test]
    fn samply_capture_reports_error_gracefully() {
        let options = Options {
            samply: true,
            lyng_bin: "/nonexistent/lyng-binary-xyz".to_string(),
            ..Options::default()
        };
        let workload = V8Workload { name: "Probe", file: "n/a" };
        let result = super::samply::capture(&workload, "1;", &options);
        // Either samply is absent (NotFound err) or samply runs but the bogus
        // lyng-bin makes it exit nonzero -> Err. Never a panic, never Ok.
        assert!(result.is_err());
    }
```

Use this second version; delete the first.

- [ ] **Step 3: Run the test**

Run: `cargo test -p lyng-bench profile`
Expected: PASS.

- [ ] **Step 4: Manually verify samply wiring (only if samply is installed)**

Run: `cargo install samply` (if needed), build the release binary `cargo build --release -p lyng-cli`, then:
`cargo run --release -p lyng-bench -- profile --filter RayTrace --samples 1 --samply`
Expected: writes `reports/lyng/samply-RayTrace.json.gz` and prints the `samply load` hint. If samply is unavailable, expect the actionable "samply not found" error — that is correct behavior.

- [ ] **Step 5: Commit**

```bash
git add tools/lyng-bench/src/profile.rs
git commit -m "feat(bench): add samply drill-down to profile (--samply)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: Documentation + first real artifact

**Files:**
- Modify: `reports/lyng/llint-parity-state-of-engine.md`
- Create: `reports/lyng/v8-raytrace-profile-2026-05-30.md`

- [ ] **Step 1: Generate the first real RayTrace profile artifact**

Run: `cargo run --release -p lyng-bench -- profile --filter RayTrace --samples 5 --report reports/lyng/profile-raytrace.md --json reports/lyng/profile-raytrace.json`
Expected: completes and writes both files.

- [ ] **Step 2: Write the narrative artifact**

Create `reports/lyng/v8-raytrace-profile-2026-05-30.md` summarizing the run: paste the top ~15 rows from `reports/lyng/profile-raytrace.md`, and write 2-3 paragraphs comparing the new *time*-attribution against the 2026-05-23 *dispatch-count* profile. Specifically call out whether `AssignNamedProperty` (5.57% of dispatches in the old report) shows a disproportionately high time share / high "Samples / Mdispatch", which was the hand-derived hypothesis the new tool now measures directly. Include the exact command used and the sampler interval.

> Content is run-dependent, so it cannot be pre-written here. Required sections: `## Command`, `## Top opcodes by time share` (the pasted table), `## Time vs dispatch-count delta` (the comparison), `## Next target` (what the data points at).

- [ ] **Step 3: Update the state-of-engine evidence list**

In `reports/lyng/llint-parity-state-of-engine.md`, in the "Current Evidence Files" list, add an entry:

```markdown
- [`v8-raytrace-profile-2026-05-30.md`](v8-raytrace-profile-2026-05-30.md):
  time-attribution profile (opcode x fast/slow path) from the in-process
  sampling profiler (`lyng-bench profile`).
```

And in the "Optimization Direction" section, add a line under the existing guidance:

```markdown
- Use `lyng-bench profile --filter <Workload>` for time attribution (which
  opcode/path actually costs wall-time), not just dispatch counts. Treat its
  ranked table as the first read before optimizing a workload.
```

- [ ] **Step 4: Commit**

```bash
git add reports/lyng/llint-parity-state-of-engine.md \
        reports/lyng/v8-raytrace-profile-2026-05-30.md \
        reports/lyng/profile-raytrace.md reports/lyng/profile-raytrace.json
git commit -m "docs(reports): add time-attribution profiler + RayTrace artifact

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Final Verification

- [ ] **Step 1: Full feature-on test pass**

Run: `cargo test --release -p lyng-vm --features diagnostic-counters && cargo test --release -p lyng-bench`
Expected: all PASS.

- [ ] **Step 2: Feature-off no-regression check**

Run: `cargo build --release -p lyng-cli && cargo test -p lyng-vm`
Expected: success. The `lyng` binary carries no profiler/counter code; the hot path is unchanged.

- [ ] **Step 3: Clippy clean (repo bar)**

Run: `cargo clippy --release -p lyng-vm --features diagnostic-counters --all-targets && cargo clippy --release -p lyng-bench --all-targets`
Expected: no warnings (the repo keeps pedantic/nursery clippy clean — see recent commit `6764799b`).

- [ ] **Step 4: fmt**

Run: `cargo fmt --all && git diff --exit-code`
Expected: no diff (already formatted as you went). If `cargo fmt` changed files, commit them.
