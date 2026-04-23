//! Atom identifiers and the shared atom table.
//!
//! Atoms are interned strings used for identifiers, property names, and
//! well-known strings. The atom namespace is shared between frontend,
//! compiler, and runtime via `AtomId` and `AtomTable`.
//!
//! Frontend-created atoms are permanent by default. Runtime atomization uses
//! an explicit collectible entrypoint while sharing the same `AtomId`
//! namespace and deduplication table.

use hashbrown::HashMap;
use std::{fmt, mem::size_of};

/// A compact, copyable atom identifier.
///
/// Atom IDs are indices into the `AtomTable`. The zero value is reserved
/// as an invalid/empty sentinel so that `Option<AtomId>` is compact.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AtomId(u32);

impl AtomId {
    /// Creates an `AtomId` from a raw index. The caller must ensure
    /// this index is valid in the owning `AtomTable`.
    #[inline]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw index.
    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl fmt::Debug for AtomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AtomId({})", self.0)
    }
}

/// Lifetime class for an atom in the shared `AtomId` namespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AtomLifetime {
    Permanent,
    Collectible,
}

/// Summary of one collectible-atom sweep.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AtomSweepStats {
    pub reclaimed_collectible: usize,
    pub retained_collectible: usize,
    pub permanent_atoms: usize,
}

/// Well-known atom constants for keywords and commonly used identifiers.
///
/// These are pre-interned in every `AtomTable` and have stable `AtomId` values
/// so they can be compared by ID without table lookup.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum WellKnownAtom {
    // === Empty / sentinel ===
    /// The empty string atom. Always index 0.
    Empty = 0,

    // === Keywords (ECMA-262 §12.6.2) ===
    r#await = 1,
    r#break = 2,
    case = 3,
    catch = 4,
    class = 5,
    r#const = 6,
    r#continue = 7,
    debugger = 8,
    default = 9,
    delete = 10,
    do_ = 11,
    r#else = 12,
    r#enum = 13,
    export = 14,
    extends = 15,
    r#false = 16,
    finally = 17,
    r#for = 18,
    function = 19,
    r#if = 20,
    import = 21,
    in_ = 22,
    instanceof = 23,
    new = 24,
    null = 25,
    r#return = 26,
    super_ = 27,
    switch = 28,
    this = 29,
    throw = 30,
    r#true = 31,
    r#try = 32,
    typeof_ = 33,
    var = 34,
    void_ = 35,
    r#while = 36,
    with = 37,
    yield_ = 38,

    // === Strict-mode reserved words ===
    implements = 39,
    interface = 40,
    let_ = 41,
    package = 42,
    private = 43,
    protected = 44,
    public = 45,
    r#static = 46,

    // === Contextual keywords ===
    as_ = 47,
    async_ = 48,
    from = 49,
    get = 50,
    meta = 51,
    of = 52,
    set = 53,
    target = 54,
    accessor = 55,
    using = 56,

    // === Common identifiers ===
    arguments = 57,
    eval = 58,
    undefined = 59,
    constructor = 60,
    prototype = 61,
    length = 62,
    name = 63,
    value = 64,
    writable = 65,
    enumerable = 66,
    configurable = 67,
    toString = 68,
    valueOf = 69,
    apply = 70,
    call = 71,
    bind = 72,

    // === Well-known symbol description atoms ===
    __proto__ = 73,

    // === Sentinel: must be last ===
    __LAST = 74,
}

impl WellKnownAtom {
    /// Returns the `AtomId` for this well-known atom.
    #[inline]
    pub const fn id(self) -> AtomId {
        AtomId(self as u32)
    }

    /// Returns the source-text string for this well-known atom.
    pub const fn as_str(self) -> &'static str {
        WELL_KNOWN_ATOM_STRINGS[self as usize]
    }
}

