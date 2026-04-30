use lyng_js_common::{AtomId, AtomTable, WellKnownAtom};
use lyng_js_gc::{AtomGcSweep, PrimitiveTracer, TraceAtomEdges, TraceHeapEdges};
use lyng_js_types::{SymbolRef, WellKnownSymbolId};

/// Agent-owned atom set used by the Phase 5 default-realm bootstrap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BootstrapAtoms {
    boolean: AtomId,
    create: AtomId,
    create_realm: AtomId,
    define_property: AtomId,
    error: AtomId,
    eval_script: AtomId,
    eval_error: AtomId,
    date: AtomId,
    array_buffer: AtomId,
    shared_array_buffer: AtomId,
    data_view: AtomId,
    typed_array: AtomId,
    atomics: AtomId,
    decode_uri: AtomId,
    decode_uri_component: AtomId,
    freeze: AtomId,
    function: AtomId,
    array: AtomId,
    get_own_property_descriptor: AtomId,
    get_prototype_of: AtomId,
    global: AtomId,
    global_this: AtomId,
    bigint: AtomId,
    encode_uri: AtomId,
    encode_uri_component: AtomId,
    escape: AtomId,
    flags: AtomId,
    has_indices: AtomId,
    has_own_property: AtomId,
    infinity: AtomId,
    is_finite: AtomId,
    is_nan: AtomId,
    is_extensible: AtomId,
    is_frozen: AtomId,
    is_prototype_of: AtomId,
    is_sealed: AtomId,
    last_index: AtomId,
    message: AtomId,
    math: AtomId,
    nan: AtomId,
    number: AtomId,
    object: AtomId,
    map: AtomId,
    set: AtomId,
    weak_map: AtomId,
    weak_set: AtomId,
    weak_ref: AtomId,
    finalization_registry: AtomId,
    json: AtomId,
    int8_array: AtomId,
    int16_array: AtomId,
    int32_array: AtomId,
    float32_array: AtomId,
    float64_array: AtomId,
    big_int64_array: AtomId,
    big_uint64_array: AtomId,
    uint32_array: AtomId,
    uint16_array: AtomId,
    uint8_clamped_array: AtomId,
    uint8_array: AtomId,
    promise: AtomId,
    aggregate_error: AtomId,
    parse_float: AtomId,
    parse_int: AtomId,
    prevent_extensions: AtomId,
    property_is_enumerable: AtomId,
    range_error: AtomId,
    reference_error: AtomId,
    regexp: AtomId,
    seal: AtomId,
    set_prototype_of: AtomId,
    source: AtomId,
    string: AtomId,
    syntax_error: AtomId,
    symbol: AtomId,
    type_error: AtomId,
    uri_error: AtomId,
    unescape: AtomId,
    key_for: AtomId,
    undefined: AtomId,
    has_instance: AtomId,
    is_concat_spreadable: AtomId,
    iterator: AtomId,
    async_iterator: AtomId,
    species: AtomId,
    to_primitive: AtomId,
    to_string_tag: AtomId,
    unscopables: AtomId,
    dispose: AtomId,
    async_dispose: AtomId,
    symbol_has_instance: AtomId,
    symbol_is_concat_spreadable: AtomId,
    symbol_iterator: AtomId,
    symbol_async_iterator: AtomId,
    match_: AtomId,
    match_all: AtomId,
    replace: AtomId,
    search: AtomId,
    symbol_species: AtomId,
    split: AtomId,
    symbol_to_primitive: AtomId,
    symbol_to_string_tag: AtomId,
    symbol_unscopables: AtomId,
    symbol_dispose: AtomId,
    symbol_async_dispose: AtomId,
    symbol_match: AtomId,
    symbol_match_all: AtomId,
    symbol_replace: AtomId,
    symbol_search: AtomId,
    symbol_split: AtomId,
}

