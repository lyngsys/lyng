use super::descriptors::{
    accessor_symbol_property, builtin_function_atom_property, builtin_function_symbol_property,
    data_atom_property, data_symbol_property, descriptor_tag, readonly_builtin_attributes,
    writable_builtin_attributes,
};
use super::{
    install_public_builtin_function, ArrayFamilyBuiltins, ArrayFamilyPrototypes,
    FamilyInstallContext,
};
use crate::bootstrap::{install_descriptor_tables, BuiltinBootstrapError};
use crate::public::{BuiltinCache, PublicRealmBuiltins};
use crate::{BuiltinAttributes, BuiltinDescriptorTable, BuiltinInstallTarget, BuiltinIntrinsic};
use lyng_js_common::{AtomId, WellKnownAtom};
use lyng_js_env::Agent;
use lyng_js_types::{
    array_at_builtin, array_builtin, array_concat_builtin, array_copy_within_builtin,
    array_entries_builtin, array_every_builtin, array_fill_builtin, array_filter_builtin,
    array_find_builtin, array_find_index_builtin, array_find_last_builtin,
    array_find_last_index_builtin, array_flat_builtin, array_flat_map_builtin,
    array_for_each_builtin, array_from_async_builtin, array_from_builtin, array_includes_builtin,
    array_index_of_builtin, array_is_array_builtin, array_iterator_next_builtin,
    array_join_builtin, array_keys_builtin, array_last_index_of_builtin, array_map_builtin,
    array_of_builtin, array_pop_builtin, array_push_builtin, array_reduce_builtin,
    array_reduce_right_builtin, array_reverse_builtin, array_shift_builtin, array_slice_builtin,
    array_some_builtin, array_sort_builtin, array_species_getter_builtin, array_splice_builtin,
    array_to_locale_string_builtin, array_to_reversed_builtin, array_to_sorted_builtin,
    array_to_spliced_builtin, array_to_string_builtin, array_unshift_builtin, array_values_builtin,
    array_with_builtin, BuiltinFunctionId, ObjectRef, RealmRef, Value, WellKnownSymbolId,
};