/// The string values for each well-known atom, indexed by discriminant.
const WELL_KNOWN_ATOM_STRINGS: &[&str] = &[
    "",             // Empty
    "await",        // await
    "break",        // break
    "case",         // case
    "catch",        // catch
    "class",        // class
    "const",        // const
    "continue",     // continue
    "debugger",     // debugger
    "default",      // default
    "delete",       // delete
    "do",           // do_
    "else",         // else
    "enum",         // enum
    "export",       // export
    "extends",      // extends
    "false",        // false
    "finally",      // finally
    "for",          // for
    "function",     // function
    "if",           // if
    "import",       // import
    "in",           // in_
    "instanceof",   // instanceof
    "new",          // new
    "null",         // null
    "return",       // return
    "super",        // super
    "switch",       // switch
    "this",         // this
    "throw",        // throw
    "true",         // true
    "try",          // try
    "typeof",       // typeof_
    "var",          // var
    "void",         // void_
    "while",        // while
    "with",         // with
    "yield",        // yield_
    "implements",   // implements
    "interface",    // interface
    "let",          // let_
    "package",      // package
    "private",      // private
    "protected",    // protected
    "public",       // public
    "static",       // static
    "as",           // as_
    "async",        // async_
    "from",         // from
    "get",          // get
    "meta",         // meta
    "of",           // of
    "set",          // set
    "target",       // target
    "accessor",     // accessor
    "using",        // using
    "arguments",    // arguments
    "eval",         // eval
    "undefined",    // undefined
    "constructor",  // constructor
    "prototype",    // prototype
    "length",       // length
    "name",         // name
    "value",        // value
    "writable",     // writable
    "enumerable",   // enumerable
    "configurable", // configurable
    "toString",     // toString
    "valueOf",      // valueOf
    "apply",        // apply
    "call",         // call
    "bind",         // bind
    "__proto__",    // __proto__
    "",             // __LAST (sentinel, not a real atom)
];

/// All well-known atoms as a static slice for iteration.
pub const WELL_KNOWN_ATOMS: &[WellKnownAtom] = &[
    WellKnownAtom::Empty,
    WellKnownAtom::r#await,
    WellKnownAtom::r#break,
    WellKnownAtom::case,
    WellKnownAtom::catch,
    WellKnownAtom::class,
    WellKnownAtom::r#const,
    WellKnownAtom::r#continue,
    WellKnownAtom::debugger,
    WellKnownAtom::default,
    WellKnownAtom::delete,
    WellKnownAtom::do_,
    WellKnownAtom::r#else,
    WellKnownAtom::r#enum,
    WellKnownAtom::export,
    WellKnownAtom::extends,
    WellKnownAtom::r#false,
    WellKnownAtom::finally,
    WellKnownAtom::r#for,
    WellKnownAtom::function,
    WellKnownAtom::r#if,
    WellKnownAtom::import,
    WellKnownAtom::in_,
    WellKnownAtom::instanceof,
    WellKnownAtom::new,
    WellKnownAtom::null,
    WellKnownAtom::r#return,
    WellKnownAtom::super_,
    WellKnownAtom::switch,
    WellKnownAtom::this,
    WellKnownAtom::throw,
    WellKnownAtom::r#true,
    WellKnownAtom::r#try,
    WellKnownAtom::typeof_,
    WellKnownAtom::var,
    WellKnownAtom::void_,
    WellKnownAtom::r#while,
    WellKnownAtom::with,
    WellKnownAtom::yield_,
    WellKnownAtom::implements,
    WellKnownAtom::interface,
    WellKnownAtom::let_,
    WellKnownAtom::package,
    WellKnownAtom::private,
    WellKnownAtom::protected,
    WellKnownAtom::public,
    WellKnownAtom::r#static,
    WellKnownAtom::as_,
    WellKnownAtom::async_,
    WellKnownAtom::from,
    WellKnownAtom::get,
    WellKnownAtom::meta,
    WellKnownAtom::of,
    WellKnownAtom::set,
    WellKnownAtom::target,
    WellKnownAtom::accessor,
    WellKnownAtom::using,
    WellKnownAtom::arguments,
    WellKnownAtom::eval,
    WellKnownAtom::undefined,
    WellKnownAtom::constructor,
    WellKnownAtom::prototype,
    WellKnownAtom::length,
    WellKnownAtom::name,
    WellKnownAtom::value,
    WellKnownAtom::writable,
    WellKnownAtom::enumerable,
    WellKnownAtom::configurable,
    WellKnownAtom::toString,
    WellKnownAtom::valueOf,
    WellKnownAtom::apply,
    WellKnownAtom::call,
    WellKnownAtom::bind,
    WellKnownAtom::__proto__,
];

