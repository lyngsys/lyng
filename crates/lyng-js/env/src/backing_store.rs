use super::RuntimeDomainAccounting;
use lyng_js_types::BackingStoreRef;
use std::mem::size_of;
use std::sync::{Arc, Mutex};

pub const MAX_BACKING_STORE_BYTE_LENGTH: usize = 1 << 30;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackingStoreRecord {
    Local(LocalBackingStoreRecord),
    Shared(SharedBackingStoreRecord),
}

impl BackingStoreRecord {
    fn new_local(byte_length: usize) -> Option<Self> {
        Some(Self::Local(LocalBackingStoreRecord {
            bytes: zeroed_backing_store_bytes(byte_length)?,
            detached: false,
        }))
    }

    fn new_shared(byte_length: usize) -> Option<Self> {
        Some(Self::Shared(SharedBackingStoreRecord::new(byte_length)?))
    }

    const fn byte_length(&self) -> usize {
        match self {
            Self::Local(record) => record.byte_length(),
            Self::Shared(record) => record.byte_length(),
        }
    }

    const fn is_detached(&self) -> bool {
        match self {
            Self::Local(record) => record.is_detached(),
            Self::Shared(record) => record.is_detached(),
        }
    }

    const fn is_shared(&self) -> bool {
        matches!(self, Self::Shared(_))
    }

    fn get_byte(&self, index: usize) -> Option<u8> {
        match self {
            Self::Local(record) => record.get_byte(index),
            Self::Shared(record) => record.get_byte(index),
        }
    }

    fn set_byte(&mut self, index: usize, value: u8) -> bool {
        match self {
            Self::Local(record) => record.set_byte(index, value),
            Self::Shared(record) => record.set_byte(index, value),
        }
    }

    fn load_bits(&self, index: usize, byte_width: usize) -> Option<u64> {
        match self {
            Self::Local(record) => record.load_bits(index, byte_width),
            Self::Shared(record) => record.load_bits(index, byte_width),
        }
    }

    fn store_bits(&mut self, index: usize, byte_width: usize, bits: u64) -> bool {
        match self {
            Self::Local(record) => record.store_bits(index, byte_width, bits),
            Self::Shared(record) => record.store_bits(index, byte_width, bits),
        }
    }

    fn resize(&mut self, byte_length: usize) -> bool {
        match self {
            Self::Local(record) => record.resize(byte_length),
            Self::Shared(_) => false,
        }
    }

    fn grow_shared(&mut self, byte_length: usize) -> bool {
        match self {
            Self::Shared(record) => record.grow(byte_length),
            Self::Local(_) => false,
        }
    }

    fn atomic_load_bits(&self, index: usize, byte_width: usize) -> Option<u64> {
        match self {
            Self::Local(record) => record.atomic_load_bits(index, byte_width),
            Self::Shared(record) => record.atomic_load_bits(index, byte_width),
        }
    }

    fn atomic_store_bits(&mut self, index: usize, byte_width: usize, bits: u64) -> bool {
        match self {
            Self::Local(record) => record.atomic_store_bits(index, byte_width, bits),
            Self::Shared(record) => record.atomic_store_bits(index, byte_width, bits),
        }
    }

