use super::{
    DescriptorAttributes, InternalMethodError, InternalMethodResult, NamedPropertyValue,
    ShapeProperty, ShapePropertyKind, Value, MIN_DENSE_ELEMENT_CAPACITY,
    SMALL_SHAPE_INLINE_PROPERTY_LIMIT,
};
use lyng_js_gc::{ObjectSlotsRef, PrimitiveMutator, ValueStoreTarget};
use lyng_js_types::{PropertyDescriptor, PropertyKey};
use std::collections::HashMap;

#[inline]
pub(crate) fn flattened_property_lookup(
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

pub(crate) fn update_integrity_flags(
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
pub(crate) enum DescriptorKind {
    Generic,
    Data,
    Accessor,
}

pub(crate) fn descriptor_kind(
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

pub(crate) fn descriptor_from_payload(
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

pub(crate) fn payload_from_complete_descriptor(
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

pub(crate) fn complete_descriptor_update(
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

pub(crate) fn validate_descriptor_change(
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
    if let Some(enumerable) = update.enumerable() {
        if enumerable != current.enumerable().unwrap_or(false) {
            return Ok(false);
        }
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
                if let Some(value) = update.value() {
                    if value != current.value().unwrap_or(Value::undefined()) {
                        return Ok(false);
                    }
                }
            }
            Ok(true)
        }
        DescriptorKind::Accessor => {
            if let Some(getter) = update.getter() {
                if getter != current.getter().unwrap_or(Value::undefined()) {
                    return Ok(false);
                }
            }
            if let Some(setter) = update.setter() {
                if setter != current.setter().unwrap_or(Value::undefined()) {
                    return Ok(false);
                }
            }
            Ok(true)
        }
    }
}

pub(crate) fn resolve_get_from_descriptor(
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

pub(crate) fn write_named_payload(
    heap: &mut PrimitiveMutator<'_>,
    slots: ObjectSlotsRef,
    slot_offset: u32,
    payload: NamedPropertyValue,
    initialize: bool,
) -> InternalMethodResult<()> {
    let store = if initialize {
        PrimitiveMutator::init_store_value
    } else {
        PrimitiveMutator::mut_store_value
    };

    let success = match payload {
        NamedPropertyValue::Data(value) => store(
            heap,
            ValueStoreTarget::ObjectSlot(slots, slot_offset),
            value,
        ),
        NamedPropertyValue::Accessor { get, set } => {
            let setter_offset = slot_offset
                .checked_add(1)
                .expect("accessor setter slot offset overflowed supported u32 range");
            store(heap, ValueStoreTarget::ObjectSlot(slots, slot_offset), get)
                && store(
                    heap,
                    ValueStoreTarget::ObjectSlot(slots, setter_offset),
                    set,
                )
        }
    };

    if success {
        Ok(())
    } else {
        Err(InternalMethodError::CorruptObjectState)
    }
}

#[inline]
pub(crate) fn ordinary_property_attrs() -> DescriptorAttributes {
    let mut attrs = DescriptorAttributes::empty();
    attrs.set_writable(true);
    attrs.set_enumerable(true);
    attrs.set_configurable(true);
    attrs
}

#[inline]
pub(crate) fn dense_element_growth_capacity(current_capacity: usize, target_index: u32) -> usize {
    let required = target_index as usize + 1;
    let mut capacity = current_capacity.max(MIN_DENSE_ELEMENT_CAPACITY);
    while capacity < required {
        capacity = capacity.saturating_mul(2);
    }
    capacity
}

pub(crate) fn trim_dense_logical_len(buffer: &[Value], current_len: u32) -> u32 {
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
