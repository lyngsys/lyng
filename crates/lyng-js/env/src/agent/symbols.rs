use super::Agent;
use crate::{AllocationLifetime, AtomTable, BootstrapAtoms, GlobalSymbolRegistryEntry};
use lyng_js_common::AtomId;
use lyng_js_gc::{StringEncoding, SymbolFlags};
use lyng_js_types::{StringRef, SymbolRef, WellKnownSymbolId};

impl Agent {
    #[inline]
    pub fn atoms(&self) -> &AtomTable {
        &self.atoms
    }

    #[inline]
    pub const fn bootstrap_atoms(&self) -> BootstrapAtoms {
        self.bootstrap_atoms
    }

    #[inline]
    pub const fn well_known_symbols(&self) -> crate::WellKnownSymbols {
        self.well_known_symbols
    }

    #[inline]
    pub const fn well_known_symbol(&self, id: WellKnownSymbolId) -> Option<SymbolRef> {
        self.well_known_symbols.get(id)
    }

    #[inline]
    pub fn global_symbol_registry(&self) -> &[GlobalSymbolRegistryEntry] {
        self.global_symbol_registry.entries()
    }

    #[inline]
    pub fn global_symbol(&self, key: AtomId) -> Option<SymbolRef> {
        self.global_symbol_registry.symbol_for(key)
    }

    #[inline]
    pub fn global_symbol_key_for(&self, symbol: SymbolRef) -> Option<AtomId> {
        self.global_symbol_registry.key_for(symbol)
    }

    #[inline]
    pub fn atoms_mut(&mut self) -> &mut AtomTable {
        &mut self.atoms
    }

    pub fn ensure_well_known_symbol(
        &mut self,
        id: WellKnownSymbolId,
        lifetime: AllocationLifetime,
    ) -> SymbolRef {
        if let Some(symbol) = self.well_known_symbol(id) {
            return symbol;
        }

        let description_atom = self.bootstrap_atoms.well_known_symbol_description(id);
        let description = self.alloc_string_for_atom(description_atom, lifetime);
        let symbol = self.heap.mutator().alloc_symbol(
            Some(description),
            SymbolFlags::well_known(),
            lifetime,
        );
        self.well_known_symbols.set(id, Some(symbol));
        symbol
    }

    pub fn global_symbol_for(&mut self, key: AtomId, lifetime: AllocationLifetime) -> SymbolRef {
        if let Some(symbol) = self.global_symbol(key) {
            return symbol;
        }

        let description = self.alloc_string_for_atom(key, lifetime);
        let symbol =
            self.heap
                .mutator()
                .alloc_symbol(Some(description), SymbolFlags::ordinary(), lifetime);
        self.global_symbol_registry.insert(key, symbol)
    }

    pub fn alloc_runtime_string(
        &mut self,
        text: &str,
        cached_atom: Option<AtomId>,
        lifetime: AllocationLifetime,
    ) -> StringRef {
        if let Some(atom) = cached_atom {
            return self.alloc_string_for_atom(atom, lifetime);
        }
        if text.chars().all(|ch| u32::from(ch) <= u32::from(u8::MAX)) {
            let bytes: Vec<u8> = text
                .chars()
                .map(|ch| u8::try_from(u32::from(ch)).expect("Latin-1 code point must fit into u8"))
                .collect();
            return self.heap.mutator().alloc_string(
                StringEncoding::Latin1,
                u32::try_from(bytes.len()).expect("Latin-1 string length must fit into u32"),
                &bytes,
                None,
                lifetime,
            );
        }

        let code_units: Vec<u16> = text.encode_utf16().collect();
        let mut bytes = Vec::with_capacity(code_units.len() * 2);
        for unit in &code_units {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        self.heap.mutator().alloc_string(
            StringEncoding::Utf16,
            u32::try_from(code_units.len()).expect("UTF-16 string length must fit into u32"),
            &bytes,
            None,
            lifetime,
        )
    }

    pub(super) fn seed_phase5_symbol_state(&mut self, lifetime: AllocationLifetime) {
        for id in WellKnownSymbolId::ALL {
            let _ = self.ensure_well_known_symbol(id, lifetime);
        }
    }

    fn alloc_string_for_atom(&mut self, atom: AtomId, lifetime: AllocationLifetime) -> StringRef {
        if let Some(text) = self.atoms.get(atom) {
            let text = text.to_owned();
            if text.chars().all(|ch| u32::from(ch) <= u32::from(u8::MAX)) {
                let bytes: Vec<u8> = text
                    .chars()
                    .map(|ch| {
                        u8::try_from(u32::from(ch)).expect("Latin-1 code point must fit into u8")
                    })
                    .collect();
                return self.heap.mutator().alloc_string(
                    StringEncoding::Latin1,
                    u32::try_from(bytes.len()).expect("Latin-1 string length must fit into u32"),
                    &bytes,
                    Some(atom),
                    lifetime,
                );
            }

            let code_units: Vec<u16> = text.encode_utf16().collect();
            let mut bytes = Vec::with_capacity(code_units.len() * 2);
            for unit in &code_units {
                bytes.extend_from_slice(&unit.to_le_bytes());
            }
            return self.heap.mutator().alloc_string(
                StringEncoding::Utf16,
                u32::try_from(code_units.len()).expect("UTF-16 string length must fit into u32"),
                &bytes,
                Some(atom),
                lifetime,
            );
        }

        let code_units = self
            .atoms
            .get_utf16(atom)
            .expect("atom should resolve to UTF-8 or UTF-16 storage")
            .to_vec();
        let mut bytes = Vec::with_capacity(code_units.len() * 2);
        for unit in &code_units {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        self.heap.mutator().alloc_string(
            StringEncoding::Utf16,
            u32::try_from(code_units.len()).expect("UTF-16 string length must fit into u32"),
            &bytes,
            Some(atom),
            lifetime,
        )
    }
}