impl BootstrapAtoms {
    pub fn new(atoms: &mut AtomTable) -> Self {
        Self {
            boolean: atoms.intern("Boolean"),
            create: atoms.intern("create"),
            create_realm: atoms.intern("createRealm"),
            define_property: atoms.intern("defineProperty"),
            error: atoms.intern("Error"),
            eval_script: atoms.intern("evalScript"),
            eval_error: atoms.intern("EvalError"),
            date: atoms.intern("Date"),
            array_buffer: atoms.intern("ArrayBuffer"),
            shared_array_buffer: atoms.intern("SharedArrayBuffer"),
            data_view: atoms.intern("DataView"),
            typed_array: atoms.intern("TypedArray"),
            atomics: atoms.intern("Atomics"),
            decode_uri: atoms.intern("decodeURI"),
            decode_uri_component: atoms.intern("decodeURIComponent"),
            freeze: atoms.intern("freeze"),
            function: atoms.intern("Function"),
            array: atoms.intern("Array"),
            get_own_property_descriptor: atoms.intern("getOwnPropertyDescriptor"),
            get_prototype_of: atoms.intern("getPrototypeOf"),
            global: atoms.intern("global"),
            global_this: atoms.intern("globalThis"),
            bigint: atoms.intern("BigInt"),
            encode_uri: atoms.intern("encodeURI"),
            encode_uri_component: atoms.intern("encodeURIComponent"),
            escape: atoms.intern("escape"),
            flags: atoms.intern("flags"),
            has_indices: atoms.intern("hasIndices"),
            has_own_property: atoms.intern("hasOwnProperty"),
            infinity: atoms.intern("Infinity"),
            is_finite: atoms.intern("isFinite"),
            is_nan: atoms.intern("isNaN"),
            is_extensible: atoms.intern("isExtensible"),
            is_frozen: atoms.intern("isFrozen"),
            is_prototype_of: atoms.intern("isPrototypeOf"),
            is_sealed: atoms.intern("isSealed"),
            last_index: atoms.intern("lastIndex"),
            message: atoms.intern("message"),
            math: atoms.intern("Math"),
            nan: atoms.intern("NaN"),
            number: atoms.intern("Number"),
            object: atoms.intern("Object"),
            map: atoms.intern("Map"),
            set: atoms.intern("Set"),
            weak_map: atoms.intern("WeakMap"),
            weak_set: atoms.intern("WeakSet"),
            weak_ref: atoms.intern("WeakRef"),
            finalization_registry: atoms.intern("FinalizationRegistry"),
            json: atoms.intern("JSON"),
            int8_array: atoms.intern("Int8Array"),
            int16_array: atoms.intern("Int16Array"),
            int32_array: atoms.intern("Int32Array"),
            float32_array: atoms.intern("Float32Array"),
            float64_array: atoms.intern("Float64Array"),
            big_int64_array: atoms.intern("BigInt64Array"),
            big_uint64_array: atoms.intern("BigUint64Array"),
            uint32_array: atoms.intern("Uint32Array"),
            uint16_array: atoms.intern("Uint16Array"),
            uint8_clamped_array: atoms.intern("Uint8ClampedArray"),
            uint8_array: atoms.intern("Uint8Array"),
            promise: atoms.intern("Promise"),
            aggregate_error: atoms.intern("AggregateError"),
            parse_float: atoms.intern("parseFloat"),
            parse_int: atoms.intern("parseInt"),
            prevent_extensions: atoms.intern("preventExtensions"),
            property_is_enumerable: atoms.intern("propertyIsEnumerable"),
            range_error: atoms.intern("RangeError"),
            reference_error: atoms.intern("ReferenceError"),
            regexp: atoms.intern("RegExp"),
            seal: atoms.intern("seal"),
            set_prototype_of: atoms.intern("setPrototypeOf"),
            source: atoms.intern("source"),
            string: atoms.intern("String"),
            syntax_error: atoms.intern("SyntaxError"),
            symbol: atoms.intern("Symbol"),
            type_error: atoms.intern("TypeError"),
            uri_error: atoms.intern("URIError"),
            unescape: atoms.intern("unescape"),
            key_for: atoms.intern("keyFor"),
            undefined: WellKnownAtom::undefined.id(),
            has_instance: atoms.intern("hasInstance"),
            is_concat_spreadable: atoms.intern("isConcatSpreadable"),
            iterator: atoms.intern("iterator"),
            async_iterator: atoms.intern("asyncIterator"),
            species: atoms.intern("species"),
            to_primitive: atoms.intern("toPrimitive"),
            to_string_tag: atoms.intern("toStringTag"),
            unscopables: atoms.intern("unscopables"),
            dispose: atoms.intern("dispose"),
            async_dispose: atoms.intern("asyncDispose"),
            symbol_has_instance: atoms.intern(WellKnownSymbolId::HasInstance.description()),
            symbol_is_concat_spreadable: atoms
                .intern(WellKnownSymbolId::IsConcatSpreadable.description()),
            symbol_iterator: atoms.intern(WellKnownSymbolId::Iterator.description()),
            symbol_async_iterator: atoms.intern(WellKnownSymbolId::AsyncIterator.description()),
            match_: atoms.intern("match"),
            match_all: atoms.intern("matchAll"),
            replace: atoms.intern("replace"),
            search: atoms.intern("search"),
            symbol_species: atoms.intern(WellKnownSymbolId::Species.description()),
            split: atoms.intern("split"),
            symbol_to_primitive: atoms.intern(WellKnownSymbolId::ToPrimitive.description()),
            symbol_to_string_tag: atoms.intern(WellKnownSymbolId::ToStringTag.description()),
            symbol_unscopables: atoms.intern(WellKnownSymbolId::Unscopables.description()),
            symbol_dispose: atoms.intern(WellKnownSymbolId::Dispose.description()),
            symbol_async_dispose: atoms.intern(WellKnownSymbolId::AsyncDispose.description()),
            symbol_match: atoms.intern(WellKnownSymbolId::Match.description()),
            symbol_match_all: atoms.intern(WellKnownSymbolId::MatchAll.description()),
            symbol_replace: atoms.intern(WellKnownSymbolId::Replace.description()),
            symbol_search: atoms.intern(WellKnownSymbolId::Search.description()),
            symbol_split: atoms.intern(WellKnownSymbolId::Split.description()),
        }
    }

