# Microbench Baseline

Samples per opcode: 7
Inner iters per sample: 5000000

| Opcode | Samples | Median ns/dispatch | Min | Max | CI95 half-width | Snippet ratio |
|---|---:|---:|---:|---:|---:|---|
| `Move` | 7 | 33.46 | 33.28 | 34.92 | ±0.12 | 4 ops/iter |
| `Add` | 7 | 105.87 | 104.99 | 106.66 | ±0.56 | 1 ops/iter |
| `GetKeyedProperty` | — | no snippet | — | — | — | — |
| `Mul` | — | no snippet | — | — | — | — |
| `Increment` | — | no snippet | — | — | — | — |
| `GetNamedProperty` | 7 | 51.75 | 51.54 | 52.01 | ±0.16 | 3 ops/iter |
| `LoadSmi8` | — | no snippet | — | — | — | — |
| `LoadLocal1` | — | no snippet | — | — | — | — |
| `LoadLocal3` | — | no snippet | — | — | — | — |
| `ShiftRight` | — | no snippet | — | — | — | — |
| `LoadLocal0` | — | no snippet | — | — | — | — |
| `LoadThis` | — | no snippet | — | — | — | — |
| `AssignKeyedProperty` | — | no snippet | — | — | — | — |
| `JumpIfFalse` | — | no snippet | — | — | — | — |
| `Jump` | 7 | 86.94 | 86.16 | 87.32 | ±0.45 | 1 ops/iter |
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