/// The shared atom namespace used by frontend, compiler, and runtime.
///
/// Pre-populated with well-known atoms at construction. New atoms are
/// interned on demand, with frontend-safe permanent interning kept as the
/// default and runtime collectible interning exposed explicitly.
///
/// Strings are stored in contiguous UTF-8 and UTF-16 arena buffers. Frontend
/// atoms stay UTF-8-resolvable, while runtime atomization can preserve raw
/// UTF-16 code-unit sequences such as lone surrogates.
pub struct AtomTable {
    /// Contiguous buffer holding UTF-8-resolvable atom strings end-to-end.
    utf8_buffer: String,
    /// Contiguous buffer holding UTF-16-only atom strings end-to-end.
    utf16_buffer: Vec<u16>,
    /// Offsets, storage kinds, and lifetime classes for each atom, indexed by `AtomId`.
    entries: Vec<AtomEntry>,
    /// Reverse lookup for UTF-8 frontend atoms and scalar-valid runtime atoms.
    utf8_lookup: HashMap<Box<str>, AtomId>,
    /// Canonical reverse lookup over UTF-16 code-unit sequences.
    utf16_lookup: HashMap<Box<[u16]>, AtomId>,
    /// Reclaimed collectible atom slots available for reuse.
    free_list: Vec<u32>,
    /// Number of currently live atoms, including permanent well-known atoms.
    live_len: usize,
}

#[derive(Clone, Copy, Debug)]
enum AtomEntry {
    Occupied(OccupiedAtomEntry),
    Vacant,
}

#[derive(Clone, Copy, Debug)]
struct OccupiedAtomEntry {
    storage: AtomStorage,
    lifetime: AtomLifetime,
    marked: bool,
}

#[derive(Clone, Copy, Debug)]
enum AtomStorage {
    Utf8 { offset: u32, len: u32 },
    Utf16 { offset: u32, len: u32 },
}

impl OccupiedAtomEntry {
    #[inline]
    const fn new(storage: AtomStorage, lifetime: AtomLifetime) -> Self {
        Self {
            storage,
            lifetime,
            marked: true,
        }
    }
}

/// A collection session over an `AtomTable`.
///
/// `lyng_js_gc` drives the session by visiting `AtomId` edges during its
/// ordinary mark walk and then calling `sweep` once tracing is complete.
pub struct AtomCollection<'a> {
    table: &'a mut AtomTable,
}

impl AtomTable {
    /// Creates a new atom table pre-populated with all well-known atoms.
    pub fn new() -> Self {
        let count = WellKnownAtom::__LAST as usize;
        let mut utf8_buffer = String::new();
        let utf16_buffer = Vec::new();
        let mut entries = Vec::with_capacity(count + 64);
        let mut utf8_lookup = HashMap::with_capacity(count + 64);
        let mut utf16_lookup = HashMap::with_capacity(count + 64);

        for &atom in WELL_KNOWN_ATOMS {
            let s = atom.as_str();
            let id = AtomId(entries.len() as u32);
            debug_assert_eq!(id, atom.id());
            let offset = utf8_buffer.len() as u32;
            utf8_buffer.push_str(s);
            entries.push(AtomEntry::Occupied(OccupiedAtomEntry::new(
                AtomStorage::Utf8 {
                    offset,
                    len: s.len() as u32,
                },
                AtomLifetime::Permanent,
            )));
            utf8_lookup.insert(s.into(), id);
            utf16_lookup.insert(s.encode_utf16().collect::<Vec<_>>().into_boxed_slice(), id);
        }

        Self {
            utf8_buffer,
            utf16_buffer,
            entries,
            utf8_lookup,
            utf16_lookup,
            free_list: Vec::new(),
            live_len: count,
        }
    }

    /// Interns a string, returning its `AtomId`. If the string is already
    /// interned, returns the existing ID. This keeps the permanent lifetime
    /// semantics expected by frontend callers.
    pub fn intern(&mut self, s: &str) -> AtomId {
        self.intern_with_lifetime_str(s, AtomLifetime::Permanent)
    }

    /// Interns a string as a collectible runtime atom.
    pub fn intern_collectible(&mut self, s: &str) -> AtomId {
        self.intern_with_lifetime_str(s, AtomLifetime::Collectible)
    }

    /// Interns a UTF-16 code-unit sequence, preserving lone surrogates.
    pub fn intern_utf16(&mut self, units: &[u16]) -> AtomId {
        self.intern_with_lifetime_utf16(units, AtomLifetime::Permanent)
    }

