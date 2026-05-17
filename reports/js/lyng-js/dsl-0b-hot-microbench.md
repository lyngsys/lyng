# Microbench Baseline

Samples per opcode: 7
Inner iters per sample: 5000000

| Opcode | Samples | Median ns/dispatch | Min | Max | CI95 half-width | Snippet ratio |
|---|---:|---:|---:|---:|---:|---|
| `Move` | 7 | 59.85 | 59.21 | 59.91 | ±0.13 | 4 ops/iter |
| `Add` | 7 | 243.33 | 241.16 | 244.03 | ±0.19 | 1 ops/iter |
| `GetKeyedProperty` | — | no snippet | — | — | — | — |
| `Mul` | — | no snippet | — | — | — | — |
| `Increment` | — | no snippet | — | — | — | — |
| `GetNamedProperty` | 7 | 165.01 | 164.18 | 165.67 | ±0.60 | 3 ops/iter |
| `LoadSmi8` | — | no snippet | — | — | — | — |
| `LoadLocal1` | — | no snippet | — | — | — | — |
| `LoadLocal3` | — | no snippet | — | — | — | — |
| `ShiftRight` | — | no snippet | — | — | — | — |
| `LoadLocal0` | — | no snippet | — | — | — | — |
| `LoadThis` | — | no snippet | — | — | — | — |
| `AssignKeyedProperty` | — | no snippet | — | — | — | — |
| `JumpIfFalse` | — | no snippet | — | — | — | — |
| `Jump` | 7 | 181.98 | 179.12 | 182.66 | ±0.44 | 1 ops/iter |
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
