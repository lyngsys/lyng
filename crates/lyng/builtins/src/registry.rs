use crate::BuiltinHandler;
use lyng_js_types::BuiltinFunctionId;

/// Cold metadata for one registered builtin entrypoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinEntryMetadata {
    display_name: &'static str,
    length: u16,
    constructible: bool,
    has_prototype_property: bool,
}

impl BuiltinEntryMetadata {
    #[inline]
    pub const fn new(
        display_name: &'static str,
        length: u16,
        constructible: bool,
        has_prototype_property: bool,
    ) -> Self {
        Self {
            display_name,
            length,
            constructible,
            has_prototype_property,
        }
    }

    #[inline]
    pub const fn display_name(self) -> &'static str {
        self.display_name
    }

    #[inline]
    pub const fn length(self) -> u16 {
        self.length
    }

    #[inline]
    pub const fn constructible(self) -> bool {
        self.constructible
    }

    #[inline]
    pub const fn has_prototype_property(self) -> bool {
        self.has_prototype_property
    }
}

/// One registry entry binding a builtin ID to metadata and a native handler.
#[derive(Clone, Copy)]
pub struct BuiltinRegistryEntry<Cx> {
    id: BuiltinFunctionId,
    metadata: BuiltinEntryMetadata,
    handler: BuiltinHandler<Cx>,
}

impl<Cx> BuiltinRegistryEntry<Cx> {
    #[inline]
    pub const fn new(
        id: BuiltinFunctionId,
        metadata: BuiltinEntryMetadata,
        handler: BuiltinHandler<Cx>,
    ) -> Self {
        Self {
            id,
            metadata,
            handler,
        }
    }

    #[inline]
    pub const fn id(&self) -> BuiltinFunctionId {
        self.id
    }

    #[inline]
    pub const fn metadata(&self) -> BuiltinEntryMetadata {
        self.metadata
    }

    #[inline]
    pub const fn handler(&self) -> BuiltinHandler<Cx> {
        self.handler
    }
}

impl<Cx> std::fmt::Debug for BuiltinRegistryEntry<Cx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuiltinRegistryEntry")
            .field("id", &self.id)
            .field("metadata", &self.metadata)
            .finish_non_exhaustive()
    }
}

impl<Cx> PartialEq for BuiltinRegistryEntry<Cx> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.metadata == other.metadata
            && std::ptr::fn_addr_eq(self.handler, other.handler)
    }
}

impl<Cx> Eq for BuiltinRegistryEntry<Cx> {}

/// Registry insertion error for duplicate builtin IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinRegistryError {
    DuplicateBuiltinId(BuiltinFunctionId),
}

/// Builtin-entry registry owned by `lyng_js_builtins`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuiltinRegistry<Cx> {
    entries: Vec<BuiltinRegistryEntry<Cx>>,
}

impl<Cx> BuiltinRegistry<Cx> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    #[inline]
    pub fn entries(&self) -> &[BuiltinRegistryEntry<Cx>] {
        &self.entries
    }

    #[inline]
    pub fn entry(&self, id: BuiltinFunctionId) -> Option<&BuiltinRegistryEntry<Cx>> {
        self.entries.iter().find(|entry| entry.id() == id)
    }

    /// Registers one builtin entry.
    ///
    /// # Errors
    /// Returns an error when the registry already contains the same builtin ID.
    pub fn register(
        &mut self,
        entry: BuiltinRegistryEntry<Cx>,
    ) -> Result<(), BuiltinRegistryError> {
        if self.entry(entry.id()).is_some() {
            return Err(BuiltinRegistryError::DuplicateBuiltinId(entry.id()));
        }
        self.entries.push(entry);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_types::{Completion, ObjectRef, Value};

    struct DummyContext;

    #[allow(
        clippy::unnecessary_wraps,
        reason = "test handler must satisfy the builtin handler function-pointer signature"
    )]
    fn handler(
        _: &mut DummyContext,
        _: Value,
        _: &[Value],
        _: Option<ObjectRef>,
    ) -> Completion<Value> {
        Ok(Value::undefined())
    }

    #[test]
    fn registry_rejects_duplicate_builtin_ids() {
        let mut registry = BuiltinRegistry::new();
        let metadata = BuiltinEntryMetadata::new("call", 1, false, false);
        let entry_id = BuiltinFunctionId::from_raw(21).expect("non-zero builtin id");

        registry
            .register(BuiltinRegistryEntry::new(entry_id, metadata, handler))
            .expect("first registration should succeed");
        let duplicate = registry.register(BuiltinRegistryEntry::new(entry_id, metadata, handler));

        assert_eq!(
            duplicate,
            Err(BuiltinRegistryError::DuplicateBuiltinId(entry_id))
        );
        assert_eq!(registry.entries().len(), 1);
    }
}