    /// Interns a UTF-16 code-unit sequence as a collectible runtime atom.
    pub fn intern_collectible_utf16(&mut self, units: &[u16]) -> AtomId {
        self.intern_with_lifetime_utf16(units, AtomLifetime::Collectible)
    }

    fn intern_with_lifetime_str(&mut self, s: &str, lifetime: AtomLifetime) -> AtomId {
        if let Some(&id) = self.utf8_lookup.get(s) {
            if lifetime == AtomLifetime::Permanent {
                let _ = self.promote_to_permanent(id);
            }
            return id;
        }

        let utf16 = s.encode_utf16().collect::<Vec<_>>().into_boxed_slice();
        if let Some(&id) = self.utf16_lookup.get(utf16.as_ref()) {
            if lifetime == AtomLifetime::Permanent {
                let _ = self.promote_to_permanent(id);
            }
            return id;
        }

        self.insert_utf8_atom(s.into(), utf16, lifetime)
    }

    fn intern_with_lifetime_utf16(&mut self, units: &[u16], lifetime: AtomLifetime) -> AtomId {
        if let Some(&id) = self.utf16_lookup.get(units) {
            if lifetime == AtomLifetime::Permanent {
                let _ = self.promote_to_permanent(id);
            }
            return id;
        }

        match String::from_utf16(units) {
            Ok(text) => self.insert_utf8_atom(text.into_boxed_str(), units.into(), lifetime),
            Err(_) => self.insert_utf16_atom(units, lifetime),
        }
    }

    fn insert_utf8_atom(
        &mut self,
        text: Box<str>,
        utf16: Box<[u16]>,
        lifetime: AtomLifetime,
    ) -> AtomId {
        let id = self
            .free_list
            .pop()
            .map_or_else(|| AtomId(self.entries.len() as u32), AtomId);
        let offset = self.utf8_buffer.len() as u32;
        self.utf8_buffer.push_str(&text);
        let entry = AtomEntry::Occupied(OccupiedAtomEntry::new(
            AtomStorage::Utf8 {
                offset,
                len: text.len() as u32,
            },
            lifetime,
        ));
        if let Some(slot) = self.entries.get_mut(id.0 as usize) {
            *slot = entry;
        } else {
            self.entries.push(entry);
        }
        self.utf8_lookup.insert(text, id);
        self.utf16_lookup.insert(utf16, id);
        self.live_len += 1;
        id
    }

    fn insert_utf16_atom(&mut self, units: &[u16], lifetime: AtomLifetime) -> AtomId {
        let id = self
            .free_list
            .pop()
            .map_or_else(|| AtomId(self.entries.len() as u32), AtomId);
        let offset = self.utf16_buffer.len() as u32;
        self.utf16_buffer.extend_from_slice(units);
        let entry = AtomEntry::Occupied(OccupiedAtomEntry::new(
            AtomStorage::Utf16 {
                offset,
                len: units.len() as u32,
            },
            lifetime,
        ));
        if let Some(slot) = self.entries.get_mut(id.0 as usize) {
            *slot = entry;
        } else {
            self.entries.push(entry);
        }
        self.utf16_lookup.insert(units.into(), id);
        self.live_len += 1;
        id
    }

    /// Returns the lifetime class for an atom ID, or `None` if the ID is invalid.
    #[inline]
    pub fn lifetime(&self, id: AtomId) -> Option<AtomLifetime> {
        match self.entries.get(id.0 as usize) {
            Some(AtomEntry::Occupied(entry)) => Some(entry.lifetime),
            Some(AtomEntry::Vacant) | None => None,
        }
    }

    /// Promotes an atom to permanent lifetime. Returns `true` if the atom changed class.
    pub fn promote_to_permanent(&mut self, id: AtomId) -> bool {
        let Some(AtomEntry::Occupied(entry)) = self.entries.get_mut(id.0 as usize) else {
            return false;
        };

        if entry.lifetime == AtomLifetime::Permanent {
            return false;
        }

        entry.lifetime = AtomLifetime::Permanent;
        entry.marked = true;
        true
    }

    /// Starts a GC-driven collection session for collectible atoms.
    pub fn start_collection(&mut self) -> AtomCollection<'_> {
        for entry in &mut self.entries {
            if let AtomEntry::Occupied(entry) = entry {
                if entry.lifetime == AtomLifetime::Collectible {
                    entry.marked = false;
                }
            }
        }