    #[inline]
    pub const fn boolean(self) -> AtomId {
        self.boolean
    }

    #[inline]
    pub const fn create(self) -> AtomId {
        self.create
    }

    #[inline]
    pub const fn create_realm(self) -> AtomId {
        self.create_realm
    }

    #[inline]
    pub const fn define_property(self) -> AtomId {
        self.define_property
    }

    #[inline]
    pub const fn error(self) -> AtomId {
        self.error
    }

    #[inline]
    pub const fn eval_script(self) -> AtomId {
        self.eval_script
    }

    #[inline]
    pub const fn eval_error(self) -> AtomId {
        self.eval_error
    }

    #[inline]
    pub const fn date(self) -> AtomId {
        self.date
    }

    #[inline]
    pub const fn array_buffer(self) -> AtomId {
        self.array_buffer
    }

    #[inline]
    pub const fn shared_array_buffer(self) -> AtomId {
        self.shared_array_buffer
    }

    #[inline]
    pub const fn data_view(self) -> AtomId {
        self.data_view
    }

    #[inline]
    pub const fn typed_array(self) -> AtomId {
        self.typed_array
    }

    #[inline]
    pub const fn atomics(self) -> AtomId {
        self.atomics
    }

    #[inline]
    pub const fn decode_uri(self) -> AtomId {
        self.decode_uri
    }

    #[inline]
    pub const fn decode_uri_component(self) -> AtomId {
        self.decode_uri_component
    }

    #[inline]
    pub const fn freeze(self) -> AtomId {
        self.freeze
    }

    #[inline]
    pub const fn function(self) -> AtomId {
        self.function
    }

    #[inline]
    pub const fn array(self) -> AtomId {
        self.array
    }

    #[inline]
    pub const fn get_own_property_descriptor(self) -> AtomId {
        self.get_own_property_descriptor
    }

    #[inline]
    pub const fn get_prototype_of(self) -> AtomId {
        self.get_prototype_of
    }

    #[inline]
    pub const fn global(self) -> AtomId {
        self.global
    }

    #[inline]
    pub const fn global_this(self) -> AtomId {
        self.global_this
    }

    #[inline]
    pub const fn bigint(self) -> AtomId {
        self.bigint
    }

    #[inline]
    pub const fn encode_uri(self) -> AtomId {
        self.encode_uri
    }

    #[inline]
    pub const fn encode_uri_component(self) -> AtomId {
        self.encode_uri_component
    }

    #[inline]
    pub const fn escape(self) -> AtomId {
        self.escape
    }

    #[inline]
    pub const fn flags(self) -> AtomId {
        self.flags
    }

    #[inline]
    pub const fn has_indices(self) -> AtomId {
        self.has_indices
    }

    #[inline]
    pub const fn has_own_property(self) -> AtomId {
        self.has_own_property
    }

    #[inline]
    pub const fn infinity(self) -> AtomId {
        self.infinity
    }

    #[inline]
    pub const fn is_finite(self) -> AtomId {
        self.is_finite
    }

    #[inline]
    pub const fn is_nan(self) -> AtomId {
        self.is_nan
    }

