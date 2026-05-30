# Lyng JS V8 RayTrace Time-Attribution Profile

Date: 2026-05-30

This report uses the new in-process sampling profiler (`lyng-bench profile`,
schema `lyng-bench/profile/v1`) to attribute wall-time to opcodes and fast/slow
paths. It complements the earlier dispatch-COUNT profile
([`v8-raytrace-profile-2026-05-23.md`](v8-raytrace-profile-2026-05-23.md)) by
measuring where time actually goes rather than which opcodes execute most.

## Command

```sh
cargo run --release -p lyng-bench -- profile \
  --filter RayTrace \
  --samples 5 \
  --report reports/lyng/profile-raytrace.md \
  --json reports/lyng/profile-raytrace.json
```

A background sampler reads the live opcode every `200us` (default interval) and
bins each sample by (opcode x fast/slow path). The run summed `5` sample-runs.
From the report header for RayTrace:

- Total samples: `143,136`
- Total dispatches: `893,286,932`
- Non-opcode samples: `1` (0.00%)

This is a statistical view: small-share rows are noise-dominated and should be
judged against the total sample count.

## Top opcodes by time share

| Opcode | Time share | Slow share (of its time) | Dispatches | Samples / Mdispatch |
| --- | ---: | ---: | ---: | ---: |
| `Construct` | 41.93% | 99.95% | 14052027 | 4270.56 |
| `Call2` | 13.12% | 99.86% | 21021347 | 893.09 |
| `ReturnUndefined` | 12.32% | 99.59% | 28378339 | 621.57 |
| `GetNamedProperty` | 9.28% | 68.00% | 202222292 | 65.67 |
| `AssignNamedProperty` | 8.52% | 87.14% | 49713950 | 245.34 |
| `LoadGlobal` | 2.42% | 3.21% | 22412715 | 154.24 |
| `Call1` | 1.67% | 99.29% | 6091645 | 391.36 |
| `JumpIfFalse8` | 1.59% | 87.86% | 67901823 | 33.47 |
| `Return` | 1.36% | 98.61% | 15601490 | 124.92 |
| `Mul` | 1.33% | 94.65% | 27269012 | 69.97 |
| `Add` | 0.90% | 93.24% | 15733913 | 81.80 |
| `LoadEnvSlot` | 0.82% | 96.24% | 14051965 | 83.26 |
| `Call0` | 0.68% | 99.38% | 2774082 | 351.47 |
| `Sub` | 0.57% | 92.33% | 11246511 | 73.00 |
| `Wide` | 0.48% | 97.83% | 7544249 | 91.73 |

## Time vs dispatch-count delta

The dispatch-count profile from 2026-05-23 ranked opcodes by how often they
execute: `GetNamedProperty` led at 22.64% of dispatches, then `LoadThis`
(11.70%), `JumpIfFalse8` (7.60%), with `AssignNamedProperty` far down at 5.57%.
That report could only count dispatches, so it *hypothesized* that
`AssignNamedProperty` was disproportionately expensive — "only 5.57% of
dispatches but fans out into the most expensive object-model machinery" — and
named it the primary bottleneck. The new profiler measures time directly, and
the picture is materially different.

By time, the leader is not the dispatch-count leader. `Construct` tops the table
at **41.93%** of measured time despite only 14.05M dispatches, and it runs on
the slow path **99.95%** of the time. Its cost-per-dispatch is by far the
highest of any hot opcode — **4270.56 samples / Mdispatch**, roughly 65x that of
`GetNamedProperty`. The other two call/return opcodes follow: `Call2` at 13.12%
(99.86% slow) and `ReturnUndefined` at 12.32% (99.59% slow). The three of them
together account for ~67% of RayTrace's wall-time and are all almost entirely on
slow paths. RayTrace's cost is dominated by call/construct/return setup, not by
property machinery.

The property opcodes land lower and behave very differently from each other.
`GetNamedProperty` — #1 by dispatch count — falls to **#4 by time** at 9.28%,
with the lowest cost-per-dispatch of the hot opcodes (65.67 / Mdispatch) and a
moderate 68.00% slow share. So the most-executed opcode is comparatively cheap
per call; its IC hit path is mostly working. `AssignNamedProperty` is #5 by
time at 8.52% with an 87.14% slow share and 245.34 / Mdispatch — genuinely
slow-path-heavy and roughly 3.7x more expensive per dispatch than
`GetNamedProperty`, but at one-fifth the total time of `Construct`.

This **revises** the 2026-05-23 hypothesis. The earlier report was directionally
right that `AssignNamedProperty` is slow-path-bound per dispatch, but wrong about
its leverage: it is not the largest time sink. The new measurement shows the
construct/call/return cluster — flagged in 2026-05-23 only as a "visible cost
center" to address *after* property work — is in fact the dominant cost.
`Construct` alone is ~5x the time of `AssignNamedProperty`. The 5-sample run
reproduces the representative single-sample shape (`Construct` ~42% with ~100%
slow share and a very high Samples/Mdispatch; `GetNamedProperty` at ~#4 by time
with low cost-per-dispatch), so this ordering is stable, not a sampling artifact.

## Next target

`Construct` is the highest-leverage target: it has both the largest time share
(41.93%) and a near-total slow-path share (99.95%), the textbook profile of an
opcode missing a fast path entirely. Every RayTrace construction
(`Vector`, `Color`, `Ray`, `IntersectionInfo`, material/shape records) pays full
slow-path construct cost — environment allocation, object-record allocation, and
construct-this setup — with no monomorphic shortcut. Adding a constructor fast
path for ordinary JS constructors (no exotic `new.target`, no argument
adaptation, frame-shaped environment) directly attacks the single largest cost
center.

The adjacent call/return opcodes (`Call2` 13.12%, `ReturnUndefined` 12.32%,
`Call1` 1.67%) share the same ~99% slow-path character and the same call-setup
machinery, so a call/return fast-path effort is likely to move all of them
together and is the natural second target. Property-store specialization for
`AssignNamedProperty` (8.52%, 87.14% slow) remains worthwhile and should follow,
but the data is clear that call/construct setup — not the object model — is where
RayTrace spends its time, and should be addressed first.