        AtomCollection { table: self }
    }

    /// Returns the UTF-8-resolvable string for an atom ID, or `None` if the ID
    /// is invalid or only representable as raw UTF-16 code units.
    #[inline]
    pub fn get(&self, id: AtomId) -> Option<&str> {
        match self.entries.get(id.0 as usize) {
            Some(AtomEntry::Occupied(entry)) => match entry.storage {
                AtomStorage::Utf8 { offset, len } => {
                    Some(&self.utf8_buffer[offset as usize..offset as usize + len as usize])
                }
                AtomStorage::Utf16 { .. } => None,
            },
            Some(AtomEntry::Vacant) | None => None,
        }
    }

    /// Returns the raw UTF-16 code units for an atom ID when the atom is stored
    /// as UTF-16-only, or `None` when the ID is invalid or UTF-8-resolvable.
    #[inline]
    pub fn get_utf16(&self, id: AtomId) -> Option<&[u16]> {
        match self.entries.get(id.0 as usize) {
            Some(AtomEntry::Occupied(entry)) => match entry.storage {
                AtomStorage::Utf8 { .. } => None,
                AtomStorage::Utf16 { offset, len } => {
                    Some(&self.utf16_buffer[offset as usize..offset as usize + len as usize])
                }
            },
            Some(AtomEntry::Vacant) | None => None,
        }
    }

    /// Returns the UTF-8-resolvable string for an atom ID. Panics if the ID is
    /// invalid or only representable as raw UTF-16 code units.
    ///
    /// # Panics
    ///
    /// Panics if `id` is invalid or resolves to an atom that is only
    /// representable as raw UTF-16 code units.
    #[inline]
    pub fn resolve(&self, id: AtomId) -> &str {
        self.get(id)
            .expect("invalid atom id or atom requires UTF-16-only resolution")
    }

    /// Returns whether an atom matches a given UTF-16 code-unit sequence.
    pub fn matches_utf16(&self, id: AtomId, units: &[u16]) -> bool {
        let Some(AtomEntry::Occupied(entry)) = self.entries.get(id.0 as usize) else {
            return false;
        };

        match entry.storage {
            AtomStorage::Utf8 { offset, len } => self.utf8_buffer
                [offset as usize..offset as usize + len as usize]
                .encode_utf16()
                .eq(units.iter().copied()),
            AtomStorage::Utf16 { offset, len } => {
                self.utf16_buffer[offset as usize..offset as usize + len as usize] == *units
            }
        }
    }

    /// Returns the number of interned atoms.
    #[inline]
    pub fn len(&self) -> usize {
        self.live_len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.live_len == 0
    }

    /// Returns the live payload bytes held by the atom storage buffers.
    #[inline]
    pub fn payload_bytes(&self) -> usize {
        self.utf8_buffer.len() + self.utf16_buffer.len() * size_of::<u16>()
    }

    /// Looks up a keyword by name. Returns the well-known atom ID if the
    /// string is a keyword, `None` otherwise.
    ///
    /// This is used by the lexer to classify identifiers as keywords.
    pub fn keyword_atom(&self, s: &str) -> Option<AtomId> {
        // Only check the keyword range (1..=38 are keywords)
        if let Some(&id) = self.utf8_lookup.get(s) {
            if id.0 >= WellKnownAtom::r#await as u32 && id.0 <= WellKnownAtom::yield_ as u32 {
                return Some(id);
            }
        }
        None
    }
}