    #[inline]
    pub const fn is_extensible(self) -> AtomId {
        self.is_extensible
    }

    #[inline]
    pub const fn is_frozen(self) -> AtomId {
        self.is_frozen
    }

    #[inline]
    pub const fn is_prototype_of(self) -> AtomId {
        self.is_prototype_of
    }

    #[inline]
    pub const fn is_sealed(self) -> AtomId {
        self.is_sealed
    }

    #[inline]
    pub const fn last_index(self) -> AtomId {
        self.last_index
    }

    #[inline]
    pub const fn message(self) -> AtomId {
        self.message
    }

    #[inline]
    pub const fn math(self) -> AtomId {
        self.math
    }

    #[inline]
    pub const fn nan(self) -> AtomId {
        self.nan
    }

    #[inline]
    pub const fn number(self) -> AtomId {
        self.number
    }

    #[inline]
    pub const fn object(self) -> AtomId {
        self.object
    }

    #[inline]
    pub const fn map(self) -> AtomId {
        self.map
    }

    #[inline]
    pub const fn set(self) -> AtomId {
        self.set
    }

    #[inline]
    pub const fn weak_map(self) -> AtomId {
        self.weak_map
    }

    #[inline]
    pub const fn weak_set(self) -> AtomId {
        self.weak_set
    }

    #[inline]
    pub const fn weak_ref(self) -> AtomId {
        self.weak_ref
    }

    #[inline]
    pub const fn finalization_registry(self) -> AtomId {
        self.finalization_registry
    }

    #[inline]
    pub const fn json(self) -> AtomId {
        self.json
    }

    #[inline]
    pub const fn int8_array(self) -> AtomId {
        self.int8_array
    }

    #[inline]
    pub const fn int16_array(self) -> AtomId {
        self.int16_array
    }

    #[inline]
    pub const fn int32_array(self) -> AtomId {
        self.int32_array
    }

    #[inline]
    pub const fn float32_array(self) -> AtomId {
        self.float32_array
    }

    #[inline]
    pub const fn float64_array(self) -> AtomId {
        self.float64_array
    }

    #[inline]
    pub const fn big_int64_array(self) -> AtomId {
        self.big_int64_array
    }

    #[inline]
    pub const fn big_uint64_array(self) -> AtomId {
        self.big_uint64_array
    }

    #[inline]
    pub const fn uint32_array(self) -> AtomId {
        self.uint32_array
    }

    #[inline]
    pub const fn uint16_array(self) -> AtomId {
        self.uint16_array
    }

    #[inline]
    pub const fn uint8_clamped_array(self) -> AtomId {
        self.uint8_clamped_array
    }

    #[inline]
    pub const fn uint8_array(self) -> AtomId {
        self.uint8_array
    }

    #[inline]
    pub const fn promise(self) -> AtomId {
        self.promise
    }

    #[inline]
    pub const fn aggregate_error(self) -> AtomId {
        self.aggregate_error
    }

    #[inline]
    pub const fn parse_float(self) -> AtomId {
        self.parse_float
    }

    #[inline]
    pub const fn parse_int(self) -> AtomId {
        self.parse_int
    }

    #[inline]
    pub const fn prevent_extensions(self) -> AtomId {
        self.prevent_extensions
    }

    #[inline]
    pub const fn property_is_enumerable(self) -> AtomId {
        self.property_is_enumerable
    }

    #[inline]
    pub const fn range_error(self) -> AtomId {
        self.range_error
    }

    #[inline]
    pub const fn reference_error(self) -> AtomId {
        self.reference_error
    }

    #[inline]
    pub const fn regexp(self) -> AtomId {
        self.regexp
    }

    #[inline]
    pub const fn seal(self) -> AtomId {
        self.seal
    }

    #[inline]
    pub const fn set_prototype_of(self) -> AtomId {
        self.set_prototype_of
    }

    #[inline]
    pub const fn source(self) -> AtomId {
        self.source
    }

    #[inline]
    pub const fn string(self) -> AtomId {
        self.string
    }

    #[inline]
    pub const fn syntax_error(self) -> AtomId {
        self.syntax_error
    }

    #[inline]
    pub const fn symbol(self) -> AtomId {
        self.symbol
    }

    #[inline]
    pub const fn type_error(self) -> AtomId {
        self.type_error
    }

    #[inline]
    pub const fn uri_error(self) -> AtomId {
        self.uri_error
    }

    #[inline]
    pub const fn unescape(self) -> AtomId {
        self.unescape
    }

