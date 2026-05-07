use super::temporal::TemporalObjectKind;
use super::{
    BackingStoreRef, BuiltinFunctionId, CodeRef, EmbeddingFunctionId, EnvironmentRef,
    FunctionPayloadRef, InternalMethodResult, NativeFunctionId, ObjectKind, ObjectRef,
    ObjectRuntime, PrimitiveMutator, RealmRef, RuntimeFunctionRecord, Value,
};
use lyng_js_gc::RuntimeBoundFunctionRecord;

/// Cold payload carried by ordinary objects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveWrapperKind {
    String,
    Number,
    Boolean,
    Symbol,
    BigInt,
}

/// Cold payload carried by one `ArrayBuffer` wrapper object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ArrayBufferObjectData {
    backing_store: BackingStoreRef,
    max_byte_length: Option<usize>,
}

impl ArrayBufferObjectData {
    #[inline]
    pub const fn new(backing_store: BackingStoreRef) -> Self {
        Self {
            backing_store,
            max_byte_length: None,
        }
    }

    #[inline]
    pub const fn new_resizable(backing_store: BackingStoreRef, max_byte_length: usize) -> Self {
        Self {
            backing_store,
            max_byte_length: Some(max_byte_length),
        }
    }

    #[inline]
    pub const fn backing_store(self) -> BackingStoreRef {
        self.backing_store
    }

    #[inline]
    pub const fn max_byte_length(self) -> Option<usize> {
        self.max_byte_length
    }

    #[inline]
    pub const fn is_resizable(self) -> bool {
        self.max_byte_length.is_some()
    }
}

/// One live `Map` entry carried by runtime-owned ordered storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MapEntry {
    key: Value,
    value: Value,
}

impl MapEntry {
    #[inline]
    pub const fn new(key: Value, value: Value) -> Self {
        Self { key, value }
    }

    #[inline]
    pub const fn key(self) -> Value {
        self.key
    }

    #[inline]
    pub const fn value(self) -> Value {
        self.value
    }

    #[inline]
    pub const fn set_value(&mut self, value: Value) {
        self.value = value;
    }
}

/// Runtime-owned insertion-ordered storage for one `Map` object.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapObjectData {
    entries: Vec<Option<MapEntry>>,
    live_len: usize,
    tombstone_len: usize,
}

impl MapObjectData {
    #[inline]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            live_len: 0,
            tombstone_len: 0,
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.live_len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.live_len == 0
    }

    #[inline]
    pub const fn tombstone_len(&self) -> usize {
        self.tombstone_len
    }

    #[inline]
    pub fn entries(&self) -> &[Option<MapEntry>] {
        &self.entries
    }

    #[inline]
    pub fn entries_mut(&mut self) -> &mut [Option<MapEntry>] {
        &mut self.entries
    }

    #[inline]
    pub fn push(&mut self, entry: MapEntry) {
        self.entries.push(Some(entry));
        self.live_len = self.live_len.saturating_add(1);
    }

    #[inline]
    pub fn delete_index(&mut self, index: usize) -> bool {
        let Some(slot) = self.entries.get_mut(index) else {
            return false;
        };
        if slot.take().is_none() {
            return false;
        }
        self.live_len = self.live_len.saturating_sub(1);
        self.tombstone_len = self.tombstone_len.saturating_add(1);
        true
    }

    #[inline]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.live_len = 0;
        self.tombstone_len = 0;
    }
}

/// Runtime-owned insertion-ordered storage for one `Set` object.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SetObjectData {
    entries: Vec<Option<Value>>,
    live_len: usize,
    tombstone_len: usize,
}

impl SetObjectData {
    #[inline]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            live_len: 0,
            tombstone_len: 0,
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.live_len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.live_len == 0
    }

    #[inline]
    pub const fn tombstone_len(&self) -> usize {
        self.tombstone_len
    }

    #[inline]
    pub fn entries(&self) -> &[Option<Value>] {
        &self.entries
    }

    #[inline]
    pub fn push(&mut self, value: Value) {
        self.entries.push(Some(value));
        self.live_len = self.live_len.saturating_add(1);
    }

    #[inline]
    pub fn delete_index(&mut self, index: usize) -> bool {
        let Some(slot) = self.entries.get_mut(index) else {
            return false;
        };
        if slot.take().is_none() {
            return false;
        }
        self.live_len = self.live_len.saturating_sub(1);
        self.tombstone_len = self.tombstone_len.saturating_add(1);
        true
    }

    #[inline]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.live_len = 0;
        self.tombstone_len = 0;
    }
}

