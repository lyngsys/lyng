use crate::{
    convert::{bigint_view_to_string, number_to_string, primitive_type_error},
    PrimitiveContext,
};
use lyng_js_common::AtomId;
use lyng_js_gc::{AllocationLifetime, PrimitiveStringView};
use lyng_js_types::{Completion, PropertyKey, StringRef, Value};

/// Allocates a flat Latin-1 runtime string through the explicit primitive context.
///
/// # Panics
///
/// Panics if `bytes.len()` does not fit into the runtime string length field.
#[inline]
pub fn alloc_latin1_string(
    context: &mut PrimitiveContext<'_>,
    bytes: &[u8],
    lifetime: AllocationLifetime,
) -> StringRef {
    context.mutator().alloc_string(
        lyng_js_gc::StringEncoding::Latin1,
        u32::try_from(bytes.len()).expect("flat Latin-1 string length must fit into u32"),
        bytes,
        None,
        lifetime,
    )
}

/// Memoizes an atomized string key through the explicit primitive context.
#[inline]
pub fn memoize_string_atom(
    context: &mut PrimitiveContext<'_>,
    string: StringRef,
    atom: AtomId,
) -> bool {
    context.mutator().memoize_string_atom(string, atom)
}

/// ECMAScript `ToString` over the primitive-runtime surface.
///
/// This covers primitive values only. Object coercion remains an object-aware
/// operation layered on top of the same entrypoint.
///
/// # Errors
///
/// Returns a primitive type error for symbols, objects, internal sentinels, and
/// invalid bigint handles.
///
/// # Panics
///
/// Panics if a value tagged as a number does not expose a numeric payload,
/// which indicates a corrupted internal `Value`.
pub fn to_string(context: &mut PrimitiveContext<'_>, value: Value) -> Completion<StringRef> {
    if value.is_undefined() {
        return Ok(alloc_latin1_string(
            context,
            b"undefined",
            AllocationLifetime::Default,
        ));
    }
    if value.is_null() {
        return Ok(alloc_latin1_string(
            context,
            b"null",
            AllocationLifetime::Default,
        ));
    }
    if let Some(boolean) = value.as_bool() {
        let bytes: &[u8] = if boolean { &b"true"[..] } else { &b"false"[..] };
        return Ok(alloc_latin1_string(
            context,
            bytes,
            AllocationLifetime::Default,
        ));
    }
    if value.is_number() {
        let text = number_to_string(value.as_f64().unwrap());
        return Ok(alloc_latin1_string(
            context,
            text.as_bytes(),
            AllocationLifetime::Default,
        ));
    }
    if let Some(string) = value.as_string_ref() {
        return Ok(string);
    }
    if value.is_symbol() {
        return Err(primitive_type_error());
    }
    if let Some(bigint) = value.as_bigint_ref() {
        let text = {
            let view = context
                .heap()
                .bigint_view(bigint)
                .ok_or_else(primitive_type_error)?;
            bigint_view_to_string(view)
        };
        return Ok(alloc_latin1_string(
            context,
            text.as_bytes(),
            AllocationLifetime::Default,
        ));
    }
    if value.is_object() || value.is_sentinel() {
        return Err(primitive_type_error());
    }

    Err(primitive_type_error())
}

/// Converts a heap-backed runtime string into a property key.
///
/// Canonical array-index names lower to `PropertyKey::Index`; all other string
/// keys atomize through the shared collectible runtime atom table and memoize
/// the resulting `AtomId` back to the string record.
pub fn string_to_property_key(
    context: &mut PrimitiveContext<'_>,
    string: StringRef,
) -> Option<PropertyKey> {
    let (mut mutator, atoms) = context.split_mut();
    let (array_index, cached_atom, code_units) = {
        let view = mutator.string_view(string)?;
        let array_index = canonical_array_index(view);
        let cached_atom = view.cached_atom();
        let code_units = if array_index.is_none() && cached_atom.is_none() {
            Some(string_code_units(view)?)
        } else {
            None
        };
        (array_index, cached_atom, code_units)
    };

    if let Some(index) = array_index {
        return Some(PropertyKey::Index(index));
    }

    if let Some(atom) = cached_atom {
        return Some(PropertyKey::from_atom(atom));
    }

    let atom = atoms.intern_collectible_utf16(code_units.as_deref()?);
    if !mutator.memoize_string_atom(string, atom) {
        return None;
    }
    Some(PropertyKey::from_atom(atom))
}

/// Converts a primitive `Value` into a property key for the currently supported
/// direct string and symbol cases.
#[inline]
pub fn to_property_key(context: &mut PrimitiveContext<'_>, value: Value) -> Option<PropertyKey> {
    if let Some(symbol) = value.as_symbol_ref() {
        return Some(PropertyKey::from_symbol(symbol));
    }

    string_to_property_key(context, value.as_string_ref()?)
}