    #[inline]
    pub const fn key_for(self) -> AtomId {
        self.key_for
    }

    #[inline]
    pub const fn undefined(self) -> AtomId {
        self.undefined
    }

    #[inline]
    pub const fn has_instance(self) -> AtomId {
        self.has_instance
    }

    #[inline]
    pub const fn is_concat_spreadable(self) -> AtomId {
        self.is_concat_spreadable
    }

    #[inline]
    pub const fn iterator(self) -> AtomId {
        self.iterator
    }

    #[inline]
    pub const fn async_iterator(self) -> AtomId {
        self.async_iterator
    }

    #[inline]
    pub const fn match_(self) -> AtomId {
        self.match_
    }

    #[inline]
    pub const fn match_all(self) -> AtomId {
        self.match_all
    }

    #[inline]
    pub const fn replace(self) -> AtomId {
        self.replace
    }

    #[inline]
    pub const fn search(self) -> AtomId {
        self.search
    }

    #[inline]
    pub const fn species(self) -> AtomId {
        self.species
    }

    #[inline]
    pub const fn split(self) -> AtomId {
        self.split
    }

    #[inline]
    pub const fn to_primitive(self) -> AtomId {
        self.to_primitive
    }

    #[inline]
    pub const fn to_string_tag(self) -> AtomId {
        self.to_string_tag
    }

    #[inline]
    pub const fn unscopables(self) -> AtomId {
        self.unscopables
    }

    #[inline]
    pub const fn dispose(self) -> AtomId {
        self.dispose
    }

    #[inline]
    pub const fn async_dispose(self) -> AtomId {
        self.async_dispose
    }

    #[inline]
    pub const fn well_known_symbol_description(self, id: WellKnownSymbolId) -> AtomId {
        match id {
            WellKnownSymbolId::HasInstance => self.symbol_has_instance,
            WellKnownSymbolId::IsConcatSpreadable => self.symbol_is_concat_spreadable,
            WellKnownSymbolId::Iterator => self.symbol_iterator,
            WellKnownSymbolId::AsyncIterator => self.symbol_async_iterator,
            WellKnownSymbolId::Match => self.symbol_match,
            WellKnownSymbolId::MatchAll => self.symbol_match_all,
            WellKnownSymbolId::Replace => self.symbol_replace,
            WellKnownSymbolId::Search => self.symbol_search,
            WellKnownSymbolId::Species => self.symbol_species,
            WellKnownSymbolId::Split => self.symbol_split,
            WellKnownSymbolId::ToPrimitive => self.symbol_to_primitive,
            WellKnownSymbolId::ToStringTag => self.symbol_to_string_tag,
            WellKnownSymbolId::Unscopables => self.symbol_unscopables,
            WellKnownSymbolId::Dispose => self.symbol_dispose,
            WellKnownSymbolId::AsyncDispose => self.symbol_async_dispose,
        }
    }
}