/// Typed element families supported by the current typed-array substrate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypedArrayElementKind {
    BigInt64,
    BigUint64,
    Int8,
    Int16,
    Int32,
    Float16,
    Float32,
    Float64,
    Uint32,
    Uint16,
    Uint8Clamped,
    Uint8,
}

impl TypedArrayElementKind {
    #[inline]
    pub const fn bytes_per_element(self) -> usize {
        match self {
            Self::BigInt64 | Self::BigUint64 | Self::Float64 => 8,
            Self::Int16 | Self::Float16 | Self::Uint16 => 2,
            Self::Int32 | Self::Float32 | Self::Uint32 => 4,
            Self::Int8 | Self::Uint8Clamped | Self::Uint8 => 1,
        }
    }
}

/// Decode a 16-bit IEEE-754 half-precision float into a 64-bit double.
///
/// # Panics
/// Panics if the internal binary16 exponent normalization invariants are violated.
#[inline]
pub fn float16_bits_to_f64(bits: u16) -> f64 {
    let sign = (bits >> 15) & 0x1;
    let exp = i32::from((bits >> 10) & 0x1f);
    let mant = u32::from(bits & 0x03ff);
    let sign_bit_f64 = u64::from(sign) << 63;

    if exp == 0 {
        if mant == 0 {
            return f64::from_bits(sign_bit_f64);
        }
        let mut mantissa = mant;
        let mut shift = 0_i32;
        while (mantissa & 0x0400) == 0 {
            mantissa <<= 1;
            shift += 1;
        }
        mantissa &= 0x03ff;
        let half_unbiased = 1 - 15 - shift;
        let f64_biased =
            u64::try_from(half_unbiased + 1023).expect("binary16 exponent should fit u64");
        let f64_mant = u64::from(mantissa) << (52 - 10);
        return f64::from_bits(sign_bit_f64 | (f64_biased << 52) | f64_mant);
    }

    if exp == 0x1f {
        if mant == 0 {
            return f64::from_bits(sign_bit_f64 | 0x7ff0_0000_0000_0000);
        }
        return f64::from_bits(sign_bit_f64 | 0x7ff8_0000_0000_0000);
    }

    let f64_biased = u64::try_from(exp - 15 + 1023).expect("binary16 exponent should fit u64");
    let f64_mant = u64::from(mant) << (52 - 10);
    f64::from_bits(sign_bit_f64 | (f64_biased << 52) | f64_mant)
}

/// Round an `f64` to IEEE-754 binary16, ties-to-even.
///
/// # Panics
/// Panics if the internal binary64-to-binary16 rounding invariants are violated.
#[inline]
pub fn f64_to_float16_bits(value: f64) -> u16 {
    let bits = value.to_bits();
    let sign16 = u16::from((bits >> 63) != 0);
    let exp64 = i32::try_from((bits >> 52) & 0x07ff).expect("binary64 exponent should fit i32");
    let mant64 = bits & 0x000f_ffff_ffff_ffff;

    if exp64 == 0x07ff {
        if mant64 == 0 {
            return (sign16 << 15) | 0x7c00;
        }
        return (sign16 << 15) | 0x7e00;
    }

    if exp64 == 0 {
        return sign16 << 15;
    }

    let unbiased = exp64 - 1023;
    let f16_unbiased = unbiased + 15;
    let signif = (1_u64 << 52) | mant64;

    if f16_unbiased >= 0x1f {
        return (sign16 << 15) | 0x7c00;
    }

    if f16_unbiased <= 0 {
        if f16_unbiased < -10 {
            return sign16 << 15;
        }
        let shift = (1 - f16_unbiased).cast_unsigned();
        let mant_with_hidden = signif >> (52 - 10 + shift);
        let round_mask = (1_u64 << (52 - 10 + shift)) - 1;
        let round_bits = signif & round_mask;
        let half = 1_u64 << (52 - 10 + shift - 1);
        let increment = round_bits > half || (round_bits == half && (mant_with_hidden & 1) != 0);
        let rounded = mant_with_hidden + u64::from(increment);
        return (sign16 << 15)
            | u16::try_from(rounded).expect("rounded subnormal binary16 mantissa should fit u16");
    }

    let exp16 = u16::try_from(f16_unbiased).expect("binary16 exponent should fit u16");
    let mant = signif >> (52 - 10);
    let round_mask = (1_u64 << (52 - 10)) - 1;
    let round_bits = signif & round_mask;
    let half = 1_u64 << (52 - 10 - 1);
    let increment = round_bits > half || (round_bits == half && (mant & 1) != 0);
    let mut rounded_mant =
        u16::try_from(mant).expect("binary16 mantissa should fit u16") + u16::from(increment);
    let mut rounded_exp = exp16;
    if (rounded_mant & 0x0800) != 0 {
        rounded_mant = 0;
        rounded_exp += 1;
        if rounded_exp >= 0x1f {
            return (sign16 << 15) | 0x7c00;
        }
    }

    (sign16 << 15) | (rounded_exp << 10) | (rounded_mant & 0x03ff)
}