    fn atomic_compare_exchange_bits(
        &mut self,
        index: usize,
        byte_width: usize,
        expected: u64,
        replacement: u64,
    ) -> Option<u64> {
        match self {
            Self::Local(record) => {
                record.atomic_compare_exchange_bits(index, byte_width, expected, replacement)
            }
            Self::Shared(record) => {
                record.atomic_compare_exchange_bits(index, byte_width, expected, replacement)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalBackingStoreRecord {
    bytes: Vec<u8>,
    detached: bool,
}

impl LocalBackingStoreRecord {
    const fn byte_length(&self) -> usize {
        if self.detached {
            0
        } else {
            self.bytes.len()
        }
    }

    const fn is_detached(&self) -> bool {
        self.detached
    }

    fn get_byte(&self, index: usize) -> Option<u8> {
        if self.detached {
            return None;
        }
        self.bytes.get(index).copied()
    }

    fn set_byte(&mut self, index: usize, value: u8) -> bool {
        if self.detached {
            return false;
        }
        let Some(slot) = self.bytes.get_mut(index) else {
            return false;
        };
        *slot = value;
        true
    }

    fn load_bits(&self, index: usize, byte_width: usize) -> Option<u64> {
        read_bits_from_bytes(&self.bytes, self.detached, index, byte_width)
    }

    fn store_bits(&mut self, index: usize, byte_width: usize, bits: u64) -> bool {
        write_bits_to_bytes(&mut self.bytes, self.detached, index, byte_width, bits)
    }

    fn resize(&mut self, byte_length: usize) -> bool {
        if self.detached {
            return false;
        }
        if byte_length > MAX_BACKING_STORE_BYTE_LENGTH {
            return false;
        }
        if byte_length > self.bytes.capacity()
            && self
                .bytes
                .try_reserve_exact(byte_length - self.bytes.len())
                .is_err()
        {
            return false;
        }
        self.bytes.resize(byte_length, 0);
        true
    }

    fn atomic_load_bits(&self, index: usize, byte_width: usize) -> Option<u64> {
        read_bits_from_bytes(&self.bytes, self.detached, index, byte_width)
    }

    fn atomic_store_bits(&mut self, index: usize, byte_width: usize, bits: u64) -> bool {
        write_bits_to_bytes(&mut self.bytes, self.detached, index, byte_width, bits)
    }

    fn atomic_compare_exchange_bits(
        &mut self,
        index: usize,
        byte_width: usize,
        expected: u64,
        replacement: u64,
    ) -> Option<u64> {
        if self.detached {
            return None;
        }
        let current = read_bits_from_bytes(&self.bytes, false, index, byte_width)?;
        if current == expected {
            let _ = write_bits_to_bytes(&mut self.bytes, false, index, byte_width, replacement);
        }
        Some(current)
    }
}

#[derive(Clone)]
pub struct SharedBackingStoreRecord {
    byte_length: usize,
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl SharedBackingStoreRecord {
    fn new(byte_length: usize) -> Option<Self> {
        Some(Self {
            byte_length,
            bytes: Arc::new(Mutex::new(zeroed_backing_store_bytes(byte_length)?)),
        })
    }

    const fn byte_length(&self) -> usize {
        self.byte_length
    }

    const fn is_detached(&self) -> bool {
        false
    }

    fn get_byte(&self, index: usize) -> Option<u8> {
        self.with_bytes(|bytes| bytes.get(index).copied())
    }

    fn set_byte(&self, index: usize, value: u8) -> bool {
        self.with_bytes_mut(|bytes| {
            let Some(slot) = bytes.get_mut(index) else {
                return false;
            };
            *slot = value;
            true
        })
    }

    fn load_bits(&self, index: usize, byte_width: usize) -> Option<u64> {
        self.with_bytes(|bytes| read_bits_from_bytes(bytes, false, index, byte_width))
    }

    fn store_bits(&self, index: usize, byte_width: usize, bits: u64) -> bool {
        self.with_bytes_mut(|bytes| write_bits_to_bytes(bytes, false, index, byte_width, bits))
    }

    fn grow(&mut self, byte_length: usize) -> bool {
        if byte_length < self.byte_length || byte_length > MAX_BACKING_STORE_BYTE_LENGTH {
            return false;
        }
        let grew = self.with_bytes_mut(|bytes| {
            if byte_length > bytes.capacity()
                && bytes
                    .try_reserve_exact(byte_length.saturating_sub(bytes.len()))
                    .is_err()
            {
                return false;
            }
            bytes.resize(byte_length, 0);
            true
        });
        if grew {
            self.byte_length = byte_length;
        }
        grew
    }

    fn atomic_load_bits(&self, index: usize, byte_width: usize) -> Option<u64> {
        self.with_bytes(|bytes| read_bits_from_bytes(bytes, false, index, byte_width))
    }

    fn atomic_store_bits(&self, index: usize, byte_width: usize, bits: u64) -> bool {
        self.with_bytes_mut(|bytes| write_bits_to_bytes(bytes, false, index, byte_width, bits))
    }

    fn atomic_compare_exchange_bits(
        &self,
        index: usize,
        byte_width: usize,
        expected: u64,
        replacement: u64,
    ) -> Option<u64> {
        self.with_bytes_mut(|bytes| {
            let current = read_bits_from_bytes(bytes, false, index, byte_width)?;
            if current == expected {
                let _ = write_bits_to_bytes(bytes, false, index, byte_width, replacement);
            }
            Some(current)
        })
    }

    fn with_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> R {
        let bytes = self
            .bytes
            .lock()
            .expect("shared backing-store mutex poisoned");
        f(&bytes)
    }

    fn with_bytes_mut<R>(&self, f: impl FnOnce(&mut Vec<u8>) -> R) -> R {
        let mut bytes = self
            .bytes
            .lock()
            .expect("shared backing-store mutex poisoned");
        f(&mut bytes)
    }
}

impl std::fmt::Debug for SharedBackingStoreRecord {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SharedBackingStoreRecord")
            .field("byte_length", &self.byte_length)
            .finish()
    }
}

impl PartialEq for SharedBackingStoreRecord {
    fn eq(&self, other: &Self) -> bool {
        self.byte_length == other.byte_length
            && self.with_bytes(<[u8]>::to_vec) == other.with_bytes(<[u8]>::to_vec)
    }
}

impl Eq for SharedBackingStoreRecord {}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BackingStoreRuntime {
    records: Vec<Option<BackingStoreRecord>>,
}

impl BackingStoreRuntime {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn allocate(&mut self, byte_length: usize) -> Option<BackingStoreRef> {
        self.allocate_record(BackingStoreRecord::new_local(byte_length)?)
    }

    pub(crate) fn allocate_shared(&mut self, byte_length: usize) -> Option<BackingStoreRef> {
        self.allocate_record(BackingStoreRecord::new_shared(byte_length)?)
    }

    pub(crate) fn byte_length(&self, id: BackingStoreRef) -> Option<usize> {
        Some(self.record(id)?.byte_length())
    }

    pub(crate) fn is_detached(&self, id: BackingStoreRef) -> Option<bool> {
        Some(self.record(id)?.is_detached())
    }

    pub(crate) fn is_shared(&self, id: BackingStoreRef) -> Option<bool> {
        Some(self.record(id)?.is_shared())
    }

    pub(crate) fn get_byte(&self, id: BackingStoreRef, index: usize) -> Option<u8> {
        self.record(id)?.get_byte(index)
    }

    pub(crate) fn set_byte(&mut self, id: BackingStoreRef, index: usize, value: u8) -> bool {
        let Some(record) = self.record_mut(id) else {
            return false;
        };
        record.set_byte(index, value)
    }

    pub(crate) fn load_bits(
        &self,
        id: BackingStoreRef,
        index: usize,
        byte_width: usize,
    ) -> Option<u64> {
        self.record(id)?.load_bits(index, byte_width)
    }

    pub(crate) fn store_bits(
        &mut self,
        id: BackingStoreRef,
        index: usize,
        byte_width: usize,
        bits: u64,
    ) -> bool {
        let Some(record) = self.record_mut(id) else {
            return false;
        };
        record.store_bits(index, byte_width, bits)
    }

    pub(crate) fn resize(&mut self, id: BackingStoreRef, byte_length: usize) -> bool {
        let Some(record) = self.record_mut(id) else {
            return false;
        };
        record.resize(byte_length)
    }

    pub(crate) fn grow_shared(&mut self, id: BackingStoreRef, byte_length: usize) -> bool {
        let Some(record) = self.record_mut(id) else {
            return false;
        };
        record.grow_shared(byte_length)
    }

    pub(crate) fn atomic_load_bits(
        &self,
        id: BackingStoreRef,
        index: usize,
        byte_width: usize,
    ) -> Option<u64> {
        self.record(id)?.atomic_load_bits(index, byte_width)
    }

    pub(crate) fn atomic_store_bits(
        &mut self,
        id: BackingStoreRef,
        index: usize,
        byte_width: usize,
        bits: u64,
    ) -> bool {
        let Some(record) = self.record_mut(id) else {
            return false;
        };
        record.atomic_store_bits(index, byte_width, bits)
    }

    pub(crate) fn atomic_compare_exchange_bits(
        &mut self,
        id: BackingStoreRef,
        index: usize,
        byte_width: usize,
        expected: u64,
        replacement: u64,
    ) -> Option<u64> {
        self.record_mut(id)?
            .atomic_compare_exchange_bits(index, byte_width, expected, replacement)
    }

    pub(crate) fn detach(&mut self, id: BackingStoreRef) -> bool {
        let Some(record) = self.record_mut(id) else {
            return false;
        };
        if record.is_shared() {
            return false;
        }
        let BackingStoreRecord::Local(record) = record else {
            return false;
        };
        if record.detached {
            return true;
        }
        record.bytes.clear();
        record.detached = true;
        true
    }

    pub(crate) fn clone_range(
        &mut self,
        id: BackingStoreRef,
        start: usize,
        end: usize,
    ) -> Option<BackingStoreRef> {
        let cloned = {
            let record = self.record(id)?;
            let BackingStoreRecord::Local(record) = record else {
                return None;
            };
            if record.detached || start > end || end > record.bytes.len() {
                return None;
            }
            record.bytes[start..end].to_vec()
        };
        self.allocate_record(BackingStoreRecord::Local(LocalBackingStoreRecord {
            bytes: cloned,
            detached: false,
        }))
    }

    pub(crate) fn accounting(&self) -> RuntimeDomainAccounting {
        let records = self.records.iter().flatten().count();
        let metadata_bytes = records * size_of::<BackingStoreRecord>();
        let payload_bytes = self
            .records
            .iter()
            .flatten()
            .map(BackingStoreRecord::byte_length)
            .sum::<usize>();
        RuntimeDomainAccounting {
            records,
            metadata_bytes,
            payload_bytes,
            live_bytes: metadata_bytes + payload_bytes,
        }
    }

    fn allocate_record(&mut self, record: BackingStoreRecord) -> Option<BackingStoreRef> {
        let raw = u32::try_from(self.records.len() + 1).ok()?;
        let id = BackingStoreRef::from_raw(raw)?;
        self.records.push(Some(record));
        Some(id)
    }

    fn record(&self, id: BackingStoreRef) -> Option<&BackingStoreRecord> {
        self.records.get(backing_store_index(id))?.as_ref()
    }

    fn record_mut(&mut self, id: BackingStoreRef) -> Option<&mut BackingStoreRecord> {
        self.records.get_mut(backing_store_index(id))?.as_mut()
    }
}

fn backing_store_index(id: BackingStoreRef) -> usize {
    usize::try_from(id.get().saturating_sub(1)).expect("backing-store index should fit into usize")
}

fn zeroed_backing_store_bytes(byte_length: usize) -> Option<Vec<u8>> {
    if byte_length > MAX_BACKING_STORE_BYTE_LENGTH {
        return None;
    }
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(byte_length).ok()?;
    bytes.resize(byte_length, 0);
    Some(bytes)
}

fn read_bits_from_bytes(
    bytes: &[u8],
    detached: bool,
    index: usize,
    byte_width: usize,
) -> Option<u64> {
    if detached || !(1..=8).contains(&byte_width) {
        return None;
    }
    let end = index.checked_add(byte_width)?;
    if end > bytes.len() {
        return None;
    }
    let mut bits = 0_u64;
    for (offset, byte) in bytes[index..end].iter().copied().enumerate() {
        bits |= u64::from(byte) << (offset * 8);
    }
    Some(bits)
}

fn write_bits_to_bytes(
    bytes: &mut [u8],
    detached: bool,
    index: usize,
    byte_width: usize,
    bits: u64,
) -> bool {
    if detached || !(1..=8).contains(&byte_width) {
        return false;
    }
    let Some(end) = index.checked_add(byte_width) else {
        return false;
    };
    if end > bytes.len() {
        return false;
    }
    for offset in 0..byte_width {
        bytes[index + offset] =
            u8::try_from((bits >> (offset * 8)) & 0xff).expect("masked element byte should fit");
    }
    true
}
