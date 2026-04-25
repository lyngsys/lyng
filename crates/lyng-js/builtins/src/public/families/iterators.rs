use super::{
    install_public_builtin_function, install_public_builtin_function_with_metadata,
    FamilyInstallContext, IteratorFamilyBuiltins, IteratorFamilyPrototypes,
};
use crate::BuiltinEntryMetadata;
use lyng_js_env::Agent;
use lyng_js_types::{
    js3_iterator_prototype_iterator_builtin, js3_map_iterator_next_builtin,
    js3_set_iterator_next_builtin,
};

pub(in crate::public) fn install_iterator_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: IteratorFamilyPrototypes,
) -> IteratorFamilyBuiltins {
    IteratorFamilyBuiltins {
        async_iterator_prototype: prototypes.async_iterator_prototype,
        iterator_prototype_iterator: install_public_builtin_function(
            agent,
            cx,
            js3_iterator_prototype_iterator_builtin(),
            None,
        ),
        async_iterator_method: install_public_builtin_function_with_metadata(
            agent,
            cx,
            js3_iterator_prototype_iterator_builtin(),
            BuiltinEntryMetadata::new("[Symbol.asyncIterator]", 0, false, false),
            None,
        ),
        map_iterator_next: install_public_builtin_function(
            agent,
            cx,
            js3_map_iterator_next_builtin(),
            None,
        ),
        set_iterator_next: install_public_builtin_function(
            agent,
            cx,
            js3_set_iterator_next_builtin(),
            None,
        ),
    }
}