impl TraceAtomEdges for BootstrapAtoms {
    fn trace_atom_edges(&self, sweep: &mut AtomGcSweep<'_>) {
        self.boolean.trace_atom_edges(sweep);
        self.create.trace_atom_edges(sweep);
        self.create_realm.trace_atom_edges(sweep);
        self.define_property.trace_atom_edges(sweep);
        self.error.trace_atom_edges(sweep);
        self.eval_script.trace_atom_edges(sweep);
        self.eval_error.trace_atom_edges(sweep);
        self.date.trace_atom_edges(sweep);
        self.array_buffer.trace_atom_edges(sweep);
        self.shared_array_buffer.trace_atom_edges(sweep);
        self.data_view.trace_atom_edges(sweep);
        self.typed_array.trace_atom_edges(sweep);
        self.atomics.trace_atom_edges(sweep);
        self.decode_uri.trace_atom_edges(sweep);
        self.decode_uri_component.trace_atom_edges(sweep);
        self.freeze.trace_atom_edges(sweep);
        self.function.trace_atom_edges(sweep);
        self.array.trace_atom_edges(sweep);
        self.get_own_property_descriptor.trace_atom_edges(sweep);
        self.get_prototype_of.trace_atom_edges(sweep);
        self.global.trace_atom_edges(sweep);
        self.global_this.trace_atom_edges(sweep);
        self.bigint.trace_atom_edges(sweep);
        self.encode_uri.trace_atom_edges(sweep);
        self.encode_uri_component.trace_atom_edges(sweep);
        self.escape.trace_atom_edges(sweep);
        self.flags.trace_atom_edges(sweep);
        self.has_indices.trace_atom_edges(sweep);
        self.has_own_property.trace_atom_edges(sweep);
        self.infinity.trace_atom_edges(sweep);
        self.is_finite.trace_atom_edges(sweep);
        self.is_nan.trace_atom_edges(sweep);
        self.is_extensible.trace_atom_edges(sweep);
        self.is_frozen.trace_atom_edges(sweep);
        self.is_prototype_of.trace_atom_edges(sweep);
        self.is_sealed.trace_atom_edges(sweep);
        self.last_index.trace_atom_edges(sweep);
        self.message.trace_atom_edges(sweep);
        self.math.trace_atom_edges(sweep);
        self.nan.trace_atom_edges(sweep);
        self.number.trace_atom_edges(sweep);
        self.object.trace_atom_edges(sweep);
        self.map.trace_atom_edges(sweep);
        self.set.trace_atom_edges(sweep);
        self.json.trace_atom_edges(sweep);
        self.int8_array.trace_atom_edges(sweep);
        self.int16_array.trace_atom_edges(sweep);
        self.int32_array.trace_atom_edges(sweep);
        self.float32_array.trace_atom_edges(sweep);
        self.float64_array.trace_atom_edges(sweep);
        self.big_int64_array.trace_atom_edges(sweep);
        self.big_uint64_array.trace_atom_edges(sweep);
        self.uint32_array.trace_atom_edges(sweep);
        self.uint16_array.trace_atom_edges(sweep);
        self.uint8_clamped_array.trace_atom_edges(sweep);
        self.uint8_array.trace_atom_edges(sweep);
        self.promise.trace_atom_edges(sweep);
        self.aggregate_error.trace_atom_edges(sweep);
        self.parse_float.trace_atom_edges(sweep);
        self.parse_int.trace_atom_edges(sweep);
        self.prevent_extensions.trace_atom_edges(sweep);
        self.property_is_enumerable.trace_atom_edges(sweep);
        self.range_error.trace_atom_edges(sweep);
        self.reference_error.trace_atom_edges(sweep);
        self.regexp.trace_atom_edges(sweep);
        self.seal.trace_atom_edges(sweep);
        self.set_prototype_of.trace_atom_edges(sweep);
        self.source.trace_atom_edges(sweep);
        self.string.trace_atom_edges(sweep);
        self.syntax_error.trace_atom_edges(sweep);
        self.symbol.trace_atom_edges(sweep);
        self.type_error.trace_atom_edges(sweep);
        self.uri_error.trace_atom_edges(sweep);
        self.unescape.trace_atom_edges(sweep);
        self.key_for.trace_atom_edges(sweep);
        self.undefined.trace_atom_edges(sweep);
        self.has_instance.trace_atom_edges(sweep);
        self.is_concat_spreadable.trace_atom_edges(sweep);
        self.iterator.trace_atom_edges(sweep);
        self.async_iterator.trace_atom_edges(sweep);
        self.match_.trace_atom_edges(sweep);
        self.match_all.trace_atom_edges(sweep);
        self.replace.trace_atom_edges(sweep);
        self.search.trace_atom_edges(sweep);
        self.species.trace_atom_edges(sweep);
        self.split.trace_atom_edges(sweep);
        self.to_primitive.trace_atom_edges(sweep);
        self.to_string_tag.trace_atom_edges(sweep);
        self.unscopables.trace_atom_edges(sweep);
        self.dispose.trace_atom_edges(sweep);
        self.async_dispose.trace_atom_edges(sweep);
        self.symbol_has_instance.trace_atom_edges(sweep);
        self.symbol_is_concat_spreadable.trace_atom_edges(sweep);
        self.symbol_iterator.trace_atom_edges(sweep);
        self.symbol_async_iterator.trace_atom_edges(sweep);
        self.symbol_match.trace_atom_edges(sweep);
        self.symbol_match_all.trace_atom_edges(sweep);
        self.symbol_replace.trace_atom_edges(sweep);
        self.symbol_search.trace_atom_edges(sweep);
        self.symbol_species.trace_atom_edges(sweep);
        self.symbol_split.trace_atom_edges(sweep);
        self.symbol_to_primitive.trace_atom_edges(sweep);
        self.symbol_to_string_tag.trace_atom_edges(sweep);
        self.symbol_unscopables.trace_atom_edges(sweep);
        self.symbol_dispose.trace_atom_edges(sweep);
        self.symbol_async_dispose.trace_atom_edges(sweep);
    }
}

