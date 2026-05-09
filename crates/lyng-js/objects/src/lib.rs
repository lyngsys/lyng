//! Object-model substrate for the lyng-js runtime layer.
//!
//! Ownership: `lyng_js_objects` owns object allocation, canonical shapes,
//! named-property metadata, indexed-element and named-slot storage references,
//! cold payload metadata, and later internal-method dispatch.

#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    reason = "object-model APIs expose domain-specific records and lightweight descriptor accessors across crates"
)]

use lyng_js_gc::{
    AllocationLifetime, FunctionPayloadRef, ObjectHandleStoreTarget, ObjectSlotsHandleStoreTarget,
    PrimitiveHeapView, PrimitiveMutator, RuntimeFunctionRecord, RuntimeObjectRecord,
    RuntimeShapeRecord, ShapeHandleStoreTarget, ValueStoreTarget,
};
use lyng_js_types::{
    BackingStoreRef, BuiltinFunctionId, CodeRef, DescriptorAttributes, EmbeddingFunctionId,
    EnvironmentRef, NativeFunctionId, ObjectRef, PropertyDescriptor, PropertyKey, RealmRef,
    ShapeId, Value,
};

mod core;
mod descriptors;
mod function_dispatch;
mod functions;
mod internal_methods;
mod module_namespace;
mod object_metadata;
mod object_records;
mod private_storage;
mod regexp;
mod runtime;
mod runtime_storage;
mod shapes;
mod temporal;

pub use self::core::{
    ElementStorageRef, InternalMethodError, InternalMethodResult, NamedSlotStorageRef, ObjectFlags,
    ObjectKind,
};
use self::descriptors::{
    complete_descriptor_update, dense_element_growth_capacity, descriptor_from_payload,
    descriptor_kind, descriptor_same_value, flattened_property_lookup, ordinary_property_attrs,
    payload_from_complete_descriptor, resolve_get_from_descriptor, trim_dense_logical_len,
    update_integrity_flags, validate_descriptor_change, write_named_payload, DescriptorKind,
};
pub use self::functions::{
    f64_to_float16_bits, float16_bits_to_f64, ArrayBufferObjectData, DataViewObjectData,
    FunctionConstructorFlags, FunctionEntryIdentity, FunctionKindFlags, FunctionObjectData,
    FunctionThisMode, GeneratorState, MapEntry, MapObjectData, NativeCallRequest,
    NativeConstructRequest, NativeFunctionRegistry, ObjectColdData, OrdinaryObjectData,
    PrimitiveWrapperKind, ProxyObjectData, SetObjectData, TypedArrayElementKind,
    TypedArrayObjectData,
};
use self::module_namespace::ModuleNamespaceObject;
pub use self::module_namespace::{ModuleNamespaceExport, ModuleNamespaceExportTarget};
pub use self::object_metadata::{ClassPrivateElementKind, PrivateDescriptorSummary};
use self::object_metadata::{
    ClassRecord, ElementStorageMetadata, InstalledPrivateBrand, NamedPropertyDictionary,
    NamedPropertyStorage, ObjectMetadata, RootShapeKey, ShapeMetadata,
};
pub use self::object_records::{ObjectAllocation, ObjectHeader, ObjectRecord};
pub use self::regexp::{
    RegExpMatchRecord, RegExpNamedCapture, RegExpObjectFlags, RegExpPayload,
    RegExpPayloadAccounting,
};
pub(crate) use self::runtime::MIN_DENSE_ELEMENT_CAPACITY;
pub use self::runtime::{
    ObjectRuntime, DENSE_ELEMENT_SPARSE_GAP_THRESHOLD, NAMED_PROPERTY_CHURN_DICTIONARY_THRESHOLD,
    SMALL_SHAPE_INLINE_PROPERTY_LIMIT,
};
pub use self::shapes::{
    ElementMode, InvalidationCause, InvalidationEvent, NamedPropertyCacheEntry,
    NamedPropertyCachePath, NamedPropertyCachePurpose, NamedPropertyDictionaryEntry,
    NamedPropertyStorageMode, NamedPropertyValue, PropertyCacheDependency, ShapeAllocation,
    ShapeProperty, ShapePropertyKind, ShapeRecord, ShapeTransitionKey, SparseElementEntry,
    PROPERTY_CACHE_MAX_DEPENDENCIES,
};
pub use self::temporal::{
    TemporalDurationObjectData, TemporalInstantObjectData, TemporalObjectData, TemporalObjectKind,
    TemporalPlainDateObjectData, TemporalPlainDateTimeObjectData, TemporalPlainMonthDayObjectData,
    TemporalPlainTimeObjectData, TemporalPlainYearMonthObjectData, TemporalZonedDateTimeObjectData,
};

#[cfg(test)]
mod tests;