fn canonical_array_index(view: PrimitiveStringView<'_>) -> Option<u32> {
    let len = view.code_unit_len() as usize;
    if len == 0 {
        return None;
    }

    let first = ascii_digit_value(view.code_unit_at(0)?)?;
    if first == 0 {
        return (len == 1).then_some(0);
    }

    let mut value = u64::from(first);
    for index in 1..len {
        let digit = u64::from(ascii_digit_value(view.code_unit_at(index)?)?);
        value = value.checked_mul(10)?.checked_add(digit)?;
        if value > u64::from(PropertyKey::MAX_ARRAY_INDEX) {
            return None;
        }
    }

    u32::try_from(value).ok()
}

fn ascii_digit_value(code_unit: u16) -> Option<u8> {
    if (u16::from(b'0')..=u16::from(b'9')).contains(&code_unit) {
        u8::try_from(code_unit - u16::from(b'0')).ok()
    } else {
        None
    }
}

fn string_code_units(view: PrimitiveStringView<'_>) -> Option<Vec<u16>> {
    if let Some(bytes) = view.latin1_bytes() {
        return Some(bytes.iter().copied().map(u16::from).collect());
    }

    let bytes = view.utf16_bytes()?;
    let mut units = Vec::with_capacity(view.code_unit_len() as usize);
    for chunk in bytes.chunks_exact(2) {
        units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    Some(units)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_common::AtomTable;
    use lyng_js_gc::{PrimitiveHeap, StringEncoding};
    use lyng_js_types::{AbruptCompletion, SymbolRef};

    #[test]
    fn allocating_helpers_take_mutable_primitive_context() {
        let mut heap = PrimitiveHeap::new();
        let mut atoms = AtomTable::new();
        let mut context = PrimitiveContext::new(&mut heap, &mut atoms);
        let string = alloc_latin1_string(&mut context, b"alloc", AllocationLifetime::Default);
        let atom = context.atoms_mut().intern_collectible("alloc");

        assert!(memoize_string_atom(&mut context, string, atom));
        assert_eq!(
            context.heap().string(string).unwrap().cached_atom(),
            Some(atom)
        );
    }

    #[test]
    fn string_to_property_key_atomizes_once_and_reuses_cached_atoms() {
        let mut heap = PrimitiveHeap::new();
        let mut atoms = AtomTable::new();
        let mut context = PrimitiveContext::new(&mut heap, &mut atoms);
        let string = context.mutator().alloc_string(
            StringEncoding::Latin1,
            1,
            &[0xE9],
            None,
            AllocationLifetime::Default,
        );

        let atom_count_before = context.atoms().len();
        let first = string_to_property_key(&mut context, string).unwrap();
        let atom_count_after_first = context.atoms().len();
        let second = string_to_property_key(&mut context, string).unwrap();

        let atom = first.as_atom().unwrap();
        assert_eq!(first, second);
        assert_eq!(atom_count_after_first, atom_count_before + 1);
        assert_eq!(context.atoms().len(), atom_count_after_first);
        assert_eq!(context.atoms().resolve(atom), "é");
        assert_eq!(
            context.heap().string(string).unwrap().cached_atom(),
            Some(atom)
        );
    }

    #[test]
    fn string_to_property_key_handles_array_indices_and_utf16_atoms() {
        let mut heap = PrimitiveHeap::new();
        let mut atoms = AtomTable::new();
        let mut context = PrimitiveContext::new(&mut heap, &mut atoms);
        let index_string = context.mutator().alloc_string(
            StringEncoding::Latin1,
            3,
            b"123",
            None,
            AllocationLifetime::Default,
        );
        let wide_string = context.mutator().alloc_string(
            StringEncoding::Utf16,
            1,
            &[0xA9, 0x03],
            None,
            AllocationLifetime::Default,
        );

        let atom_count_before = context.atoms().len();
        assert_eq!(
            string_to_property_key(&mut context, index_string),
            Some(PropertyKey::from_array_index(123).unwrap())
        );
        let wide_key = string_to_property_key(&mut context, wide_string).unwrap();

        assert_eq!(context.atoms().len(), atom_count_before + 1);
        assert_eq!(context.atoms().resolve(wide_key.as_atom().unwrap()), "Ω");
        assert_eq!(
            context.heap().string(index_string).unwrap().cached_atom(),
            None
        );
    }

    #[test]
    fn string_to_property_key_preserves_lone_surrogate_utf16_strings() {
        let mut heap = PrimitiveHeap::new();
        let mut atoms = AtomTable::new();
        let mut context = PrimitiveContext::new(&mut heap, &mut atoms);
        let lone_surrogate = context.mutator().alloc_string(
            StringEncoding::Utf16,
            1,
            &[0x00, 0xD8],
            None,
            AllocationLifetime::Default,
        );

        let atom_count_before = context.atoms().len();
        let key = string_to_property_key(&mut context, lone_surrogate).unwrap();
        let atom = key.as_atom().unwrap();

        assert_eq!(context.atoms().len(), atom_count_before + 1);
        assert_eq!(context.atoms().get(atom), None);
        assert!(context.atoms().matches_utf16(atom, &[0xD800]));
        assert_eq!(
            context.heap().string(lone_surrogate).unwrap().cached_atom(),
            Some(atom)
        );
        assert_eq!(
            string_to_property_key(&mut context, lone_surrogate),
            Some(PropertyKey::from_atom(atom))
        );
    }

    #[test]
    fn to_property_key_accepts_direct_string_and_symbol_values() {
        let mut heap = PrimitiveHeap::new();
        let mut atoms = AtomTable::new();
        let mut context = PrimitiveContext::new(&mut heap, &mut atoms);
        let string = alloc_latin1_string(&mut context, b"name", AllocationLifetime::Default);
        let symbol = SymbolRef::from_raw(9).unwrap();

        let string_key = to_property_key(&mut context, Value::from_string_ref(string)).unwrap();
        let symbol_key = to_property_key(&mut context, Value::from_symbol_ref(symbol)).unwrap();

        assert_eq!(
            context.atoms().resolve(string_key.as_atom().unwrap()),
            "name"
        );
        assert_eq!(symbol_key, PropertyKey::from_symbol(symbol));
        assert_eq!(to_property_key(&mut context, Value::from_smi(7)), None);
    }

    #[test]
    fn to_string_covers_primitive_cases() {
        let mut heap = PrimitiveHeap::new();
        let mut atoms = AtomTable::new();
        let mut context = PrimitiveContext::new(&mut heap, &mut atoms);
        let existing = alloc_latin1_string(&mut context, b"keep", AllocationLifetime::Default);
        let bigint = context.mutator().alloc_bigint(
            lyng_js_gc::BigIntSign::Negative,
            &[42],
            AllocationLifetime::Default,
        );

        let undefined = to_string(&mut context, Value::undefined()).unwrap();
        let null = to_string(&mut context, Value::null()).unwrap();
        let boolean = to_string(&mut context, Value::from_bool(true)).unwrap();
        let number = to_string(&mut context, Value::from_f64(-0.0)).unwrap();
        let bigint_text = to_string(&mut context, Value::from_bigint_ref(bigint)).unwrap();

        assert_eq!(
            to_string(&mut context, Value::from_string_ref(existing)),
            Ok(existing)
        );
        assert_eq!(
            to_string(
                &mut context,
                Value::from_symbol_ref(SymbolRef::from_raw(11).unwrap())
            ),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
        let scientific = to_string(&mut context, Value::from_f64(1e21)).unwrap();
        let fractional_scientific = to_string(&mut context, Value::from_f64(1e-7)).unwrap();
        let fixed = to_string(&mut context, Value::from_f64(1e20)).unwrap();
        assert_eq!(
            to_string(
                &mut context,
                Value::from_object_ref(lyng_js_types::ObjectRef::from_raw(12).unwrap())
            ),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
        assert_eq!(
            to_string(&mut context, Value::array_hole()),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
        assert_eq!(
            context.heap().string_payload(undefined),
            Some(&b"undefined"[..])
        );
        assert_eq!(context.heap().string_payload(null), Some(&b"null"[..]));
        assert_eq!(context.heap().string_payload(boolean), Some(&b"true"[..]));
        assert_eq!(context.heap().string_payload(number), Some(&b"0"[..]));
        assert_eq!(
            context.heap().string_payload(scientific),
            Some(&b"1e+21"[..])
        );
        assert_eq!(
            context.heap().string_payload(fractional_scientific),
            Some(&b"1e-7"[..])
        );
        assert_eq!(
            context.heap().string_payload(fixed),
            Some(&b"100000000000000000000"[..])
        );
        assert_eq!(
            context.heap().string_payload(bigint_text),
            Some(&b"-42"[..])
        );
    }

    #[test]
    fn invalid_runtime_handles_fail_without_panicking() {
        let mut heap = PrimitiveHeap::new();
        let mut atoms = AtomTable::new();
        let mut context = PrimitiveContext::new(&mut heap, &mut atoms);
        let invalid_string = StringRef::from_raw(55).unwrap();
        let invalid_bigint = lyng_js_types::BigIntRef::from_raw(56).unwrap();

        assert_eq!(string_to_property_key(&mut context, invalid_string), None);
        assert_eq!(
            to_string(&mut context, Value::from_bigint_ref(invalid_bigint)),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
    }
}
