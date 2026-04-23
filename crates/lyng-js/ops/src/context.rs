use lyng_js_common::AtomTable;
use lyng_js_gc::{PrimitiveHeap, PrimitiveHeapView, PrimitiveMutator};

/// Concrete borrowed primitive-runtime facade used by allocating operations.
pub struct PrimitiveContext<'a> {
    heap: &'a mut PrimitiveHeap,
    atoms: &'a mut AtomTable,
}

impl<'a> PrimitiveContext<'a> {
    #[inline]
    pub fn new(heap: &'a mut PrimitiveHeap, atoms: &'a mut AtomTable) -> Self {
        Self { heap, atoms }
    }

    #[inline]
    pub fn heap(&self) -> PrimitiveHeapView<'_> {
        self.heap.view()
    }

    #[inline]
    pub fn mutator(&mut self) -> PrimitiveMutator<'_> {
        self.heap.mutator()
    }

    #[inline]
    pub fn atoms(&self) -> &AtomTable {
        self.atoms
    }

    #[inline]
    pub fn atoms_mut(&mut self) -> &mut AtomTable {
        self.atoms
    }

    #[inline]
    pub fn split_mut(&mut self) -> (PrimitiveMutator<'_>, &mut AtomTable) {
        let heap = &mut *self.heap;
        let atoms = &mut *self.atoms;
        (heap.mutator(), atoms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_gc::{AllocationLifetime, StringEncoding};

    #[test]
    fn primitive_context_exposes_explicit_heap_and_atom_borrows() {
        let mut heap = PrimitiveHeap::new();
        let mut atoms = AtomTable::new();
        let mut context = PrimitiveContext::new(&mut heap, &mut atoms);

        let string = context.mutator().alloc_string(
            StringEncoding::Latin1,
            4,
            b"ctx!",
            None,
            AllocationLifetime::Default,
        );
        let atom = context.atoms_mut().intern_collectible("ctx-atom");

        assert_eq!(context.heap().string_payload(string), Some(&b"ctx!"[..]));
        assert_eq!(context.atoms().resolve(atom), "ctx-atom");
        {
            let (mut mutator, atoms) = context.split_mut();
            assert_eq!(atoms.resolve(atom), "ctx-atom");
            assert!(mutator.memoize_string_atom(string, atom));
        }
        assert_eq!(
            context.heap().string(string).unwrap().cached_atom(),
            Some(atom)
        );
    }
}
