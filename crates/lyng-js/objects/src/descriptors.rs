use super::{
    DescriptorAttributes, InternalMethodError, InternalMethodResult, NamedPropertyValue, ObjectRef,
    ObjectRuntime, ShapeProperty, ShapePropertyKind, SlotLocation, Value,
    MIN_DENSE_ELEMENT_CAPACITY, SMALL_SHAPE_INLINE_PROPERTY_LIMIT,
};
use lyng_js_gc::{PrimitiveHeapView, PrimitiveMutator, ValueStoreTarget};
use lyng_js_types::{PropertyDescriptor, PropertyKey};
use std::collections::HashMap;

#[inline]
pub fn flattened_property_lookup(
    properties: &[ShapeProperty],
) -> Option<HashMap<PropertyKey, usize>> {
    if properties.len() <= SMALL_SHAPE_INLINE_PROPERTY_LIMIT {
        return None;
    }

    let mut lookup = HashMap::with_capacity(properties.len());
    for (index, property) in properties.iter().enumerate() {
        lookup.insert(property.key(), index);
    }
    Some(lookup)
}

pub fn update_integrity_flags(
    kind: ShapePropertyKind,
    attrs: DescriptorAttributes,
    sealed: &mut bool,
    frozen: &mut bool,
) {
    if attrs.configurable() {
        *sealed = false;
        *frozen = false;
        return;
    }

    if kind == ShapePropertyKind::Data && attrs.writable() {
        *frozen = false;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DescriptorKind {
    Generic,
    Data,
    Accessor,
}

pub const fn descriptor_kind(
    descriptor: PropertyDescriptor,
) -> InternalMethodResult<DescriptorKind> {
    let is_data = descriptor.has_value() || descriptor.has_writable();
    let is_accessor = descriptor.has_get() || descriptor.has_set();
    match (is_data, is_accessor) {
        (true, true) => Err(InternalMethodError::InvalidDescriptor),
        (true, false) => Ok(DescriptorKind::Data),
        (false, true) => Ok(DescriptorKind::Accessor),
        (false, false) => Ok(DescriptorKind::Generic),
    }
}

pub const fn descriptor_from_payload(
    payload: NamedPropertyValue,
    attrs: DescriptorAttributes,
) -> PropertyDescriptor {
    let mut descriptor = PropertyDescriptor::new();
    match payload {
        NamedPropertyValue::Data(value) => {
            descriptor.set_value(value);
            descriptor.set_writable(attrs.writable());
        }
        NamedPropertyValue::Accessor { get, set } => {
            descriptor.set_getter(get);
            descriptor.set_setter(set);
        }
    }
    descriptor.set_enumerable(attrs.enumerable());
    descriptor.set_configurable(attrs.configurable());
    descriptor
}

pub fn payload_from_complete_descriptor(
    descriptor: PropertyDescriptor,
) -> InternalMethodResult<NamedPropertyValue> {
    match descriptor_kind(descriptor)? {
        DescriptorKind::Data | DescriptorKind::Generic => Ok(NamedPropertyValue::data(
            descriptor.value().unwrap_or(Value::undefined()),
        )),
        DescriptorKind::Accessor => Ok(NamedPropertyValue::accessor(
            descriptor.getter().unwrap_or(Value::undefined()),
            descriptor.setter().unwrap_or(Value::undefined()),
        )),
    }
}

pub fn complete_descriptor_update(
    current: Option<PropertyDescriptor>,
    update: PropertyDescriptor,
) -> InternalMethodResult<(NamedPropertyValue, DescriptorAttributes)> {
    let update_kind = descriptor_kind(update)?;
    let mut attrs = DescriptorAttributes::empty();

    match current {
        None => {
            attrs.set_enumerable(update.enumerable().unwrap_or(false));
            attrs.set_configurable(update.configurable().unwrap_or(false));
            match update_kind {
                DescriptorKind::Accessor => Ok((
                    NamedPropertyValue::accessor(
                        update.getter().unwrap_or(Value::undefined()),
                        update.setter().unwrap_or(Value::undefined()),
                    ),
                    attrs,
                )),
                DescriptorKind::Data | DescriptorKind::Generic => {
                    attrs.set_writable(update.writable().unwrap_or(false));
                    Ok((
                        NamedPropertyValue::data(update.value().unwrap_or(Value::undefined())),
                        attrs,
                    ))
                }
            }
        }
        Some(current) => {
            let current_kind = descriptor_kind(current)?;
            let target_kind = match update_kind {
                DescriptorKind::Generic => current_kind,
                other => other,
            };
            attrs = current.attrs();
            if let Some(enumerable) = update.enumerable() {
                attrs.set_enumerable(enumerable);
            }
            if let Some(configurable) = update.configurable() {
                attrs.set_configurable(configurable);
            }

            match target_kind {
                // `descriptor_from_payload` only materializes Data or Accessor descriptors, and
                // `DescriptorKind::Generic` updates inherit the current descriptor family above.
                DescriptorKind::Generic => unreachable!(),
                DescriptorKind::Data => {
                    let value = if update.has_value() {
                        update.value().unwrap_or(Value::undefined())
                    } else if current_kind == DescriptorKind::Data {
                        current.value().unwrap_or(Value::undefined())
                    } else {
                        Value::undefined()
                    };
                    let writable = if update.has_writable() {
                        update.writable().unwrap_or(false)
                    } else if current_kind == DescriptorKind::Data {
                        current.writable().unwrap_or(false)
                    } else {
                        false
                    };
                    attrs.set_writable(writable);
                    Ok((NamedPropertyValue::data(value), attrs))
                }
                DescriptorKind::Accessor => {
                    attrs.set_writable(false);
                    let existing_get = if current_kind == DescriptorKind::Accessor {
                        current.getter().unwrap_or(Value::undefined())
                    } else {
                        Value::undefined()
                    };
                    let existing_set = if current_kind == DescriptorKind::Accessor {
                        current.setter().unwrap_or(Value::undefined())
                    } else {
                        Value::undefined()
                    };
                    let get = if update.has_get() {
                        update.getter().unwrap_or(Value::undefined())
                    } else {
                        existing_get
                    };
                    let set = if update.has_set() {
                        update.setter().unwrap_or(Value::undefined())
                    } else {
                        existing_set
                    };
                    Ok((NamedPropertyValue::accessor(get, set), attrs))
                }
            }
        }
    }
}

pub fn validate_descriptor_change(
    heap: PrimitiveHeapView<'_>,
    current: PropertyDescriptor,
    update: PropertyDescriptor,
) -> InternalMethodResult<bool> {
    let current_kind = descriptor_kind(current)?;
    let update_kind = descriptor_kind(update)?;
    let current_configurable = current.configurable().unwrap_or(false);
    if current_configurable {
        return Ok(true);
    }

    if update.configurable() == Some(true) {
        return Ok(false);
    }
    if let Some(enumerable) = update.enumerable()
        && enumerable != current.enumerable().unwrap_or(false)
    {
        return Ok(false);
    }
    if update_kind != DescriptorKind::Generic && update_kind != current_kind {
        return Ok(false);
    }

    match current_kind {
        DescriptorKind::Generic => Ok(true),
        DescriptorKind::Data => {
            let current_writable = current.writable().unwrap_or(false);
            if !current_writable {
                if update.writable() == Some(true) {
                    return Ok(false);
                }
                if let Some(value) = update.value()
                    && !descriptor_same_value(
                        heap,
                        value,
                        current.value().unwrap_or(Value::undefined()),
                    )?
                {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        DescriptorKind::Accessor => {
            if let Some(getter) = update.getter()
                && getter != current.getter().unwrap_or(Value::undefined())
            {
                return Ok(false);
            }
            if let Some(setter) = update.setter()
                && setter != current.setter().unwrap_or(Value::undefined())
            {
                return Ok(false);
            }
            Ok(true)
        }
    }
}

pub fn descriptor_same_value(
    heap: PrimitiveHeapView<'_>,
    left: Value,
    right: Value,
) -> InternalMethodResult<bool> {
    match (left.as_f64(), right.as_f64()) {
        (Some(left), Some(right)) => {
            if left.is_nan() && right.is_nan() {
                return Ok(true);
            }
            return Ok(left.to_bits() == right.to_bits());
        }
        (Some(_), None) | (None, Some(_)) => return Ok(false),
        (None, None) => {}
    }

    match (left.as_string_ref(), right.as_string_ref()) {
        (Some(left), Some(right)) => {
            return heap
                .strings_equal(left, right)
                .ok_or(InternalMethodError::CorruptObjectState);
        }
        (Some(_), None) | (None, Some(_)) => return Ok(false),
        (None, None) => {}
    }

    match (left.as_bigint_ref(), right.as_bigint_ref()) {
        (Some(left), Some(right)) => {
            let left = heap
                .bigint_view(left)
                .ok_or(InternalMethodError::CorruptObjectState)?;
            let right = heap
                .bigint_view(right)
                .ok_or(InternalMethodError::CorruptObjectState)?;
            return Ok(left.sign() == right.sign() && left.limb_bytes_le() == right.limb_bytes_le());
        }
        (Some(_), None) | (None, Some(_)) => return Ok(false),
        (None, None) => {}
    }

    Ok(left == right)
}

pub fn resolve_get_from_descriptor(
    descriptor: PropertyDescriptor,
    _receiver: Value,
) -> InternalMethodResult<Value> {
    match descriptor_kind(descriptor)? {
        DescriptorKind::Generic | DescriptorKind::Data => {
            Ok(descriptor.value().unwrap_or(Value::undefined()))
        }
        DescriptorKind::Accessor => {
            let getter = descriptor.getter().unwrap_or(Value::undefined());
            if getter == Value::undefined() {
                Ok(Value::undefined())
            } else {
                Err(InternalMethodError::AccessorCallPending)
            }
        }
    }
}

/// Write one shape-stable named-property payload into the holder's slot storage.
///
/// Dispatches on the encoded `slot_offset` via the decoded [`SlotLocation`]:
/// - inline → routes the write to [`ValueStoreTarget::InlineNamedSlot`], which targets
///   `RuntimeObjectRecord.inline_named_slots` and runs the standard incremental-marking
///   value barrier on the holder.
/// - out-of-line → routes the write to [`ValueStoreTarget::ObjectSlot`] against
///   `holder.named_slots()` exactly as before.
///
/// For accessor properties the setter is written at the next consecutive position within
/// the same storage tier (inline slot `index+1` or out-of-line slot `offset+1`).
pub fn write_named_payload(
    _runtime: &mut ObjectRuntime,
    heap: &mut PrimitiveMutator<'_>,
    holder: ObjectRef,
    slot_offset: u32,
    payload: NamedPropertyValue,
    initialize: bool,
) -> InternalMethodResult<()> {
    let primary_target = match SlotLocation::decode(slot_offset) {
        SlotLocation::Inline(index) => ValueStoreTarget::InlineNamedSlot(holder, index),
        SlotLocation::OutOfLine(offset) => {
            let slots = heap
                .view()
                .object(holder)
                .and_then(super::RuntimeObjectRecord::named_slots)
                .ok_or(InternalMethodError::CorruptObjectState)?;
            ValueStoreTarget::ObjectSlot(slots, offset)
        }
    };
    let store = if initialize {
        PrimitiveMutator::init_store_value
    } else {
        PrimitiveMutator::mut_store_value
    };

    let success = match payload {
        NamedPropertyValue::Data(value) => store(heap, primary_target, value),
        NamedPropertyValue::Accessor { get, set } => {
            let setter_target = match primary_target {
                ValueStoreTarget::InlineNamedSlot(id, index) => {
                    let setter_index = index
                        .checked_add(1)
                        .expect("inline accessor setter index overflowed");
                    ValueStoreTarget::InlineNamedSlot(id, setter_index)
                }
                ValueStoreTarget::ObjectSlot(slots, offset) => {
                    let setter_offset = offset
                        .checked_add(1)
                        .expect("out-of-line accessor setter slot offset overflowed");
                    ValueStoreTarget::ObjectSlot(slots, setter_offset)
                }
                _ => unreachable!("named payload targets are slot-shaped"),
            };
            store(heap, primary_target, get) && store(heap, setter_target, set)
        }
    };

    if success {
        Ok(())
    } else {
        Err(InternalMethodError::CorruptObjectState)
    }
}

#[inline]
pub const fn ordinary_property_attrs() -> DescriptorAttributes {
    let mut attrs = DescriptorAttributes::empty();
    attrs.set_writable(true);
    attrs.set_enumerable(true);
    attrs.set_configurable(true);
    attrs
}

#[inline]
pub fn dense_element_growth_capacity(current_capacity: usize, target_index: u32) -> usize {
    let required = target_index as usize + 1;
    let mut capacity = current_capacity.max(MIN_DENSE_ELEMENT_CAPACITY);
    while capacity < required {
        capacity = capacity.saturating_mul(2);
    }
    capacity
}

pub fn trim_dense_logical_len(buffer: &[Value], current_len: u32) -> u32 {
    let mut trimmed = current_len as usize;
    while trimmed > 0 {
        if buffer
            .get(trimmed - 1)
            .copied()
            .unwrap_or(Value::array_hole())
            != Value::array_hole()
        {
            break;
        }
        trimmed -= 1;
    }
    u32::try_from(trimmed).expect("dense logical length must fit into u32")
}
