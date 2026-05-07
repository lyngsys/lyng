use super::{SymbolRef, Value};
use lyng_js_common::AtomId;

/// Spec-facing property key classification used by slow-path runtime helpers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PropertyKey {
    Index(u32),
    Atom(AtomId),
    Symbol(SymbolRef),
}

impl PropertyKey {
    pub const MAX_ARRAY_INDEX: u32 = u32::MAX - 1;

    /// Attempts to construct a canonical array-index key.
    #[inline]
    pub fn from_array_index(index: u64) -> Option<Self> {
        u32::try_from(index)
            .ok()
            .filter(|index| *index <= Self::MAX_ARRAY_INDEX)
            .map(Self::Index)
    }

    /// Returns whether `index` falls inside the canonical array-index range.
    #[inline]
    pub const fn is_array_index(index: u64) -> bool {
        index <= Self::MAX_ARRAY_INDEX as u64
    }

    #[inline]
    pub const fn from_atom(atom: AtomId) -> Self {
        Self::Atom(atom)
    }

    #[inline]
    pub const fn from_symbol(symbol: SymbolRef) -> Self {
        Self::Symbol(symbol)
    }

    #[inline]
    pub const fn is_index(self) -> bool {
        matches!(self, Self::Index(_))
    }

    #[inline]
    pub const fn is_atom(self) -> bool {
        matches!(self, Self::Atom(_))
    }

    #[inline]
    pub const fn is_symbol(self) -> bool {
        matches!(self, Self::Symbol(_))
    }