/// Cold payload carried by one `DataView` wrapper object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DataViewObjectData {
    viewed_array_buffer: ObjectRef,
    backing_store: BackingStoreRef,
    byte_offset: usize,
    byte_length: usize,
    length_tracking: bool,
}

impl DataViewObjectData {
    #[inline]
    pub const fn new(
        viewed_array_buffer: ObjectRef,
        backing_store: BackingStoreRef,
        byte_offset: usize,
        byte_length: usize,
    ) -> Self {
        Self {
            viewed_array_buffer,
            backing_store,
            byte_offset,
            byte_length,
            length_tracking: false,
        }
    }

    #[inline]
    pub const fn new_length_tracking(
        viewed_array_buffer: ObjectRef,
        backing_store: BackingStoreRef,
        byte_offset: usize,
    ) -> Self {
        Self {
            viewed_array_buffer,
            backing_store,
            byte_offset,
            byte_length: 0,
            length_tracking: true,
        }
    }

    #[inline]
    pub const fn viewed_array_buffer(self) -> ObjectRef {
        self.viewed_array_buffer
    }

    #[inline]
    pub const fn backing_store(self) -> BackingStoreRef {
        self.backing_store
    }

    #[inline]
    pub const fn byte_offset(self) -> usize {
        self.byte_offset
    }

    #[inline]
    pub const fn byte_length(self) -> usize {
        self.byte_length
    }

    #[inline]
    pub const fn is_length_tracking(self) -> bool {
        self.length_tracking
    }
}

/// Cold payload carried by one typed-array wrapper object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypedArrayObjectData {
    viewed_array_buffer: ObjectRef,
    backing_store: BackingStoreRef,
    byte_offset: usize,
    length: usize,
    kind: TypedArrayElementKind,
    length_tracking: bool,
}

impl TypedArrayObjectData {
    #[inline]
    pub const fn new(
        viewed_array_buffer: ObjectRef,
        backing_store: BackingStoreRef,
        byte_offset: usize,
        length: usize,
        kind: TypedArrayElementKind,
    ) -> Self {
        Self {
            viewed_array_buffer,
            backing_store,
            byte_offset,
            length,
            kind,
            length_tracking: false,
        }
    }

    #[inline]
    pub const fn new_length_tracking(
        viewed_array_buffer: ObjectRef,
        backing_store: BackingStoreRef,
        byte_offset: usize,
        length: usize,
        kind: TypedArrayElementKind,
    ) -> Self {
        Self {
            viewed_array_buffer,
            backing_store,
            byte_offset,
            length,
            kind,
            length_tracking: true,
        }
    }

    #[inline]
    pub const fn viewed_array_buffer(self) -> ObjectRef {
        self.viewed_array_buffer
    }

    #[inline]
    pub const fn backing_store(self) -> BackingStoreRef {
        self.backing_store
    }

    #[inline]
    pub const fn byte_offset(self) -> usize {
        self.byte_offset
    }

    #[inline]
    pub const fn length(self) -> usize {
        self.length
    }

    #[inline]
    pub const fn is_length_tracking(self) -> bool {
        self.length_tracking
    }

