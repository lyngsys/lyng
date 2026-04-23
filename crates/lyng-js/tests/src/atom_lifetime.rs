//! Integration coverage for atom lifetime behavior across the frontend pipeline.

use lyng_js_common::{AtomLifetime, AtomTable, SourceId};
use lyng_js_parser::parse_script;
use lyng_js_sema::analyze_script;

#[test]
fn parser_and_sema_keep_frontend_atoms_permanent() {
    let mut atoms = AtomTable::new();
    let parsed = parse_script(&mut atoms, SourceId::new(0), "let alpha = beta;");
    assert!(!parsed.diagnostics.has_errors());

    let sema = analyze_script(&parsed, &atoms);
    assert!(!sema.diagnostics.has_errors());

    let alpha = sema
        .binding_table
        .as_slice()
        .iter()
        .find(|binding| atoms.resolve(binding.name) == "alpha")
        .map(|binding| binding.name)
        .expect("binding for alpha");
    let beta = sema
        .use_sites
        .as_slice()
        .iter()
        .find(|use_site| atoms.resolve(use_site.name) == "beta")
        .map(|use_site| use_site.name)
        .expect("use site for beta");

    assert_eq!(atoms.lifetime(alpha), Some(AtomLifetime::Permanent));
    assert_eq!(atoms.lifetime(beta), Some(AtomLifetime::Permanent));
}
