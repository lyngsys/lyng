use crate::ids::{BytecodeFunctionId, EnvironmentLayoutRef};
use crate::metadata::{ArgumentsMode, BytecodeFunctionFlags, BytecodeFunctionKind, ThisMode};
use lyng_js_common::{AtomId, Span};

/// Header metadata frozen for one immutable bytecode template.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BytecodeFunctionHeader {
    id: BytecodeFunctionId,
    kind: BytecodeFunctionKind,
    flags: BytecodeFunctionFlags,
    name: Option<AtomId>,
    this_mode: ThisMode,
    arguments_mode: ArgumentsMode,
    parameter_count: u16,
    minimum_argument_count: u16,
    parameter_initializer_end_offset: u32,
    register_count: u16,
    hidden_register_count: u16,
    needs_environment: bool,
    environment_slot_count: u16,
    has_rest_parameter: bool,
    environment_layout: Option<EnvironmentLayoutRef>,
    source_span: Option<Span>,
}

impl BytecodeFunctionHeader {
    #[inline]
    pub const fn new(
        id: BytecodeFunctionId,
        kind: BytecodeFunctionKind,
        name: Option<AtomId>,
        this_mode: ThisMode,
        arguments_mode: ArgumentsMode,
    ) -> Self {
        Self {
            id,
            kind,
            flags: BytecodeFunctionFlags::empty(),
            name,
            this_mode,
            arguments_mode,
            parameter_count: 0,
            minimum_argument_count: 0,
            parameter_initializer_end_offset: 0,
            register_count: 0,
            hidden_register_count: 0,
            needs_environment: false,
            environment_slot_count: 0,
            has_rest_parameter: false,
            environment_layout: None,
            source_span: None,
        }
    }

    #[inline]
    pub const fn with_flags(mut self, flags: BytecodeFunctionFlags) -> Self {
        self.flags = flags;
        self
    }

    #[inline]
    pub const fn with_kind(mut self, kind: BytecodeFunctionKind) -> Self {
        self.kind = kind;
        self
    }

    #[inline]
    pub const fn with_this_mode(mut self, this_mode: ThisMode) -> Self {
        self.this_mode = this_mode;
        self
    }

    #[inline]
    pub const fn with_parameter_counts(
        mut self,
        parameter_count: u16,
        minimum_argument_count: u16,
    ) -> Self {
        self.parameter_count = parameter_count;
        self.minimum_argument_count = minimum_argument_count;
        self
    }

    #[inline]
    pub const fn with_parameter_initializer_end_offset(
        mut self,
        parameter_initializer_end_offset: u32,
    ) -> Self {
        self.parameter_initializer_end_offset = parameter_initializer_end_offset;
        self
    }

    #[inline]
    pub const fn with_register_counts(
        mut self,
        register_count: u16,
        hidden_register_count: u16,
    ) -> Self {
        self.register_count = register_count;
        self.hidden_register_count = hidden_register_count;
        self
    }

    #[inline]
    pub const fn with_environment_layout(
        mut self,
        environment_layout: Option<EnvironmentLayoutRef>,
    ) -> Self {
        self.environment_layout = environment_layout;
        self
    }

    #[inline]
    pub const fn with_source_span(mut self, source_span: Option<Span>) -> Self {
        self.source_span = source_span;
        self
    }

    #[inline]
    pub const fn with_needs_environment(mut self, needs_environment: bool) -> Self {
        self.needs_environment = needs_environment;
        self
    }

    #[inline]
    pub const fn with_environment_slot_count(mut self, environment_slot_count: u16) -> Self {
        self.environment_slot_count = environment_slot_count;
        self
    }

    #[inline]
    pub const fn with_has_rest_parameter(mut self, has_rest_parameter: bool) -> Self {
        self.has_rest_parameter = has_rest_parameter;
        self
    }

    #[inline]
    pub const fn id(self) -> BytecodeFunctionId {
        self.id
    }

    #[inline]
    pub const fn kind(self) -> BytecodeFunctionKind {
        self.kind
    }

    #[inline]
    pub const fn flags(self) -> BytecodeFunctionFlags {
        self.flags
    }

    #[inline]
    pub const fn name(self) -> Option<AtomId> {
        self.name
    }

    #[inline]
    pub const fn this_mode(self) -> ThisMode {
        self.this_mode
    }

    #[inline]
    pub const fn arguments_mode(self) -> ArgumentsMode {
        self.arguments_mode
    }

    #[inline]
    pub const fn parameter_count(self) -> u16 {
        self.parameter_count
    }

    #[inline]
    pub const fn minimum_argument_count(self) -> u16 {
        self.minimum_argument_count
    }

    #[inline]
    pub const fn parameter_initializer_end_offset(self) -> u32 {
        self.parameter_initializer_end_offset
    }

    #[inline]
    pub const fn register_count(self) -> u16 {
        self.register_count
    }

    #[inline]
    pub const fn hidden_register_count(self) -> u16 {
        self.hidden_register_count
    }

    #[inline]
    pub const fn needs_environment(self) -> bool {
        self.needs_environment
    }

    #[inline]
    pub const fn environment_slot_count(self) -> u16 {
        self.environment_slot_count
    }

    #[inline]
    pub const fn has_rest_parameter(self) -> bool {
        self.has_rest_parameter
    }

    #[inline]
    pub const fn environment_layout(self) -> Option<EnvironmentLayoutRef> {
        self.environment_layout
    }

    #[inline]
    pub const fn source_span(self) -> Option<Span> {
        self.source_span
    }
}