    #[inline]
    pub const fn with_length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    #[inline]
    pub const fn kind(self) -> TypedArrayElementKind {
        self.kind
    }

    #[inline]
    pub const fn byte_length(self) -> usize {
        self.length.saturating_mul(self.kind.bytes_per_element())
    }
}

/// Cold payload carried by one proxy exotic object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProxyObjectData {
    target: ObjectRef,
    handler: Option<ObjectRef>,
    revoked: bool,
    callable: bool,
    constructible: bool,
}

impl ProxyObjectData {
    #[inline]
    pub const fn new(
        target: ObjectRef,
        handler: ObjectRef,
        callable: bool,
        constructible: bool,
    ) -> Self {
        Self {
            target,
            handler: Some(handler),
            revoked: false,
            callable,
            constructible,
        }
    }

    #[inline]
    pub const fn target(self) -> ObjectRef {
        self.target
    }

    #[inline]
    pub const fn handler(self) -> Option<ObjectRef> {
        self.handler
    }

    #[inline]
    pub const fn revoked(self) -> bool {
        self.revoked
    }

    #[inline]
    pub const fn callable(self) -> bool {
        self.callable
    }

    #[inline]
    pub const fn constructible(self) -> bool {
        self.constructible
    }

    #[inline]
    pub const fn with_handler(mut self, handler: Option<ObjectRef>) -> Self {
        self.handler = handler;
        self
    }

    #[inline]
    pub const fn with_revoked(mut self, revoked: bool) -> Self {
        self.revoked = revoked;
        self
    }
}

/// Cold payload carried by ordinary objects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OrdinaryObjectData {
    #[default]
    Plain,
    PrimitiveWrapper(PrimitiveWrapperKind),
    Map,
    Set,
    WeakMap,
    WeakSet,
    WeakRef,
    FinalizationRegistry,
    ArrayBuffer,
    SharedArrayBuffer,
    DataView,
    TypedArray(TypedArrayElementKind),
    Date(Value),
    Temporal(TemporalObjectKind),
    JsonRaw,
    RegExp,
    Generator,
    IteratorHelper,
    ArrayIterator,
    MapIterator,
    SetIterator,
    StringIterator,
    RegExpStringIterator,
}

impl OrdinaryObjectData {
    #[inline]
    pub const fn wrapper_kind(self) -> Option<PrimitiveWrapperKind> {
        match self {
            Self::PrimitiveWrapper(kind) => Some(kind),
            _ => None,
        }
    }

    #[inline]
    pub const fn is_map(self) -> bool {
        matches!(self, Self::Map)
    }

    #[inline]
    pub const fn is_set(self) -> bool {
        matches!(self, Self::Set)
    }

    #[inline]
    pub const fn is_weak_map(self) -> bool {
        matches!(self, Self::WeakMap)
    }

    #[inline]
    pub const fn is_weak_set(self) -> bool {
        matches!(self, Self::WeakSet)
    }

    #[inline]
    pub const fn is_weak_ref(self) -> bool {
        matches!(self, Self::WeakRef)
    }

    #[inline]
    pub const fn is_finalization_registry(self) -> bool {
        matches!(self, Self::FinalizationRegistry)
    }

    #[inline]
    pub const fn is_array_buffer(self) -> bool {
        matches!(self, Self::ArrayBuffer)
    }

    #[inline]
    pub const fn is_shared_array_buffer(self) -> bool {
        matches!(self, Self::SharedArrayBuffer)
    }

    #[inline]
    pub const fn is_array_buffer_family(self) -> bool {
        matches!(self, Self::ArrayBuffer | Self::SharedArrayBuffer)
    }

    #[inline]
    pub const fn is_data_view(self) -> bool {
        matches!(self, Self::DataView)
    }

    #[inline]
    pub const fn typed_array_kind(self) -> Option<TypedArrayElementKind> {
        match self {
            Self::TypedArray(kind) => Some(kind),
            _ => None,
        }
    }

    #[inline]
    pub const fn is_date(self) -> bool {
        matches!(self, Self::Date(_))
    }