#[allow(clippy::too_many_lines)]
pub(in crate::public) fn install_array_family(
    agent: &mut Agent,
    cx: FamilyInstallContext,
    prototypes: ArrayFamilyPrototypes,
) -> ArrayFamilyBuiltins {
    ArrayFamilyBuiltins {
        array: install_public_builtin_function(
            agent,
            cx,
            array_builtin(),
            Some(prototypes.array_prototype),
        ),
        array_from: install_public_builtin_function(agent, cx, array_from_builtin(), None),
        array_from_async: install_public_builtin_function(
            agent,
            cx,
            array_from_async_builtin(),
            None,
        ),
        array_of: install_public_builtin_function(agent, cx, array_of_builtin(), None),
        array_unscopables: prototypes.array_unscopables,
        array_is_array: install_public_builtin_function(agent, cx, array_is_array_builtin(), None),
        array_at: install_public_builtin_function(agent, cx, array_at_builtin(), None),
        array_concat: install_public_builtin_function(agent, cx, array_concat_builtin(), None),
        array_copy_within: install_public_builtin_function(
            agent,
            cx,
            array_copy_within_builtin(),
            None,
        ),
        array_fill: install_public_builtin_function(agent, cx, array_fill_builtin(), None),
        array_flat: install_public_builtin_function(agent, cx, array_flat_builtin(), None),
        array_flat_map: install_public_builtin_function(agent, cx, array_flat_map_builtin(), None),
        array_join: install_public_builtin_function(agent, cx, array_join_builtin(), None),
        array_pop: install_public_builtin_function(agent, cx, array_pop_builtin(), None),
        array_push: install_public_builtin_function(agent, cx, array_push_builtin(), None),
        array_shift: install_public_builtin_function(agent, cx, array_shift_builtin(), None),
        array_unshift: install_public_builtin_function(agent, cx, array_unshift_builtin(), None),
        array_every: install_public_builtin_function(agent, cx, array_every_builtin(), None),
        array_filter: install_public_builtin_function(agent, cx, array_filter_builtin(), None),
        array_find: install_public_builtin_function(agent, cx, array_find_builtin(), None),
        array_find_index: install_public_builtin_function(
            agent,
            cx,
            array_find_index_builtin(),
            None,
        ),
        array_find_last: install_public_builtin_function(
            agent,
            cx,
            array_find_last_builtin(),
            None,
        ),
        array_find_last_index: install_public_builtin_function(
            agent,
            cx,
            array_find_last_index_builtin(),
            None,
        ),
        array_for_each: install_public_builtin_function(agent, cx, array_for_each_builtin(), None),
        array_includes: install_public_builtin_function(agent, cx, array_includes_builtin(), None),
        array_index_of: install_public_builtin_function(agent, cx, array_index_of_builtin(), None),
        array_map: install_public_builtin_function(agent, cx, array_map_builtin(), None),
        array_reduce: install_public_builtin_function(agent, cx, array_reduce_builtin(), None),
        array_reduce_right: install_public_builtin_function(
            agent,
            cx,
            array_reduce_right_builtin(),
            None,
        ),
        array_reverse: install_public_builtin_function(agent, cx, array_reverse_builtin(), None),
        array_slice: install_public_builtin_function(agent, cx, array_slice_builtin(), None),
        array_some: install_public_builtin_function(agent, cx, array_some_builtin(), None),
        array_last_index_of: install_public_builtin_function(
            agent,
            cx,
            array_last_index_of_builtin(),
            None,
        ),
        array_sort: install_public_builtin_function(agent, cx, array_sort_builtin(), None),
        array_splice: install_public_builtin_function(agent, cx, array_splice_builtin(), None),
        array_to_reversed: install_public_builtin_function(
            agent,
            cx,
            array_to_reversed_builtin(),
            None,
        ),
        array_to_sorted: install_public_builtin_function(
            agent,
            cx,
            array_to_sorted_builtin(),
            None,
        ),
        array_to_spliced: install_public_builtin_function(
            agent,
            cx,
            array_to_spliced_builtin(),
            None,
        ),
        array_to_string: install_public_builtin_function(
            agent,
            cx,
            array_to_string_builtin(),
            None,
        ),
        array_to_locale_string: install_public_builtin_function(
            agent,
            cx,
            array_to_locale_string_builtin(),
            None,
        ),
        array_values: install_public_builtin_function(agent, cx, array_values_builtin(), None),
        array_keys: install_public_builtin_function(agent, cx, array_keys_builtin(), None),
        array_entries: install_public_builtin_function(agent, cx, array_entries_builtin(), None),
        array_with: install_public_builtin_function(agent, cx, array_with_builtin(), None),
        array_iterator_next: install_public_builtin_function(
            agent,
            cx,
            array_iterator_next_builtin(),
            None,
        ),
    }
}

pub(in crate::public) fn install_array_family_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
) -> Result<(), BuiltinBootstrapError> {
    let atoms = ArrayDescriptorAtoms::new(agent);
    install_array_constructor_descriptors(agent, cache, realm, &atoms)?;
    install_array_prototype_descriptors(agent, cache, realm, builtins, &atoms)?;
    install_array_unscopables_descriptors(agent, cache, realm, builtins, &atoms)?;
    install_array_iterator_prototype_descriptors(agent, cache, realm)
}

fn install_array_constructor_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    atoms: &ArrayDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        builtin_function_atom_property(atoms.from, array_from_builtin()),
        builtin_function_atom_property(atoms.from_async, array_from_async_builtin()),
        builtin_function_atom_property(atoms.of, array_of_builtin()),
        builtin_function_atom_property(atoms.is_array, array_is_array_builtin()),
        accessor_symbol_property(
            WellKnownSymbolId::Species,
            Some(array_species_getter_builtin()),
            None,
            readonly_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::Array),
            &descriptors,
        )],
    )
}

