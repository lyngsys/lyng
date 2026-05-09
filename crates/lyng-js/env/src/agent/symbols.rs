use super::{Agent, RecentShortLatin1String, RecentTwoCodeUnitString};
use crate::{AllocationLifetime, AtomTable, BootstrapAtoms, GlobalSymbolRegistryEntry};
use lyng_js_common::AtomId;
use lyng_js_gc::{StringEncoding, SymbolFlags};
use lyng_js_types::{StringRef, SymbolRef, WellKnownSymbolId};

impl Agent {
    #[inline]
    pub const fn atoms(&self) -> &AtomTable {
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
    pub const fn atoms_mut(&mut self) -> &mut AtomTable {
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

    /// Allocates a runtime string using the narrowest available internal encoding.
    ///
    /// # Panics
    /// Panics if the string length exceeds the `u32` code-unit limit used by runtime string
    /// headers.
    pub fn alloc_runtime_string(
        &mut self,
        text: &str,
        cached_atom: Option<AtomId>,
        lifetime: AllocationLifetime,
    ) -> StringRef {
        if let Some(atom) = cached_atom {
            return self.alloc_string_for_atom(atom, lifetime);
        }
        if let Ok(bytes) = text
            .chars()
            .map(|ch| u8::try_from(u32::from(ch)))
            .collect::<Result<Vec<_>, _>>()
        {
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

    pub fn latin1_single_code_unit_string(&mut self, unit: u8) -> StringRef {
        if let Some(string) = self.latin1_single_code_unit_strings[usize::from(unit)] {
            return string;
        }
        let string = self.heap.mutator().alloc_string(
            StringEncoding::Latin1,
            1,
            &[unit],
            None,
            AllocationLifetime::LongLived,
        );
        self.latin1_single_code_unit_strings[usize::from(unit)] = Some(string);
        string
    }

    /// Returns a cached two- or three-byte Latin-1 primitive string when available.
    ///
    /// # Panics
    /// Panics when `bytes` does not contain exactly two or three bytes.
    pub fn cached_short_latin1_string(
        &mut self,
        bytes: &[u8],
        lifetime: AllocationLifetime,
    ) -> StringRef {
        assert!(
            (2..=3).contains(&bytes.len()),
            "short Latin-1 string cache only supports two or three bytes"
        );
        let mut key = [0_u8; 3];
        key[..bytes.len()].copy_from_slice(bytes);
        let len = u8::try_from(bytes.len()).expect("short Latin-1 cache length should fit in u8");
        let index = short_latin1_cache_index(key, len);

        if let Some(cached) = self.recent_short_latin1_strings[index]
            && cached.len == len
            && cached.bytes == key
        {
            return cached.string;
        }

        let string = self.heap.mutator().alloc_string(
            StringEncoding::Latin1,
            u32::from(len),
            bytes,
            None,
            lifetime,
        );
        self.recent_short_latin1_strings[index] = Some(RecentShortLatin1String {
            bytes: key,
            len,
            string,
        });
        string
    }

    pub fn cached_two_code_unit_string(
        &mut self,
        units: [u16; 2],
        lifetime: AllocationLifetime,
    ) -> StringRef {
        if let Some(cached) = self.recent_two_code_unit_string
            && cached.units == units
        {
            return cached.string;
        }

        let string = if let (Ok(left), Ok(right)) = (u8::try_from(units[0]), u8::try_from(units[1]))
        {
            self.heap.mutator().alloc_string(
                StringEncoding::Latin1,
                2,
                &[left, right],
                None,
                lifetime,
            )
        } else {
            let mut bytes = [0_u8; 4];
            bytes[..2].copy_from_slice(&units[0].to_le_bytes());
            bytes[2..].copy_from_slice(&units[1].to_le_bytes());
            self.heap
                .mutator()
                .alloc_string(StringEncoding::Utf16, 2, &bytes, None, lifetime)
        };

        self.recent_two_code_unit_string = Some(RecentTwoCodeUnitString { units, string });
        string
    }

    pub(super) fn seed_builtin_symbol_state(&mut self, lifetime: AllocationLifetime) {
        for id in WellKnownSymbolId::ALL {
            let _ = self.ensure_well_known_symbol(id, lifetime);
        }
    }

    fn alloc_string_for_atom(&mut self, atom: AtomId, lifetime: AllocationLifetime) -> StringRef {
        if let Some(text) = self.atoms.get(atom) {
            let text = text.to_owned();
            if text.chars().all(|ch| u8::try_from(u32::from(ch)).is_ok()) {
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

fn short_latin1_cache_index(bytes: [u8; 3], len: u8) -> usize {
    let hash = usize::from(len).wrapping_mul(0x45D9)
        ^ usize::from(bytes[0]).wrapping_mul(0x9E37)
        ^ usize::from(bytes[1]).wrapping_mul(0x85EB)
        ^ usize::from(bytes[2]).wrapping_mul(0xC2B2);
    hash & 0xFF
}