/// Agent-owned well-known symbol table used by the Phase 5 builtin bootstrap.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WellKnownSymbols {
    has_instance: Option<SymbolRef>,
    is_concat_spreadable: Option<SymbolRef>,
    iterator: Option<SymbolRef>,
    async_iterator: Option<SymbolRef>,
    match_: Option<SymbolRef>,
    match_all: Option<SymbolRef>,
    replace: Option<SymbolRef>,
    search: Option<SymbolRef>,
    species: Option<SymbolRef>,
    split: Option<SymbolRef>,
    to_primitive: Option<SymbolRef>,
    to_string_tag: Option<SymbolRef>,
    unscopables: Option<SymbolRef>,
    dispose: Option<SymbolRef>,
    async_dispose: Option<SymbolRef>,
}

impl WellKnownSymbols {
    #[inline]
    pub const fn new() -> Self {
        Self {
            has_instance: None,
            is_concat_spreadable: None,
            iterator: None,
            async_iterator: None,
            match_: None,
            match_all: None,
            replace: None,
            search: None,
            species: None,
            split: None,
            to_primitive: None,
            to_string_tag: None,
            unscopables: None,
            dispose: None,
            async_dispose: None,
        }
    }

    #[inline]
    pub const fn get(self, id: WellKnownSymbolId) -> Option<SymbolRef> {
        match id {
            WellKnownSymbolId::HasInstance => self.has_instance,
            WellKnownSymbolId::IsConcatSpreadable => self.is_concat_spreadable,
            WellKnownSymbolId::Iterator => self.iterator,
            WellKnownSymbolId::AsyncIterator => self.async_iterator,
            WellKnownSymbolId::Match => self.match_,
            WellKnownSymbolId::MatchAll => self.match_all,
            WellKnownSymbolId::Replace => self.replace,
            WellKnownSymbolId::Search => self.search,
            WellKnownSymbolId::Species => self.species,
            WellKnownSymbolId::Split => self.split,
            WellKnownSymbolId::ToPrimitive => self.to_primitive,
            WellKnownSymbolId::ToStringTag => self.to_string_tag,
            WellKnownSymbolId::Unscopables => self.unscopables,
            WellKnownSymbolId::Dispose => self.dispose,
            WellKnownSymbolId::AsyncDispose => self.async_dispose,
        }
    }

    #[inline]
    pub fn set(&mut self, id: WellKnownSymbolId, value: Option<SymbolRef>) {
        match id {
            WellKnownSymbolId::HasInstance => self.has_instance = value,
            WellKnownSymbolId::IsConcatSpreadable => self.is_concat_spreadable = value,
            WellKnownSymbolId::Iterator => self.iterator = value,
            WellKnownSymbolId::AsyncIterator => self.async_iterator = value,
            WellKnownSymbolId::Match => self.match_ = value,
            WellKnownSymbolId::MatchAll => self.match_all = value,
            WellKnownSymbolId::Replace => self.replace = value,
            WellKnownSymbolId::Search => self.search = value,
            WellKnownSymbolId::Species => self.species = value,
            WellKnownSymbolId::Split => self.split = value,
            WellKnownSymbolId::ToPrimitive => self.to_primitive = value,
            WellKnownSymbolId::ToStringTag => self.to_string_tag = value,
            WellKnownSymbolId::Unscopables => self.unscopables = value,
            WellKnownSymbolId::Dispose => self.dispose = value,
            WellKnownSymbolId::AsyncDispose => self.async_dispose = value,
        }
    }

    #[inline]
    pub const fn has_instance(self) -> Option<SymbolRef> {
        self.has_instance
    }

    #[inline]
    pub const fn is_concat_spreadable(self) -> Option<SymbolRef> {
        self.is_concat_spreadable
    }

    #[inline]
    pub const fn iterator(self) -> Option<SymbolRef> {
        self.iterator
    }

    #[inline]
    pub const fn async_iterator(self) -> Option<SymbolRef> {
        self.async_iterator
    }

    #[inline]
    pub const fn match_(self) -> Option<SymbolRef> {
        self.match_
    }

    #[inline]
    pub const fn match_all(self) -> Option<SymbolRef> {
        self.match_all
    }

    #[inline]
    pub const fn replace(self) -> Option<SymbolRef> {
        self.replace
    }

    #[inline]
    pub const fn search(self) -> Option<SymbolRef> {
        self.search
    }

