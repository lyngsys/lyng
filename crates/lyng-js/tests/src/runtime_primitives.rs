//! Compile-smoke coverage for the runtime primitive crate DAG.

use lyng_js_common::{AtomId, AtomTable, SourceId};
use lyng_js_gc::{AllocationLifetime, PrimitiveHeap, PrimitiveHeapMarker, StringEncoding};
use lyng_js_ops::{allocating, pure, read, PrimitiveContext, PrimitiveOpsMarker};
use lyng_js_types::{
    AbruptCompletion, BigIntRef, Completion, PropertyDescriptor, PropertyKey, StringRef, SymbolRef,
    TypeOwnershipMarker, Value,
};

#[test]
fn phase2_runtime_crates_form_expected_dependency_chain() {
    let property_name = AtomId::from_raw(1);
    let type_marker = TypeOwnershipMarker::new(property_name);
    let heap_marker = PrimitiveHeapMarker::new(type_marker, SourceId::new(7));
    let ops_marker = PrimitiveOpsMarker::new(heap_marker, property_name);

    assert_eq!(ops_marker.property_name(), property_name);
    assert_eq!(ops_marker.heap(), heap_marker);
    assert_eq!(ops_marker.heap().type_marker(), type_marker);
    assert_eq!(ops_marker.heap().source(), SourceId::new(7));
    assert_eq!(
        ops_marker.heap().type_marker().property_name(),
        property_name
    );
}

#[test]
fn property_key_and_descriptor_surface_is_reexported() {
    let symbol = SymbolRef::from_raw(9).unwrap();
    let key = PropertyKey::from_array_index(4).unwrap();
    let mut descriptor = PropertyDescriptor::new();
    let accessor = {
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_getter(Value::undefined());
        descriptor
    };

    descriptor.set_value(Value::undefined());
    descriptor.set_writable(false);

    assert_eq!(key.as_index(), Some(4));
    assert_eq!(PropertyKey::from_symbol(symbol).as_symbol(), Some(symbol));
    assert_eq!(
        PropertyKey::from_atom(AtomId::from_raw(12)).as_atom(),
        Some(AtomId::from_raw(12))
    );
    assert_eq!(descriptor.value(), Some(Value::undefined()));
    assert_eq!(descriptor.writable(), Some(false));
    assert_eq!(
        pure::descriptor_kind(descriptor),
        pure::DescriptorKind::Data
    );
    assert!(pure::is_accessor_descriptor(accessor));
    assert!(!pure::is_generic_descriptor(descriptor));
    assert_eq!(
        pure::complete_property_descriptor(PropertyDescriptor::new()).writable(),
        Some(false)
    );
}

fn completion_identity(value: Completion<Value>) -> Completion<Value> {
    let value = value?;
    Ok(value)
}

#[test]
fn completion_surface_is_reexported() {
    let thrown = AbruptCompletion::throw(Value::from_smi(5));
    let labeled_continue = AbruptCompletion::continue_(Some(AtomId::from_raw(18)));

    assert_eq!(completion_identity(Ok(Value::null())), Ok(Value::null()));
    assert_eq!(
        completion_identity(Err(thrown)),
        Err(AbruptCompletion::Throw(Value::from_smi(5)))
    );
    assert_eq!(
        labeled_continue.continue_label(),
        Some(Some(AtomId::from_raw(18)))
    );
}

#[test]
fn primitive_context_surface_is_reexported() {
    let mut heap = PrimitiveHeap::new();
    let mut atoms = AtomTable::new();
    let mut context = PrimitiveContext::new(&mut heap, &mut atoms);
    let string = allocating::alloc_latin1_string(&mut context, b"ctx", AllocationLifetime::Default);
    let numeric_string =
        allocating::alloc_latin1_string(&mut context, b"1", AllocationLifetime::Default);
    let lone_surrogate = context.mutator().alloc_string(
        StringEncoding::Utf16,
        1,
        &[0x00, 0xD8],
        None,
        AllocationLifetime::Default,
    );
    let bigint = context.mutator().alloc_bigint(
        lyng_js_gc::BigIntSign::Negative,
        &[9, 0],
        AllocationLifetime::Default,
    );
    let atom = context.atoms_mut().intern_collectible("ctx");

    assert!(pure::is_nullish(Value::null()));
    assert!(!pure::is_nullish(Value::from_string_ref(
        StringRef::from_raw(1).unwrap()
    )));
    assert!(allocating::memoize_string_atom(&mut context, string, atom));
    assert_eq!(
        read::strings_equal(context.heap(), string, string),
        Some(true)
    );
    assert_eq!(
        read::bigint_sign(context.heap(), bigint),
        Some(lyng_js_gc::BigIntSign::Negative)
    );
    assert_eq!(read::bigint_is_zero(context.heap(), bigint), Some(false));
    assert_eq!(read::string_code_unit_len(context.heap(), string), Some(3));
    assert_eq!(read::bigint_limb_count(context.heap(), bigint), Some(1));
    assert_eq!(
        read::to_number(context.heap(), Value::from_string_ref(numeric_string)),
        Ok(Value::from_smi(1))
    );
    assert_eq!(
        read::to_numeric(context.heap(), Value::from_bigint_ref(bigint)),
        Ok(Value::from_bigint_ref(bigint))
    );
    assert_eq!(
        read::is_loosely_equal(
            context.heap(),
            Value::from_bool(true),
            Value::from_string_ref(numeric_string),
        ),
        Ok(true)
    );
    assert_eq!(
        read::to_boolean(
            context.heap(),
            Value::from_symbol_ref(SymbolRef::from_raw(2).unwrap())
        ),
        Ok(true)
    );
    assert_eq!(
        read::same_value(context.heap(), Value::from_smi(0), Value::from_f64(-0.0)),
        Ok(false)
    );
    assert_eq!(
        read::same_value_zero(context.heap(), Value::from_smi(0), Value::from_f64(-0.0)),
        Ok(true)
    );
    assert_eq!(
        read::is_strictly_equal(context.heap(), Value::from_smi(1), Value::from_f64(1.0)),
        Ok(true)
    );
    assert_eq!(
        allocating::to_property_key(&mut context, Value::from_string_ref(string)),
        Some(PropertyKey::from_atom(atom))
    );
    let bigint_text = allocating::to_string(&mut context, Value::from_bigint_ref(bigint)).unwrap();
    let surrogate_key =
        allocating::to_property_key(&mut context, Value::from_string_ref(lone_surrogate)).unwrap();
    assert_eq!(
        allocating::to_property_key(
            &mut context,
            Value::from_symbol_ref(SymbolRef::from_raw(2).unwrap())
        ),
        Some(PropertyKey::from_symbol(SymbolRef::from_raw(2).unwrap()))
    );
    assert_eq!(context.atoms().get(surrogate_key.as_atom().unwrap()), None);
    assert!(context
        .atoms()
        .matches_utf16(surrogate_key.as_atom().unwrap(), &[0xD800]));
    assert_eq!(context.heap().string_payload(bigint_text), Some(&b"-9"[..]));
    assert_eq!(
        context.heap().string(string).unwrap().cached_atom(),
        Some(atom)
    );
    assert_eq!(BigIntRef::from_raw(bigint.get()), Some(bigint));
}