    #[inline]
    pub const fn date_value(self) -> Option<Value> {
        match self {
            Self::Date(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub const fn temporal_kind(self) -> Option<TemporalObjectKind> {
        match self {
            Self::Temporal(kind) => Some(kind),
            _ => None,
        }
    }

    #[inline]
    pub const fn is_json_raw(self) -> bool {
        matches!(self, Self::JsonRaw)
    }

    #[inline]
    pub const fn is_regexp(self) -> bool {
        matches!(self, Self::RegExp)
    }

    #[inline]
    pub const fn is_generator(self) -> bool {
        matches!(self, Self::Generator)
    }
}

/// Function `this` binding strategy frozen by the Phase 3 substrate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum FunctionThisMode {
    Lexical,
    #[default]
    Strict,
    Global,
}

/// Constructor capability flags frozen separately from function-kind flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct FunctionConstructorFlags(u8);

impl FunctionConstructorFlags {
    const CONSTRUCTIBLE: u8 = 1 << 0;

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn constructible() -> Self {
        Self(Self::CONSTRUCTIBLE)
    }

    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    #[inline]
    pub const fn with_constructible(mut self, enabled: bool) -> Self {
        if enabled {
            self.0 |= Self::CONSTRUCTIBLE;
        } else {
            self.0 &= !Self::CONSTRUCTIBLE;
        }
        self
    }

    #[inline]
    pub const fn is_constructible(self) -> bool {
        self.contains(Self::constructible())
    }
}

/// Function-shape flags that describe semantic categories for one callable object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct FunctionKindFlags(u8);

impl FunctionKindFlags {
    pub const ARROW: Self = Self(1 << 0);
    pub const CLASS_CONSTRUCTOR: Self = Self(1 << 1);
    pub const GENERATOR: Self = Self(1 << 2);
    pub const ASYNC: Self = Self(1 << 3);
    pub const ASYNC_GENERATOR: Self = Self(1 << 4);

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    #[inline]
    pub const fn is_arrow(self) -> bool {
        self.contains(Self::ARROW)
    }

    #[inline]
    pub const fn is_class_constructor(self) -> bool {
        self.contains(Self::CLASS_CONSTRUCTOR)
    }

    #[inline]
    pub const fn is_generator(self) -> bool {
        self.contains(Self::GENERATOR)
    }

    #[inline]
    pub const fn is_async(self) -> bool {
        self.contains(Self::ASYNC)
    }

    #[inline]
    pub const fn is_async_generator(self) -> bool {
        self.contains(Self::ASYNC_GENERATOR)
    }
}

/// Stable generator execution state tracked alongside one generator object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GeneratorState {
    SuspendedStart,
    SuspendedYield,
    Executing,
    Completed,
}

/// Stable callable-entry identity shared by native and later bytecode functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FunctionEntryIdentity {
    Native(NativeFunctionId),
    Bytecode(CodeRef),
    Bound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundFunctionInit {
    target: ObjectRef,
    this_value: Value,
    arguments: Box<[Value]>,
}

impl BoundFunctionInit {
    #[inline]
    pub(crate) const fn new(target: ObjectRef, this_value: Value, arguments: Box<[Value]>) -> Self {
        Self {
            target,
            this_value,
            arguments,
        }
    }

    #[inline]
    pub(crate) const fn target(&self) -> ObjectRef {
        self.target
    }

    #[inline]
    pub(crate) const fn this_value(&self) -> Value {
        self.this_value
    }

    #[inline]
    pub(crate) fn arguments(&self) -> &[Value] {
        &self.arguments
    }
}

/// Frozen function payload metadata stored out of line from the object header.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FunctionObjectData {
    realm: Option<RealmRef>,
    environment: Option<EnvironmentRef>,
    private_env: Option<EnvironmentRef>,
    this_mode: FunctionThisMode,
    home_object: Option<ObjectRef>,
    constructor_flags: FunctionConstructorFlags,
    has_prototype_property: bool,
    kind_flags: FunctionKindFlags,
    entry: Option<FunctionEntryIdentity>,
    bound_init: Option<BoundFunctionInit>,
    gc_payload: Option<FunctionPayloadRef>,
}

impl FunctionObjectData {
    #[inline]
    pub const fn native(
        realm: RealmRef,
        environment: EnvironmentRef,
        entry: BuiltinFunctionId,
    ) -> Self {
        Self {
            realm: Some(realm),
            environment: Some(environment),
            private_env: None,
            this_mode: FunctionThisMode::Strict,
            home_object: None,
            constructor_flags: FunctionConstructorFlags::empty(),
            has_prototype_property: false,
            kind_flags: FunctionKindFlags::empty(),
            entry: Some(FunctionEntryIdentity::Native(NativeFunctionId::builtin(
                entry,
            ))),
            bound_init: None,
            gc_payload: None,
        }
    }

