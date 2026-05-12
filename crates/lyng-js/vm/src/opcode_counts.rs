use std::cell::Cell;

use lyng_js_bytecode::{Opcode, OPCODE_COUNT};

const OPCODE_COUNT_LEN: usize = OPCODE_COUNT as usize;

pub struct OpcodeDispatchCounterStore {
    counts: Box<[Cell<u64>]>,
}

impl OpcodeDispatchCounterStore {
    pub fn new() -> Self {
        Self {
            counts: (0..OPCODE_COUNT_LEN)
                .map(|_| Cell::new(0))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    #[inline]
    pub fn increment(&self, opcode: Opcode) {
        let slot = &self.counts[usize::from(opcode as u8)];
        slot.set(slot.get().saturating_add(1));
    }

    pub fn reset(&self) {
        for slot in &self.counts {
            slot.set(0);
        }
    }

    pub fn snapshot(&self) -> OpcodeDispatchCounts {
        OpcodeDispatchCounts {
            counts: self.counts.iter().map(Cell::get).collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpcodeDispatchCounts {
    counts: Vec<u64>,
}

impl OpcodeDispatchCounts {
    #[must_use]
    pub fn from_counts<I>(counts: I) -> Self
    where
        I: IntoIterator<Item = (Opcode, u64)>,
    {
        let mut snapshot = Self::zeroed();
        for (opcode, count) in counts {
            snapshot.counts[usize::from(opcode as u8)] =
                snapshot.counts[usize::from(opcode as u8)].saturating_add(count);
        }
        snapshot
    }

    #[must_use]
    pub fn count(&self, opcode: Opcode) -> u64 {
        self.counts
            .get(usize::from(opcode as u8))
            .copied()
            .unwrap_or(0)
    }

    #[must_use]
    pub fn total(&self) -> u64 {
        self.counts
            .iter()
            .fold(0_u64, |total, count| total.saturating_add(*count))
    }

    pub fn iter(&self) -> impl Iterator<Item = OpcodeDispatchCount> + '_ {
        self.counts.iter().enumerate().filter_map(|(index, count)| {
            let raw = u8::try_from(index).ok()?;
            Some(OpcodeDispatchCount {
                opcode: Opcode::from_byte(raw)?,
                count: *count,
            })
        })
    }

    #[must_use]
    pub fn top(&self, limit: usize) -> Vec<OpcodeDispatchCount> {
        let mut counts = self
            .iter()
            .filter(|entry| entry.count() != 0)
            .collect::<Vec<_>>();
        counts.sort_unstable_by(|left, right| {
            right
                .count()
                .cmp(&left.count())
                .then_with(|| left.opcode().name().cmp(right.opcode().name()))
        });
        counts.truncate(limit);
        counts
    }

    fn zeroed() -> Self {
        Self {
            counts: vec![0; OPCODE_COUNT_LEN],
        }
    }
}

impl Default for OpcodeDispatchCounts {
    fn default() -> Self {
        Self::zeroed()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpcodeDispatchCount {
    opcode: Opcode,
    count: u64,
}

impl OpcodeDispatchCount {
    #[inline]
    pub const fn opcode(self) -> Opcode {
        self.opcode
    }

    #[inline]
    pub const fn count(self) -> u64 {
        self.count
    }
}
