#![no_main]

use libfuzzer_sys::fuzz_target;

use lyng_js_common::{AtomTable, SourceId};

fuzz_target!(|data: &[u8]| {
    // Only fuzz valid UTF-8.
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };

    // Limit input size to avoid OOM from deeply nested or very large inputs.
    if source.len() > 4096 {
        return;
    }

    let mut atoms = AtomTable::new();
    // Parse as script. Errors are fine — panics are not.
    let _parsed = lyng_js_parser::parse_script(&mut atoms, SourceId::new(0), source);
});