    #[inline]
    pub const fn embedding(
        realm: RealmRef,
        environment: EnvironmentRef,
        entry: EmbeddingFunctionId,
    ) -> Self {
        Self {
            realm: Some(realm),
            environment: Some(environment),
            private_env: None,
            this_mode: FunctionThisMode::Strict,
            home_object: None,
            constructor_flags: FunctionConstructorFlags::empty(),
            has_prototype_property: false,
            kind_flags: FunctionKindFlags::empty(),
            entry: Some(FunctionEntryIdentity::Native(NativeFunctionId::embedding(
                entry,
            ))),
            bound_init: None,
            gc_payload: None,
        }
    }

    #[inline]
    pub const fn bytecode(realm: RealmRef, environment: EnvironmentRef, code: CodeRef) -> Self {
        Self {
            realm: Some(realm),
            environment: Some(environment),
            private_env: None,
            this_mode: FunctionThisMode::Strict,
            home_object: None,
            constructor_flags: FunctionConstructorFlags::empty(),
            has_prototype_property: false,
            kind_flags: FunctionKindFlags::empty(),
            entry: Some(FunctionEntryIdentity::Bytecode(code)),
            bound_init: None,
            gc_payload: None,
        }
    }

    #[inline]
    pub const fn bound(
        realm: RealmRef,
        environment: EnvironmentRef,
        target: ObjectRef,
        this_value: Value,
        arguments: Box<[Value]>,
    ) -> Self {
        Self {
            realm: Some(realm),
            environment: Some(environment),
            private_env: None,
            this_mode: FunctionThisMode::Strict,
            home_object: None,
            constructor_flags: FunctionConstructorFlags::empty(),
            has_prototype_property: false,
            kind_flags: FunctionKindFlags::empty(),
            entry: Some(FunctionEntryIdentity::Bound),
            bound_init: Some(BoundFunctionInit::new(target, this_value, arguments)),
            gc_payload: None,
        }
    }

    #[inline]
    pub const fn realm(&self) -> Option<RealmRef> {
        self.realm
    }

    #[inline]
    pub const fn environment(&self) -> Option<EnvironmentRef> {
        self.environment
    }

    #[inline]
    pub const fn private_env(&self) -> Option<EnvironmentRef> {
        self.private_env
    }

    #[inline]
    pub const fn this_mode(&self) -> FunctionThisMode {
        self.this_mode
    }

    #[inline]
    pub const fn home_object(&self) -> Option<ObjectRef> {
        self.home_object
    }

    #[inline]
    pub const fn constructor_flags(&self) -> FunctionConstructorFlags {
        self.constructor_flags
    }

    #[inline]
    pub const fn has_prototype_property(&self) -> bool {
        self.has_prototype_property
    }

    #[inline]
    pub const fn kind_flags(&self) -> FunctionKindFlags {
        self.kind_flags
    }

    #[inline]
    pub const fn entry(&self) -> Option<FunctionEntryIdentity> {
        self.entry
    }

    #[inline]
    pub(crate) const fn bound_init(&self) -> Option<&BoundFunctionInit> {
        self.bound_init.as_ref()
    }

    #[inline]
    pub const fn gc_payload(&self) -> Option<FunctionPayloadRef> {
        self.gc_payload
    }

    #[inline]
    pub const fn is_constructible(&self) -> bool {
        self.constructor_flags.is_constructible()
    }

    #[inline]
    pub const fn with_this_mode(mut self, this_mode: FunctionThisMode) -> Self {
        self.this_mode = this_mode;
        self
    }

    #[inline]
    pub const fn with_environment(mut self, environment: Option<EnvironmentRef>) -> Self {
        self.environment = environment;
        self
    }

    #[inline]
    pub const fn with_private_env(mut self, private_env: Option<EnvironmentRef>) -> Self {
        self.private_env = private_env;
        self
    }

    #[inline]
    pub const fn with_home_object(mut self, home_object: Option<ObjectRef>) -> Self {
        self.home_object = home_object;
        self
    }