impl AtomCollection<'_> {
    /// Visits a live atom edge discovered during the GC mark walk.
    pub fn visit_atom(&mut self, id: AtomId) {
        let Some(AtomEntry::Occupied(entry)) = self.table.entries.get_mut(id.0 as usize) else {
            return;
        };

        if entry.lifetime == AtomLifetime::Collectible {
            entry.marked = true;
        }
    }

    /// Sweeps collectible atoms not visited during this collection session.
    pub fn sweep(self) -> AtomSweepStats {
        let AtomTable {
            utf8_buffer,
            utf16_buffer,
            entries,
            utf8_lookup,
            utf16_lookup,
            free_list,
            live_len,
        } = self.table;
        let old_utf8_buffer = std::mem::take(utf8_buffer);
        let old_utf16_buffer = std::mem::take(utf16_buffer);
        let mut new_utf8_buffer = String::with_capacity(old_utf8_buffer.len());
        let mut new_utf16_buffer = Vec::with_capacity(old_utf16_buffer.len());
        let mut stats = AtomSweepStats::default();

        for (index, entry) in entries.iter_mut().enumerate() {
            let AtomEntry::Occupied(atom) = entry else {
                continue;
            };

            if atom.lifetime == AtomLifetime::Collectible && !atom.marked {
                match atom.storage {
                    AtomStorage::Utf8 { offset, len } => {
                        let text =
                            &old_utf8_buffer[offset as usize..offset as usize + len as usize];
                        utf8_lookup.remove(text);
                        let utf16 = text.encode_utf16().collect::<Vec<_>>();
                        utf16_lookup.remove(utf16.as_slice());
                    }
                    AtomStorage::Utf16 { offset, len } => {
                        let units =
                            &old_utf16_buffer[offset as usize..offset as usize + len as usize];
                        utf16_lookup.remove(units);
                    }
                }
                *entry = AtomEntry::Vacant;
                free_list.push(index as u32);
                *live_len -= 1;
                stats.reclaimed_collectible += 1;
                continue;
            }

            atom.storage = match atom.storage {
                AtomStorage::Utf8 { offset, len } => {
                    let text = &old_utf8_buffer[offset as usize..offset as usize + len as usize];
                    let new_offset = new_utf8_buffer.len() as u32;
                    new_utf8_buffer.push_str(text);
                    AtomStorage::Utf8 {
                        offset: new_offset,
                        len,
                    }
                }
                AtomStorage::Utf16 { offset, len } => {
                    let units = &old_utf16_buffer[offset as usize..offset as usize + len as usize];
                    let new_offset = new_utf16_buffer.len() as u32;
                    new_utf16_buffer.extend_from_slice(units);
                    AtomStorage::Utf16 {
                        offset: new_offset,
                        len,
                    }
                }
            };
            atom.marked = true;

            match atom.lifetime {
                AtomLifetime::Permanent => stats.permanent_atoms += 1,
                AtomLifetime::Collectible => stats.retained_collectible += 1,
            }
        }

        *utf8_buffer = new_utf8_buffer;
        *utf16_buffer = new_utf16_buffer;
        stats
    }
}

