# Microbench Baseline

Samples per opcode: 7
Inner iters per sample: 5000000

| Opcode | Samples | Median ns/dispatch | Min | Max | CI95 half-width | Snippet ratio |
|---|---:|---:|---:|---:|---:|---|
| `Move` | 7 | 33.60 | 33.20 | 34.26 | ±0.07 | 4 ops/iter |
| `Add` | 7 | 107.57 | 106.93 | 108.01 | ±0.46 | 1 ops/iter |
| `GetKeyedProperty` | — | no snippet | — | — | — | — |
| `Mul` | — | no snippet | — | — | — | — |
| `Increment` | — | no snippet | — | — | — | — |
| `GetNamedProperty` | 7 | 51.65 | 51.17 | 52.31 | ±0.40 | 3 ops/iter |
| `LoadSmi8` | — | no snippet | — | — | — | — |
| `LoadLocal1` | — | no snippet | — | — | — | — |
| `LoadLocal3` | — | no snippet | — | — | — | — |
| `ShiftRight` | — | no snippet | — | — | — | — |
| `LoadLocal0` | — | no snippet | — | — | — | — |
| `LoadThis` | — | no snippet | — | — | — | — |
| `AssignKeyedProperty` | — | no snippet | — | — | — | — |
| `JumpIfFalse` | — | no snippet | — | — | — | — |
| `Jump` | 7 | 87.18 | 86.36 | 88.19 | ±0.31 | 1 ops/iter |
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