    #[inline]
    pub const fn with_constructor_flags(
        mut self,
        constructor_flags: FunctionConstructorFlags,
    ) -> Self {
        self.constructor_flags = constructor_flags;
        self
    }

    #[inline]
    pub const fn with_has_prototype_property(mut self, has_prototype_property: bool) -> Self {
        self.has_prototype_property = has_prototype_property;
        self
    }

    #[inline]
    pub const fn with_constructible(mut self, constructible: bool) -> Self {
        self.constructor_flags = self.constructor_flags.with_constructible(constructible);
        self
    }

    #[inline]
    pub const fn with_kind_flags(mut self, kind_flags: FunctionKindFlags) -> Self {
        self.kind_flags = kind_flags;
        self
    }

    pub(crate) const fn runtime_record(&self) -> Option<RuntimeFunctionRecord> {
        if self.realm.is_none()
            && self.environment.is_none()
            && self.private_env.is_none()
            && self.home_object.is_none()
            && !matches!(
                self.entry,
                Some(FunctionEntryIdentity::Bytecode(_) | FunctionEntryIdentity::Bound)
            )
        {
            return None;
        }

        let mut record = RuntimeFunctionRecord::new(
            self.realm,
            self.environment,
            self.private_env,
            self.home_object,
            match self.entry {
                Some(FunctionEntryIdentity::Bytecode(code)) => Some(code),
                _ => None,
            },
        );
        if let Some(bound) = &self.bound_init {
            record = record.with_bound(Some(RuntimeBoundFunctionRecord::new(
                bound.target(),
                bound.this_value(),
                None,
            )));
        }
        Some(record)
    }

    pub(crate) const fn with_gc_payload(mut self, payload: Option<FunctionPayloadRef>) -> Self {
        self.gc_payload = payload;
        self
    }

    pub(crate) fn without_bound_init(mut self) -> Self {
        self.bound_init = None;
        self
    }
}

/// Request surface passed to injected native-call registries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeCallRequest<'a> {
    callee: ObjectRef,
    this_value: Value,
    arguments: &'a [Value],
    realm: RealmRef,
    environment: EnvironmentRef,
    private_env: Option<EnvironmentRef>,
    this_mode: FunctionThisMode,
    home_object: Option<ObjectRef>,
    constructor_flags: FunctionConstructorFlags,
    kind_flags: FunctionKindFlags,
    entry: NativeFunctionId,
}

impl<'a> NativeCallRequest<'a> {
    #[inline]
    #[allow(
        clippy::too_many_arguments,
        reason = "native call requests are field-for-field dispatch records assembled at one call site"
    )]
    pub(crate) const fn new(
        callee: ObjectRef,
        this_value: Value,
        arguments: &'a [Value],
        realm: RealmRef,
        environment: EnvironmentRef,
        private_env: Option<EnvironmentRef>,
        this_mode: FunctionThisMode,
        home_object: Option<ObjectRef>,
        constructor_flags: FunctionConstructorFlags,
        kind_flags: FunctionKindFlags,
        entry: NativeFunctionId,
    ) -> Self {
        Self {
            callee,
            this_value,
            arguments,
            realm,
            environment,
            private_env,
            this_mode,
            home_object,
            constructor_flags,
            kind_flags,
            entry,
        }
    }

    #[inline]
    pub const fn callee(self) -> ObjectRef {
        self.callee
    }

    #[inline]
    pub const fn this_value(self) -> Value {
        self.this_value
    }

    #[inline]
    pub const fn arguments(self) -> &'a [Value] {
        self.arguments
    }

    #[inline]
    pub const fn realm(self) -> RealmRef {
        self.realm
    }

    #[inline]
    pub const fn environment(self) -> EnvironmentRef {
        self.environment
    }

    #[inline]
    pub const fn private_env(self) -> Option<EnvironmentRef> {
        self.private_env
    }

    #[inline]
    pub const fn this_mode(self) -> FunctionThisMode {
        self.this_mode
    }

    #[inline]
    pub const fn home_object(self) -> Option<ObjectRef> {
        self.home_object
    }

    #[inline]
    pub const fn constructor_flags(self) -> FunctionConstructorFlags {
        self.constructor_flags
    }

    #[inline]
    pub const fn kind_flags(self) -> FunctionKindFlags {
        self.kind_flags
    }

    #[inline]
    pub const fn entry(self) -> NativeFunctionId {
        self.entry
    }
}