impl Default for AtomTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_known_atoms_populated() {
        let table = AtomTable::new();
        assert_eq!(table.resolve(WellKnownAtom::r#break.id()), "break");
        assert_eq!(table.resolve(WellKnownAtom::function.id()), "function");
        assert_eq!(table.resolve(WellKnownAtom::Empty.id()), "");
    }

    #[test]
    fn intern_returns_existing() {
        let mut table = AtomTable::new();
        let id1 = table.intern("break");
        assert_eq!(id1, WellKnownAtom::r#break.id());
        let id2 = table.intern("break");
        assert_eq!(id1, id2);
    }

    #[test]
    fn intern_new_string() {
        let mut table = AtomTable::new();
        let id = table.intern("myVariable");
        assert_eq!(table.resolve(id), "myVariable");
        // Re-interning returns the same ID
        assert_eq!(table.intern("myVariable"), id);
    }

    #[test]
    fn keyword_lookup() {
        let table = AtomTable::new();
        assert!(table.keyword_atom("if").is_some());
        assert!(table.keyword_atom("while").is_some());
        assert!(table.keyword_atom("notAKeyword").is_none());
        // "let" is a strict-mode reserved word, not a keyword
        assert!(table.keyword_atom("let").is_none());
        // "async" is a contextual keyword, not in the keyword range
        assert!(table.keyword_atom("async").is_none());
    }

    #[test]
    fn well_known_atom_strings_consistent() {
        for &atom in WELL_KNOWN_ATOMS {
            let s = atom.as_str();
            let table = AtomTable::new();
            assert_eq!(table.resolve(atom.id()), s);
        }
    }

    #[test]
    fn well_known_atoms_are_permanent() {
        let table = AtomTable::new();

        for &atom in WELL_KNOWN_ATOMS {
            assert_eq!(table.lifetime(atom.id()), Some(AtomLifetime::Permanent));
        }
    }

    #[test]
    fn collectible_atoms_reuse_ids_and_keep_collectible_lifetime() {
        let mut table = AtomTable::new();

        let first = table.intern_collectible("dynamic-key");
        let second = table.intern_collectible("dynamic-key");

        assert_eq!(first, second);
        assert_eq!(table.lifetime(first), Some(AtomLifetime::Collectible));
    }

    #[test]
    fn permanent_intern_promotes_existing_collectible_atom() {
        let mut table = AtomTable::new();

        let id = table.intern_collectible("promoted-key");
        assert_eq!(table.lifetime(id), Some(AtomLifetime::Collectible));

        assert_eq!(table.intern("promoted-key"), id);
        assert_eq!(table.lifetime(id), Some(AtomLifetime::Permanent));
    }

    #[test]
    fn collectible_intern_reuses_existing_permanent_atom_without_demotion() {
        let mut table = AtomTable::new();

        let id = table.intern("frontend-name");
        assert_eq!(table.intern_collectible("frontend-name"), id);
        assert_eq!(table.lifetime(id), Some(AtomLifetime::Permanent));
    }

    #[test]
    fn collection_preserves_marked_collectible_atoms() {
        let mut table = AtomTable::new();
        let live = table.intern_collectible("live-key");

        let mut collection = table.start_collection();
        collection.visit_atom(live);
        let stats = collection.sweep();

        assert_eq!(
            stats,
            AtomSweepStats {
                reclaimed_collectible: 0,
                retained_collectible: 1,
                permanent_atoms: WELL_KNOWN_ATOMS.len(),
            }
        );
        assert_eq!(table.resolve(live), "live-key");
        assert_eq!(table.lifetime(live), Some(AtomLifetime::Collectible));
    }

    #[test]
    fn collection_reclaims_unvisited_collectible_atoms() {
        let mut table = AtomTable::new();
        let dead = table.intern_collectible("dead-key");

        let stats = table.start_collection().sweep();

        assert_eq!(stats.reclaimed_collectible, 1);
        assert_eq!(table.get(dead), None);
        assert_eq!(table.lifetime(dead), None);
    }

    #[test]
    fn reclaimed_collectible_slot_is_reused() {
        let mut table = AtomTable::new();
        let dead = table.intern_collectible("dead-key");
        let _ = table.start_collection().sweep();

        let replacement = table.intern_collectible("replacement-key");

        assert_eq!(replacement, dead);
        assert_eq!(table.resolve(replacement), "replacement-key");
    }

    #[test]
    fn scalar_valid_utf16_runtime_atoms_dedupe_with_utf8_atoms() {
        let mut table = AtomTable::new();

        let permanent = table.intern("Omega");
        let runtime = table.intern_collectible_utf16(&[
            u16::from(b'O'),
            u16::from(b'm'),
            u16::from(b'e'),
            u16::from(b'g'),
            u16::from(b'a'),
        ]);

        assert_eq!(runtime, permanent);
        assert_eq!(table.lifetime(runtime), Some(AtomLifetime::Permanent));
        assert_eq!(table.resolve(runtime), "Omega");
    }

    #[test]
    fn utf16_runtime_atoms_preserve_lone_surrogates() {
        let mut table = AtomTable::new();

        let first = table.intern_collectible_utf16(&[0xD800]);
        let second = table.intern_collectible_utf16(&[0xD800]);

        assert_eq!(first, second);
        assert_eq!(table.get(first), None);
        assert_eq!(table.get_utf16(first), Some(&[0xD800][..]));
        assert!(table.matches_utf16(first, &[0xD800]));
        assert_eq!(table.lifetime(first), Some(AtomLifetime::Collectible));
    }

    #[test]
    fn utf16_only_collectible_atoms_sweep_and_reuse_slots() {
        let mut table = AtomTable::new();
        let dead = table.intern_collectible_utf16(&[0xD800]);

        let stats = table.start_collection().sweep();
        let replacement = table.intern_collectible_utf16(&[0xD801]);

        assert_eq!(stats.reclaimed_collectible, 1);
        assert_eq!(table.lifetime(dead), Some(AtomLifetime::Collectible));
        assert_eq!(replacement, dead);
        assert!(table.matches_utf16(replacement, &[0xD801]));
    }

    #[test]
    fn payload_bytes_tracks_utf8_and_utf16_storage() {
        let mut table = AtomTable::new();
        let baseline = table.payload_bytes();

        table.intern("payload-utf8");
        table.intern_collectible_utf16(&[0xD800, 0xD801]);

        assert_eq!(
            table.payload_bytes(),
            baseline + "payload-utf8".len() + 2 * size_of::<u16>()
        );
    }
}