#[allow(clippy::too_many_lines)]
fn install_array_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
    atoms: &ArrayDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        data_atom_property(
            WellKnownAtom::constructor.id(),
            Value::from_object_ref(builtins.array),
            writable_builtin_attributes(),
        ),
        builtin_function_atom_property(atoms.at, array_at_builtin()),
        builtin_function_atom_property(WellKnownAtom::toString.id(), array_to_string_builtin()),
        data_atom_property(
            WellKnownAtom::length.id(),
            Value::from_smi(0),
            BuiltinAttributes::new(true, false, false),
        ),
        builtin_function_atom_property(atoms.concat, array_concat_builtin()),
        builtin_function_atom_property(atoms.copy_within, array_copy_within_builtin()),
        builtin_function_atom_property(atoms.fill, array_fill_builtin()),
        builtin_function_atom_property(atoms.join, array_join_builtin()),
        builtin_function_atom_property(atoms.pop, array_pop_builtin()),
        builtin_function_atom_property(atoms.push, array_push_builtin()),
        builtin_function_atom_property(atoms.shift, array_shift_builtin()),
        builtin_function_atom_property(atoms.unshift, array_unshift_builtin()),
        builtin_function_atom_property(atoms.every, array_every_builtin()),
        builtin_function_atom_property(atoms.filter, array_filter_builtin()),
        builtin_function_atom_property(atoms.flat, array_flat_builtin()),
        builtin_function_atom_property(atoms.flat_map, array_flat_map_builtin()),
        builtin_function_atom_property(atoms.find, array_find_builtin()),
        builtin_function_atom_property(atoms.find_index, array_find_index_builtin()),
        builtin_function_atom_property(atoms.find_last, array_find_last_builtin()),
        builtin_function_atom_property(atoms.find_last_index, array_find_last_index_builtin()),
        builtin_function_atom_property(atoms.for_each, array_for_each_builtin()),
        builtin_function_atom_property(atoms.includes, array_includes_builtin()),
        builtin_function_atom_property(atoms.index_of, array_index_of_builtin()),
        builtin_function_atom_property(atoms.map, array_map_builtin()),
        builtin_function_atom_property(atoms.reduce, array_reduce_builtin()),
        builtin_function_atom_property(atoms.reduce_right, array_reduce_right_builtin()),
        builtin_function_atom_property(atoms.reverse, array_reverse_builtin()),
        builtin_function_atom_property(atoms.slice, array_slice_builtin()),
        builtin_function_atom_property(atoms.some, array_some_builtin()),
        builtin_function_atom_property(atoms.last_index_of, array_last_index_of_builtin()),
        builtin_function_atom_property(atoms.sort, array_sort_builtin()),
        builtin_function_atom_property(atoms.splice, array_splice_builtin()),
        builtin_function_atom_property(atoms.to_reversed, array_to_reversed_builtin()),
        builtin_function_atom_property(atoms.to_sorted, array_to_sorted_builtin()),
        builtin_function_atom_property(atoms.to_spliced, array_to_spliced_builtin()),
        builtin_function_atom_property(atoms.to_locale_string, array_to_locale_string_builtin()),
        builtin_function_atom_property(atoms.values, array_values_builtin()),
        builtin_function_atom_property(atoms.keys, array_keys_builtin()),
        builtin_function_atom_property(atoms.entries, array_entries_builtin()),
        builtin_function_atom_property(atoms.with, array_with_builtin()),
        data_symbol_property(
            WellKnownSymbolId::Unscopables,
            Value::from_object_ref(builtins.array_unscopables),
            readonly_builtin_attributes(),
        ),
        builtin_function_symbol_property(
            WellKnownSymbolId::Iterator,
            array_values_builtin(),
            writable_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::ArrayPrototype),
            &descriptors,
        )],
    )
}