/// Request surface passed to injected native-construct registries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeConstructRequest<'a> {
    callee: ObjectRef,
    new_target: ObjectRef,
    arguments: &'a [Value],
    realm: RealmRef,
    environment: EnvironmentRef,
    private_env: Option<EnvironmentRef>,
    this_mode: FunctionThisMode,
    home_object: Option<ObjectRef>,
    constructor_flags: FunctionConstructorFlags,
    kind_flags: FunctionKindFlags,
    entry: NativeFunctionId,
}

impl<'a> NativeConstructRequest<'a> {
    #[inline]
    #[allow(
        clippy::too_many_arguments,
        reason = "native construct requests are field-for-field dispatch records assembled at one call site"
    )]
    pub(crate) const fn new(
        callee: ObjectRef,
        new_target: ObjectRef,
        arguments: &'a [Value],
        realm: RealmRef,
        environment: EnvironmentRef,
        private_env: Option<EnvironmentRef>,
        this_mode: FunctionThisMode,
        home_object: Option<ObjectRef>,
        constructor_flags: FunctionConstructorFlags,
        kind_flags: FunctionKindFlags,
        entry: NativeFunctionId,
    ) -> Self {
        Self {
            callee,
            new_target,
            arguments,
            realm,
            environment,
            private_env,
            this_mode,
            home_object,
            constructor_flags,
            kind_flags,
            entry,
        }
    }

    #[inline]
    pub const fn callee(self) -> ObjectRef {
        self.callee
    }

    #[inline]
    pub const fn new_target(self) -> ObjectRef {
        self.new_target
    }

    #[inline]
    pub const fn arguments(self) -> &'a [Value] {
        self.arguments
    }

    #[inline]
    pub const fn realm(self) -> RealmRef {
        self.realm
    }

    #[inline]
    pub const fn environment(self) -> EnvironmentRef {
        self.environment
    }

    #[inline]
    pub const fn private_env(self) -> Option<EnvironmentRef> {
        self.private_env
    }

    #[inline]
    pub const fn this_mode(self) -> FunctionThisMode {
        self.this_mode
    }

    #[inline]
    pub const fn home_object(self) -> Option<ObjectRef> {
        self.home_object
    }

    #[inline]
    pub const fn constructor_flags(self) -> FunctionConstructorFlags {
        self.constructor_flags
    }

    #[inline]
    pub const fn kind_flags(self) -> FunctionKindFlags {
        self.kind_flags
    }

    #[inline]
    pub const fn entry(self) -> NativeFunctionId {
        self.entry
    }
}

/// Injected substrate-native call/construct registry used by Phase 3 tests and harnesses.
pub trait NativeFunctionRegistry {
    /// Dispatches one native call.
    ///
    /// # Errors
    /// Returns an error when the registry cannot complete the requested dispatch.
    fn call(
        &mut self,
        runtime: &mut ObjectRuntime,
        heap: &mut PrimitiveMutator<'_>,
        request: NativeCallRequest<'_>,
    ) -> InternalMethodResult<Value>;

    /// Dispatches one native construction call.
    ///
    /// # Errors
    /// Returns an error when the registry cannot complete the requested construction.
    fn construct(
        &mut self,
        runtime: &mut ObjectRuntime,
        heap: &mut PrimitiveMutator<'_>,
        request: NativeConstructRequest<'_>,
    ) -> InternalMethodResult<ObjectRef>;
}

/// Kind-specific metadata stored out of line from the object hot header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectColdData {
    Ordinary(OrdinaryObjectData),
    Function(FunctionObjectData),
    Proxy(ProxyObjectData),
}

impl Default for ObjectColdData {
    fn default() -> Self {
        Self::Ordinary(OrdinaryObjectData::Plain)
    }
}

impl ObjectColdData {
    #[inline]
    pub const fn kind(&self) -> ObjectKind {
        match self {
            Self::Ordinary(_) => ObjectKind::Ordinary,
            Self::Function(_) => ObjectKind::Function,
            Self::Proxy(_) => ObjectKind::Proxy,
        }
    }
}