    #[inline]
    pub const fn as_index(self) -> Option<u32> {
        match self {
            Self::Index(index) => Some(index),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_atom(self) -> Option<AtomId> {
        match self {
            Self::Atom(atom) => Some(atom),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_symbol(self) -> Option<SymbolRef> {
        match self {
            Self::Symbol(symbol) => Some(symbol),
            _ => None,
        }
    }
}

/// Presence bits for spec-facing descriptor fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct DescriptorPresent(u8);

impl DescriptorPresent {
    const VALUE: u8 = 1 << 0;
    const GET: u8 = 1 << 1;
    const SET: u8 = 1 << 2;
    const WRITABLE: u8 = 1 << 3;
    const ENUMERABLE: u8 = 1 << 4;
    const CONFIGURABLE: u8 = 1 << 5;

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn bits(self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn has_value(self) -> bool {
        self.contains(Self::VALUE)
    }

    #[inline]
    pub const fn has_get(self) -> bool {
        self.contains(Self::GET)
    }

    #[inline]
    pub const fn has_set(self) -> bool {
        self.contains(Self::SET)
    }

    #[inline]
    pub const fn has_writable(self) -> bool {
        self.contains(Self::WRITABLE)
    }

    #[inline]
    pub const fn has_enumerable(self) -> bool {
        self.contains(Self::ENUMERABLE)
    }

    #[inline]
    pub const fn has_configurable(self) -> bool {
        self.contains(Self::CONFIGURABLE)
    }

    #[inline]
    const fn contains(self, mask: u8) -> bool {
        (self.0 & mask) != 0
    }

    #[inline]
    const fn insert(&mut self, mask: u8) {
        self.0 |= mask;
    }

    #[inline]
    const fn remove(&mut self, mask: u8) {
        self.0 &= !mask;
    }
}

/// Boolean attribute values for spec-facing descriptors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct DescriptorAttributes(u8);

impl DescriptorAttributes {
    const WRITABLE: u8 = 1 << 0;
    const ENUMERABLE: u8 = 1 << 1;
    const CONFIGURABLE: u8 = 1 << 2;

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn bits(self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn writable(self) -> bool {
        self.contains(Self::WRITABLE)
    }

    #[inline]
    pub const fn enumerable(self) -> bool {
        self.contains(Self::ENUMERABLE)
    }

    #[inline]
    pub const fn configurable(self) -> bool {
        self.contains(Self::CONFIGURABLE)
    }

    #[inline]
    pub const fn set_writable(&mut self, writable: bool) {
        self.set(Self::WRITABLE, writable);
    }

    #[inline]
    pub const fn set_enumerable(&mut self, enumerable: bool) {
        self.set(Self::ENUMERABLE, enumerable);
    }

    #[inline]
    pub const fn set_configurable(&mut self, configurable: bool) {
        self.set(Self::CONFIGURABLE, configurable);
    }

    #[inline]
    pub const fn clear_writable(&mut self) {
        self.0 &= !Self::WRITABLE;
    }

    #[inline]
    pub const fn clear_enumerable(&mut self) {
        self.0 &= !Self::ENUMERABLE;
    }

    #[inline]
    pub const fn clear_configurable(&mut self) {
        self.0 &= !Self::CONFIGURABLE;
    }

    #[inline]
    const fn contains(self, mask: u8) -> bool {
        (self.0 & mask) != 0
    }

    #[inline]
    const fn set(&mut self, mask: u8, enabled: bool) {
        if enabled {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }
    }
}

/// Spec-facing property descriptor with explicit field-presence tracking.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct PropertyDescriptor {
    present: DescriptorPresent,
    attrs: DescriptorAttributes,
    value: Value,
    get: Value,
    set: Value,
}

impl PropertyDescriptor {
    #[inline]
    pub const fn new() -> Self {
        Self {
            present: DescriptorPresent::empty(),
            attrs: DescriptorAttributes::empty(),
            value: Value::undefined(),
            get: Value::undefined(),
            set: Value::undefined(),
        }
    }

    #[inline]
    pub const fn present(self) -> DescriptorPresent {
        self.present
    }

    #[inline]
    pub const fn attrs(self) -> DescriptorAttributes {
        self.attrs
    }

    #[inline]
    pub const fn has_value(self) -> bool {
        self.present.has_value()
    }

    #[inline]
    pub const fn value(self) -> Option<Value> {
        if self.has_value() {
            Some(self.value)
        } else {
            None
        }
    }

    #[inline]
    pub const fn has_get(self) -> bool {
        self.present.has_get()
    }

    #[inline]
    pub const fn getter(self) -> Option<Value> {
        if self.has_get() {
            Some(self.get)
        } else {
            None
        }
    }

    #[inline]
    pub const fn has_set(self) -> bool {
        self.present.has_set()
    }

    #[inline]
    pub const fn setter(self) -> Option<Value> {
        if self.has_set() {
            Some(self.set)
        } else {
            None
        }
    }

    #[inline]
    pub const fn has_writable(self) -> bool {
        self.present.has_writable()
    }

    #[inline]
    pub const fn writable(self) -> Option<bool> {
        if self.has_writable() {
            Some(self.attrs.writable())
        } else {
            None
        }
    }

    #[inline]
    pub const fn has_enumerable(self) -> bool {
        self.present.has_enumerable()
    }

    #[inline]
    pub const fn enumerable(self) -> Option<bool> {
        if self.has_enumerable() {
            Some(self.attrs.enumerable())
        } else {
            None
        }
    }

    #[inline]
    pub const fn has_configurable(self) -> bool {
        self.present.has_configurable()
    }

    #[inline]
    pub const fn configurable(self) -> Option<bool> {
        if self.has_configurable() {
            Some(self.attrs.configurable())
        } else {
            None
        }
    }

    #[inline]
    pub const fn set_value(&mut self, value: Value) {
        self.value = value;
        self.present.insert(DescriptorPresent::VALUE);
    }

    #[inline]
    pub const fn clear_value(&mut self) {
        self.value = Value::undefined();
        self.present.remove(DescriptorPresent::VALUE);
    }

    #[inline]
    pub const fn set_getter(&mut self, getter: Value) {
        self.get = getter;
        self.present.insert(DescriptorPresent::GET);
    }

    #[inline]
    pub const fn clear_getter(&mut self) {
        self.get = Value::undefined();
        self.present.remove(DescriptorPresent::GET);
    }

    #[inline]
    pub const fn set_setter(&mut self, setter: Value) {
        self.set = setter;
        self.present.insert(DescriptorPresent::SET);
    }

    #[inline]
    pub const fn clear_setter(&mut self) {
        self.set = Value::undefined();
        self.present.remove(DescriptorPresent::SET);
    }

    #[inline]
    pub const fn set_writable(&mut self, writable: bool) {
        self.attrs.set_writable(writable);
        self.present.insert(DescriptorPresent::WRITABLE);
    }

    #[inline]
    pub const fn clear_writable(&mut self) {
        self.attrs.clear_writable();
        self.present.remove(DescriptorPresent::WRITABLE);
    }

    #[inline]
    pub const fn set_enumerable(&mut self, enumerable: bool) {
        self.attrs.set_enumerable(enumerable);
        self.present.insert(DescriptorPresent::ENUMERABLE);
    }

    #[inline]
    pub const fn clear_enumerable(&mut self) {
        self.attrs.clear_enumerable();
        self.present.remove(DescriptorPresent::ENUMERABLE);
    }

    #[inline]
    pub const fn set_configurable(&mut self, configurable: bool) {
        self.attrs.set_configurable(configurable);
        self.present.insert(DescriptorPresent::CONFIGURABLE);
    }

    #[inline]
    pub const fn clear_configurable(&mut self) {
        self.attrs.clear_configurable();
        self.present.remove(DescriptorPresent::CONFIGURABLE);
    }
}

impl Default for PropertyDescriptor {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ObjectRef;
    use std::mem::size_of;

    #[test]
    fn property_keys_cover_index_atom_and_symbol_cases() {
        let index = PropertyKey::from_array_index(7).unwrap();
        let atom = PropertyKey::from_atom(AtomId::from_raw(11));
        let symbol = PropertyKey::from_symbol(SymbolRef::from_raw(13).unwrap());

        assert_eq!(index.as_index(), Some(7));
        assert_eq!(atom.as_atom(), Some(AtomId::from_raw(11)));
        assert_eq!(symbol.as_symbol(), SymbolRef::from_raw(13));
        assert!(index.is_index());
        assert!(atom.is_atom());
        assert!(symbol.is_symbol());
        assert_eq!(index.as_atom(), None);
        assert_eq!(atom.as_symbol(), None);
    }

    #[test]
    fn property_key_array_index_boundary_is_explicit() {
        assert_eq!(PropertyKey::MAX_ARRAY_INDEX, u32::MAX - 1);
        assert!(PropertyKey::is_array_index(0));
        assert!(PropertyKey::is_array_index(u64::from(u32::MAX - 1)));
        assert_eq!(
            PropertyKey::from_array_index(u64::from(u32::MAX - 1)),
            Some(PropertyKey::Index(u32::MAX - 1))
        );
        assert!(!PropertyKey::is_array_index(u64::from(u32::MAX)));
        assert_eq!(PropertyKey::from_array_index(u64::from(u32::MAX)), None);
        assert_eq!(PropertyKey::from_array_index(u64::from(u32::MAX) + 1), None);
    }

    #[test]
    fn descriptor_distinguishes_absent_fields_from_present_false_or_undefined() {
        let absent = PropertyDescriptor::new();
        let mut present_undefined = PropertyDescriptor::new();
        let mut present_false = PropertyDescriptor::new();

        present_undefined.set_value(Value::undefined());
        present_false.set_writable(false);

        assert_eq!(absent.value(), None);
        assert_eq!(present_undefined.value(), Some(Value::undefined()));
        assert_eq!(absent.writable(), None);
        assert_eq!(present_false.writable(), Some(false));
        assert_ne!(absent, present_undefined);
        assert_ne!(absent, present_false);
    }

    #[test]
    fn descriptor_tracks_payloads_attributes_and_presence_bits_independently() {
        let mut descriptor = PropertyDescriptor::new();
        let value = Value::from_object_ref(ObjectRef::from_raw(3).unwrap());

        descriptor.set_value(value);
        descriptor.set_getter(Value::undefined());
        descriptor.set_setter(Value::from_bool(true));
        descriptor.set_writable(true);
        descriptor.set_enumerable(false);
        descriptor.set_configurable(true);

        assert_eq!(descriptor.value(), Some(value));
        assert_eq!(descriptor.getter(), Some(Value::undefined()));
        assert_eq!(descriptor.setter(), Some(Value::from_bool(true)));
        assert_eq!(descriptor.writable(), Some(true));
        assert_eq!(descriptor.enumerable(), Some(false));
        assert_eq!(descriptor.configurable(), Some(true));
        assert!(descriptor.present().has_value());
        assert!(descriptor.present().has_get());
        assert!(descriptor.present().has_set());
        assert!(descriptor.present().has_writable());
        assert!(descriptor.present().has_enumerable());
        assert!(descriptor.present().has_configurable());
        assert!(descriptor.attrs().writable());
        assert!(!descriptor.attrs().enumerable());
        assert!(descriptor.attrs().configurable());
    }

    #[test]
    fn clearing_descriptor_fields_returns_to_empty_state() {
        let mut descriptor = PropertyDescriptor::new();

        descriptor.set_value(Value::from_smi(17));
        descriptor.set_getter(Value::from_bool(false));
        descriptor.set_setter(Value::from_bool(true));
        descriptor.set_writable(true);
        descriptor.set_enumerable(true);
        descriptor.set_configurable(false);

        descriptor.clear_value();
        descriptor.clear_getter();
        descriptor.clear_setter();
        descriptor.clear_writable();
        descriptor.clear_enumerable();
        descriptor.clear_configurable();

        assert_eq!(descriptor, PropertyDescriptor::new());
        assert!(descriptor.present().is_empty());
        assert_eq!(descriptor.attrs().bits(), 0);
    }

    #[test]
    fn descriptor_bitfields_stay_compact() {
        assert_eq!(size_of::<DescriptorPresent>(), 1);
        assert_eq!(size_of::<DescriptorAttributes>(), 1);
    }
}