fn install_array_unscopables_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
    builtins: &PublicRealmBuiltins,
    atoms: &ArrayDescriptorAtoms,
) -> Result<(), BuiltinBootstrapError> {
    let descriptors = [
        array_unscopables_property(atoms.at),
        array_unscopables_property(atoms.copy_within),
        array_unscopables_property(atoms.entries),
        array_unscopables_property(atoms.fill),
        array_unscopables_property(atoms.find),
        array_unscopables_property(atoms.find_index),
        array_unscopables_property(atoms.find_last),
        array_unscopables_property(atoms.find_last_index),
        array_unscopables_property(atoms.flat),
        array_unscopables_property(atoms.flat_map),
        array_unscopables_property(atoms.includes),
        array_unscopables_property(atoms.keys),
        array_unscopables_property(atoms.to_reversed),
        array_unscopables_property(atoms.to_sorted),
        array_unscopables_property(atoms.to_spliced),
        array_unscopables_property(atoms.values),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Object(builtins.array_unscopables),
            &descriptors,
        )],
    )
}

fn install_array_iterator_prototype_descriptors(
    agent: &mut Agent,
    cache: &mut BuiltinCache,
    realm: RealmRef,
) -> Result<(), BuiltinBootstrapError> {
    let next = agent.atoms_mut().intern_collectible("next");
    let array_iterator_tag = descriptor_tag(agent, "Array Iterator");
    let descriptors = [
        builtin_function_atom_property(next, array_iterator_next_builtin()),
        data_symbol_property(
            WellKnownSymbolId::ToStringTag,
            array_iterator_tag,
            readonly_builtin_attributes(),
        ),
    ];
    install_descriptor_tables(
        agent,
        cache,
        realm,
        &[BuiltinDescriptorTable::new(
            BuiltinInstallTarget::Intrinsic(BuiltinIntrinsic::ArrayIteratorPrototype),
            &descriptors,
        )],
    )
}

fn array_unscopables_property(atom: AtomId) -> crate::BuiltinPropertyDescriptor {
    data_atom_property(
        atom,
        Value::from_bool(true),
        BuiltinAttributes::new(true, true, true),
    )
}

#[derive(Clone, Copy, Debug)]
struct ArrayDescriptorAtoms {
    at: AtomId,
    concat: AtomId,
    copy_within: AtomId,
    entries: AtomId,
    every: AtomId,
    fill: AtomId,
    filter: AtomId,
    find: AtomId,
    find_index: AtomId,
    find_last: AtomId,
    find_last_index: AtomId,
    flat: AtomId,
    flat_map: AtomId,
    for_each: AtomId,
    from: AtomId,
    from_async: AtomId,
    includes: AtomId,
    index_of: AtomId,
    is_array: AtomId,
    join: AtomId,
    keys: AtomId,
    last_index_of: AtomId,
    map: AtomId,
    of: AtomId,
    pop: AtomId,
    push: AtomId,
    reduce: AtomId,
    reduce_right: AtomId,
    reverse: AtomId,
    shift: AtomId,
    slice: AtomId,
    some: AtomId,
    sort: AtomId,
    splice: AtomId,
    to_locale_string: AtomId,
    to_reversed: AtomId,
    to_sorted: AtomId,
    to_spliced: AtomId,
    unshift: AtomId,
    values: AtomId,
    with: AtomId,
}

impl ArrayDescriptorAtoms {
    fn new(agent: &mut Agent) -> Self {
        let atoms = agent.atoms_mut();
        Self {
            at: atoms.intern_collectible("at"),
            concat: atoms.intern_collectible("concat"),
            copy_within: atoms.intern_collectible("copyWithin"),
            entries: atoms.intern_collectible("entries"),
            every: atoms.intern_collectible("every"),
            fill: atoms.intern_collectible("fill"),
            filter: atoms.intern_collectible("filter"),
            find: atoms.intern_collectible("find"),
            find_index: atoms.intern_collectible("findIndex"),
            find_last: atoms.intern_collectible("findLast"),
            find_last_index: atoms.intern_collectible("findLastIndex"),
            flat: atoms.intern_collectible("flat"),
            flat_map: atoms.intern_collectible("flatMap"),
            for_each: atoms.intern_collectible("forEach"),
            from: atoms.intern_collectible("from"),
            from_async: atoms.intern_collectible("fromAsync"),
            includes: atoms.intern_collectible("includes"),
            index_of: atoms.intern_collectible("indexOf"),
            is_array: atoms.intern_collectible("isArray"),
            join: atoms.intern_collectible("join"),
            keys: atoms.intern_collectible("keys"),
            last_index_of: atoms.intern_collectible("lastIndexOf"),
            map: atoms.intern_collectible("map"),
            of: atoms.intern_collectible("of"),
            pop: atoms.intern_collectible("pop"),
            push: atoms.intern_collectible("push"),
            reduce: atoms.intern_collectible("reduce"),
            reduce_right: atoms.intern_collectible("reduceRight"),
            reverse: atoms.intern_collectible("reverse"),
            shift: atoms.intern_collectible("shift"),
            slice: atoms.intern_collectible("slice"),
            some: atoms.intern_collectible("some"),
            sort: atoms.intern_collectible("sort"),
            splice: atoms.intern_collectible("splice"),
            to_locale_string: atoms.intern_collectible("toLocaleString"),
            to_reversed: atoms.intern_collectible("toReversed"),
            to_sorted: atoms.intern_collectible("toSorted"),
            to_spliced: atoms.intern_collectible("toSpliced"),
            unshift: atoms.intern_collectible("unshift"),
            values: atoms.intern_collectible("values"),
            with: atoms.intern_collectible("with"),
        }
    }
}