    #[inline]
    pub const fn species(self) -> Option<SymbolRef> {
        self.species
    }

    #[inline]
    pub const fn split(self) -> Option<SymbolRef> {
        self.split
    }

    #[inline]
    pub const fn to_primitive(self) -> Option<SymbolRef> {
        self.to_primitive
    }

    #[inline]
    pub const fn to_string_tag(self) -> Option<SymbolRef> {
        self.to_string_tag
    }

    #[inline]
    pub const fn unscopables(self) -> Option<SymbolRef> {
        self.unscopables
    }

    #[inline]
    pub const fn dispose(self) -> Option<SymbolRef> {
        self.dispose
    }

    #[inline]
    pub const fn async_dispose(self) -> Option<SymbolRef> {
        self.async_dispose
    }
}

impl TraceHeapEdges for WellKnownSymbols {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.has_instance.trace_heap_edges(tracer);
        self.is_concat_spreadable.trace_heap_edges(tracer);
        self.iterator.trace_heap_edges(tracer);
        self.async_iterator.trace_heap_edges(tracer);
        self.match_.trace_heap_edges(tracer);
        self.match_all.trace_heap_edges(tracer);
        self.replace.trace_heap_edges(tracer);
        self.search.trace_heap_edges(tracer);
        self.species.trace_heap_edges(tracer);
        self.split.trace_heap_edges(tracer);
        self.to_primitive.trace_heap_edges(tracer);
        self.to_string_tag.trace_heap_edges(tracer);
        self.unscopables.trace_heap_edges(tracer);
        self.dispose.trace_heap_edges(tracer);
        self.async_dispose.trace_heap_edges(tracer);
    }
}

/// One agent-owned global symbol registry entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlobalSymbolRegistryEntry {
    key: AtomId,
    symbol: SymbolRef,
}

impl GlobalSymbolRegistryEntry {
    #[inline]
    pub const fn new(key: AtomId, symbol: SymbolRef) -> Self {
        Self { key, symbol }
    }

    #[inline]
    pub const fn key(self) -> AtomId {
        self.key
    }

    #[inline]
    pub const fn symbol(self) -> SymbolRef {
        self.symbol
    }
}

impl TraceAtomEdges for GlobalSymbolRegistryEntry {
    fn trace_atom_edges(&self, sweep: &mut AtomGcSweep<'_>) {
        self.key.trace_atom_edges(sweep);
    }
}

impl TraceHeapEdges for GlobalSymbolRegistryEntry {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.symbol.trace_heap_edges(tracer);
    }
}

/// Agent-owned `Symbol.for`/`Symbol.keyFor` registry state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GlobalSymbolRegistry {
    entries: Vec<GlobalSymbolRegistryEntry>,
}

impl GlobalSymbolRegistry {
    #[inline]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    #[inline]
    pub fn entries(&self) -> &[GlobalSymbolRegistryEntry] {
        &self.entries
    }

    #[inline]
    pub fn symbol_for(&self, key: AtomId) -> Option<SymbolRef> {
        self.entries
            .iter()
            .find(|entry| entry.key() == key)
            .map(|entry| entry.symbol())
    }

    #[inline]
    pub fn key_for(&self, symbol: SymbolRef) -> Option<AtomId> {
        self.entries
            .iter()
            .find(|entry| entry.symbol() == symbol)
            .map(|entry| entry.key())
    }

    #[inline]
    pub fn insert(&mut self, key: AtomId, symbol: SymbolRef) -> SymbolRef {
        if let Some(existing) = self.symbol_for(key) {
            debug_assert_eq!(
                existing, symbol,
                "global symbol registry keys must stay bound to one symbol identity"
            );
            return existing;
        }
        if let Some(existing_key) = self.key_for(symbol) {
            debug_assert_eq!(
                existing_key, key,
                "global symbol registry symbols must not be rebound under a second key"
            );
            return symbol;
        }
        self.entries
            .push(GlobalSymbolRegistryEntry::new(key, symbol));
        symbol
    }
}

impl TraceAtomEdges for GlobalSymbolRegistry {
    fn trace_atom_edges(&self, sweep: &mut AtomGcSweep<'_>) {
        for entry in &self.entries {
            entry.trace_atom_edges(sweep);
        }
    }
}

impl TraceHeapEdges for GlobalSymbolRegistry {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        for entry in &self.entries {
            entry.trace_heap_edges(tracer);
        }
    }
}
