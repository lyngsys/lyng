# Microbench Baseline

Samples per opcode: 7
Inner iters per sample: 5000000

| Opcode | Samples | Median ns/dispatch | Min | Max | CI95 half-width | Snippet ratio |
|---|---:|---:|---:|---:|---:|---|
| `Move` | 7 | 55.42 | 55.25 | 55.69 | ±0.18 | 4 ops/iter |
| `Add` | 7 | 228.16 | 226.45 | 231.51 | ±0.56 | 1 ops/iter |
| `GetKeyedProperty` | — | no snippet | — | — | — | — |
| `Mul` | — | no snippet | — | — | — | — |
| `Increment` | — | no snippet | — | — | — | — |
| `GetNamedProperty` | 7 | 120.25 | 119.83 | 122.40 | ±0.14 | 3 ops/iter |
| `LoadSmi8` | — | no snippet | — | — | — | — |
| `LoadLocal1` | — | no snippet | — | — | — | — |
| `LoadLocal3` | — | no snippet | — | — | — | — |
| `ShiftRight` | — | no snippet | — | — | — | — |
| `LoadLocal0` | — | no snippet | — | — | — | — |
| `LoadThis` | — | no snippet | — | — | — | — |
| `AssignKeyedProperty` | — | no snippet | — | — | — | — |
| `JumpIfFalse` | — | no snippet | — | — | — | — |
| `Jump` | 7 | 171.10 | 170.84 | 171.65 | ±0.18 | 1 ops/iter |
| `LoadZero` | — | no snippet | — | — | — | — |
| `JumpIfFalse8` | — | no snippet | — | — | — | — |
| `LoadLocal2` | — | no snippet | — | — | — | — |
| `LoadEnvSlot` | — | no snippet | — | — | — | — |
| `GreaterEqual` | — | no snippet | — | — | — | — |
| `LoadConst8` | — | no snippet | — | — | — | — |
| `StoreLocal3` | — | no snippet | — | — | — | — |
| `Decrement` | — | no snippet | — | — | — | — |
| `BitAnd` | — | no snippet | — | — | — | — |
| `ShiftLeft` | — | no snippet | — | — | — | — |
| `Ldar` | — | no snippet | — | — | — | — |
| `LessEqual` | — | no snippet | — | — | — | — |
| `AssignNamedProperty` | — | no snippet | — | — | — | — |
| `Sub` | — | no snippet | — | — | — | — |
| `LoadGlobal` | — | no snippet | — | — | — | — |
