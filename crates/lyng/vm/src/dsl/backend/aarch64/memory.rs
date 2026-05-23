//! Raw load/store fragments — small utilities the other backend
//! modules call into. Kept separate so the named operations
//! (`load_reg!`, `load_record_inline_slot!`, etc.) carry domain
//! semantics, while these stay generic for one-off byte fetches.
//!
//! `$base` is an ident naming the *index suffix* of the X-register
//! (e.g. `19` for `x19`); `$dst` / `$src` are likewise plain idents
//! (typically `t0..t6` ↔ `9..15`).

/// Load 1 byte zero-extended into `wDst`.
///
/// Usage: `load_byte!(19, 4 => 10)` → `ldrb w10, [x19, #4]`.
#[macro_export]
macro_rules! load_byte {
    ($base:literal, $offset:literal => $dst:ident) => {
        concat!(
            "ldrb   w",
            stringify!($dst),
            ", [x",
            stringify!($base),
            ", #",
            stringify!($offset),
            "]\n",
        )
    };
}

/// Load 2 bytes zero-extended into `wDst`.
#[macro_export]
macro_rules! load_half {
    ($base:literal, $offset:literal => $dst:ident) => {
        concat!(
            "ldrh   w",
            stringify!($dst),
            ", [x",
            stringify!($base),
            ", #",
            stringify!($offset),
            "]\n",
        )
    };
}

/// Load 4 bytes into `wDst`.
#[macro_export]
macro_rules! load_word {
    ($base:literal, $offset:literal => $dst:ident) => {
        concat!(
            "ldr    w",
            stringify!($dst),
            ", [x",
            stringify!($base),
            ", #",
            stringify!($offset),
            "]\n",
        )
    };
}

/// Load 8 bytes into `xDst`.
#[macro_export]
macro_rules! load_quad {
    ($base:literal, $offset:literal => $dst:ident) => {
        concat!(
            "ldr    x",
            stringify!($dst),
            ", [x",
            stringify!($base),
            ", #",
            stringify!($offset),
            "]\n",
        )
    };
}

/// Store 1 byte from `wSrc`.
#[macro_export]
macro_rules! store_byte {
    ($src:ident, $base:literal, $offset:literal) => {
        concat!(
            "strb   w",
            stringify!($src),
            ", [x",
            stringify!($base),
            ", #",
            stringify!($offset),
            "]\n",
        )
    };
}

/// Store 2 bytes from `wSrc`.
#[macro_export]
macro_rules! store_half {
    ($src:ident, $base:literal, $offset:literal) => {
        concat!(
            "strh   w",
            stringify!($src),
            ", [x",
            stringify!($base),
            ", #",
            stringify!($offset),
            "]\n",
        )
    };
}

/// Store 4 bytes from `wSrc`.
#[macro_export]
macro_rules! store_word {
    ($src:ident, $base:literal, $offset:literal) => {
        concat!(
            "str    w",
            stringify!($src),
            ", [x",
            stringify!($base),
            ", #",
            stringify!($offset),
            "]\n",
        )
    };
}

/// Store 8 bytes from `xSrc`.
#[macro_export]
macro_rules! store_quad {
    ($src:ident, $base:literal, $offset:literal) => {
        concat!(
            "str    x",
            stringify!($src),
            ", [x",
            stringify!($base),
            ", #",
            stringify!($offset),
            "]\n",
        )
    };
}
