# Lyng JS V8 RayTrace Profile

Date: 2026-05-23

## Target

Profiled the V8 v7 `RayTrace` workload because it is one of the largest
JSC LLInt-parity gaps in the checked-in external-engine comparison and its
opcode mix is property-heavy:

- Checked-in V8-suite median: score `413`
- Fresh one-sample score run: score `399`
- Direct profiled run score: `414`

Commands:

```sh
target/release/lyng-bench v8suite \
  --filter RayTrace \
  --samples 1 \
  --report /tmp/lyng-v8-raytrace-profile-score.md \
  --json /tmp/lyng-v8-raytrace-profile-score.json

target/release/lyng-bench v8suite \
  --filter RayTrace \
  --samples 1 \
  --count-opcodes \
  --counts-json /tmp/lyng-v8-raytrace-opcodes.json

sample <lyng-pid> 5 -file /tmp/lyng-v8-raytrace.sample.txt
```

`cargo-flamegraph` and the standalone `flamegraph` tool both launched the
workload through macOS Time Profiler, but both failed while collapsing the
generated trace:

```text
Error: unable to collapse generated profile data
Caused by:
    Read xml event failed: IllFormed(MismatchedEndTag { expected: "frame", found: "backtrace" })
```

The report below uses the successful `sample` call tree plus the V8-suite
opcode-count run.

## Opcode Shape

RayTrace executed `177,810,678` bytecode dispatches in the counted run.

| Opcode | Dispatches | Share |
| --- | ---: | ---: |
| `GetNamedProperty` | `40,252,779` | `22.64%` |
| `LoadThis` | `20,811,405` | `11.70%` |
| `JumpIfFalse8` | `13,516,003` | `7.60%` |
| `LoadLocal0` | `10,644,046` | `5.99%` |
| `AssignNamedProperty` | `9,895,668` | `5.57%` |
| `Move` | `7,279,631` | `4.09%` |
| `LoadLocal1` | `7,274,806` | `4.09%` |
| `LoadZero` | `6,471,051` | `3.64%` |
| `ReturnUndefined` | `5,648,769` | `3.18%` |
| `Mul` | `5,427,955` | `3.05%` |
| `Jump8` | `4,523,002` | `2.54%` |
| `LoadGlobal` | `4,461,299` | `2.51%` |
| `Call2` | `4,184,344` | `2.35%` |
| `LoadLocal2` | `3,852,410` | `2.17%` |
| `LoadTrue` | `3,254,286` | `1.83%` |

The workload is property-heavy, not arithmetic-heavy. The top two opcodes
alone are `34.34%` of all dispatches, and `AssignNamedProperty` adds another
`5.57%`.

## Sample Hotspots

`sample` captured `888` main-thread samples. The strongest stacks point at
three cost centers.

### Named Property Writes

The biggest recursive stack cluster is named-property assignment:

- `op_assign_named_property_dsl`: `97` recursive stack appearances
- `op_assign_named_property_slow_rs`: `92`
- `Vm::execute_set_named_property_opcode`: `55` at one site, plus `22` at
  another
- `ordinary_set`: `55`
- `ObjectRuntime::set`: `51` and `37`
- `ObjectRuntime::define_own_property`: `48`
- `ordinary_define_own_property`: `36`
- `ordinary_define_own_named_property`: `22`

Top-of-stack samples also show `BTreeMap::insert` at `88`, plus descriptor,
shape, and write-barrier work below named-property writes. This is the clearest
slow area: RayTrace repeatedly writes ordinary object properties, but the path
still falls through the generic define/set machinery and map-backed metadata
updates.

Likely follow-up: specialize ordinary named-property store when the feedback
site has a stable receiver shape and an own writable data slot. Avoid
`ordinary_set` / `define_own_property` / descriptor recomputation in the
monomorphic case.

### Named Property Reads

Reads dominate dispatch counts and also show up in samples:

- `GetNamedProperty`: `22.64%` of dispatches
- `op_get_named_property_dsl`: `13` recursive stack appearances, `10` top
  samples
- `op_get_named_property_slow_rs`: `12` recursive stack appearances
- `op_get_named_property_semantic`: `32` top samples
- `observe_named_property_slow_path`: `13` and `12` recursive stack
  appearances, `14` top samples
- `ShapeMetadata::property`: `22` top samples
- `ordinary_own_named_property`: `19` top samples
- `BuildHasher::hash_one`: visible in both read and write stacks

This suggests the named-load inline cache is either missing the hot shapes for
RayTrace or still paying too much feedback/shape lookup cost even after a hit.
The high `GetNamedProperty` dispatch share makes this a first-order target.

Likely follow-up: measure hit/miss state for RayTrace named loads, then either
improve IC attachment for Prototype-style objects or add a tighter asm/Rust fast
path for own/prototype data-slot reads.

### Construction And Call Setup

RayTrace constructs many small objects (`Vector`, `Color`, `Ray`,
`IntersectionInfo`, material and shape records). The profile shows:

- `op_construct_dsl`: `35` recursive stack appearances
- `op_construct_slow_rs`: `35`
- `Vm::construct_value`: `26` at one call-site and `8` at another
- `enter_bytecode_call`: `23`
- `prepare_bytecode_call`: `42`
- `install_prepared_bytecode_call`: `13` top samples, with many recursive
  children
- `alloc_function_environment`: `42`
- `ObjectRuntime::alloc_object`: visible in construction and `create_construct_this`

The call/construct path allocates function environments and object records in a
loop-heavy workload. Some of that is semantically necessary, but the sample
shows repeated card-marking, slot allocation, descriptor initialization, and
environment metadata drops.

Likely follow-up: add a constructor-call fast path for ordinary JS constructors
with no exotic `new.target`, no argument adaptation, and no captured function
environment writes beyond the known frame layout.

## Dispatch-Layer Notes

Several hot opcodes in this workload still enter slow Rust stubs:

- `AssignNamedProperty`
- `GetNamedProperty`
- `Construct`
- `Call2`
- `ReturnUndefined`
- `LoadThis`

The asm-DSL substrate itself is not the only issue here. The hottest costs are
semantic slow paths reached from property and call opcodes. Inline-porting more
simple local/load arithmetic opcodes will help total dispatch overhead, but
RayTrace needs property IC and call/construct specialization more than another
Smi arithmetic port.

## Recommendations

1. Treat `AssignNamedProperty` as the primary RayTrace bottleneck. It is only
   `5.57%` of dispatches, but the sample shows it fans out into the most
   expensive object-model machinery.
2. Treat `GetNamedProperty` as the broadest bottleneck. It is `22.64%` of
   dispatches and still visible in `op_get_named_property_semantic`,
   `ShapeMetadata::property`, and feedback slow-path observation.
3. Audit RayTrace named-property IC hit rates before changing code. The profile
   suggests either poor hit rate or expensive hit handling; the fix differs.
4. Add a narrow constructor fast path only after property work. Construction is
   a visible cost center, but many construction samples are downstream object
   allocation and property definition costs that property-store specialization
   may reduce first.

## Artifacts

- Score report: `/tmp/lyng-v8-raytrace-profile-score.md`
- Score JSON: `/tmp/lyng-v8-raytrace-profile-score.json`
- Opcode counts: `/tmp/lyng-v8-raytrace-opcodes.json`
- Sample call tree: `/tmp/lyng-v8-raytrace.sample.txt`