pub(in crate::public) fn array_builtin_object(
    builtins: &PublicRealmBuiltins,
    entry: BuiltinFunctionId,
) -> Option<ObjectRef> {
    [
        (array_builtin(), builtins.array),
        (array_from_builtin(), builtins.array_from),
        (array_from_async_builtin(), builtins.array_from_async),
        (array_of_builtin(), builtins.array_of),
        (array_is_array_builtin(), builtins.array_is_array),
        (array_at_builtin(), builtins.array_at),
        (array_concat_builtin(), builtins.array_concat),
        (array_copy_within_builtin(), builtins.array_copy_within),
        (array_fill_builtin(), builtins.array_fill),
        (array_flat_builtin(), builtins.array_flat),
        (array_flat_map_builtin(), builtins.array_flat_map),
        (array_join_builtin(), builtins.array_join),
        (array_pop_builtin(), builtins.array_pop),
        (array_push_builtin(), builtins.array_push),
        (array_shift_builtin(), builtins.array_shift),
        (array_unshift_builtin(), builtins.array_unshift),
        (array_every_builtin(), builtins.array_every),
        (array_filter_builtin(), builtins.array_filter),
        (array_find_builtin(), builtins.array_find),
        (array_find_index_builtin(), builtins.array_find_index),
        (array_find_last_builtin(), builtins.array_find_last),
        (
            array_find_last_index_builtin(),
            builtins.array_find_last_index,
        ),
        (array_for_each_builtin(), builtins.array_for_each),
        (array_includes_builtin(), builtins.array_includes),
        (array_index_of_builtin(), builtins.array_index_of),
        (array_map_builtin(), builtins.array_map),
        (array_reduce_builtin(), builtins.array_reduce),
        (array_reduce_right_builtin(), builtins.array_reduce_right),
        (array_reverse_builtin(), builtins.array_reverse),
        (array_slice_builtin(), builtins.array_slice),
        (array_some_builtin(), builtins.array_some),
        (array_last_index_of_builtin(), builtins.array_last_index_of),
        (array_sort_builtin(), builtins.array_sort),
        (array_splice_builtin(), builtins.array_splice),
        (array_to_reversed_builtin(), builtins.array_to_reversed),
        (array_to_sorted_builtin(), builtins.array_to_sorted),
        (array_to_spliced_builtin(), builtins.array_to_spliced),
        (array_to_string_builtin(), builtins.array_to_string),
        (
            array_to_locale_string_builtin(),
            builtins.array_to_locale_string,
        ),
        (array_values_builtin(), builtins.array_values),
        (array_keys_builtin(), builtins.array_keys),
        (array_entries_builtin(), builtins.array_entries),
        (array_with_builtin(), builtins.array_with),
        (array_iterator_next_builtin(), builtins.array_iterator_next),
    ]
    .into_iter()
    .find_map(|(id, object)| (entry == id).then_some(object))
}
