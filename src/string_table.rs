//! String table used for the interning of strings. Utilises fxhash in order to maintain a
//! hash table of Cows

use std::borrow::Cow;
use std::collections::HashMap;

/// This implementation of [StringTable] utilises fx hash for quick and dirty hashing of
/// values stored within the table.  This is not a cryptographically secure implementation,
/// but then it doesn't need to be in order to provide basic string interning functionality
/// within the parser
#[derive(Debug, Clone)]
pub struct StringTable<'a> {
    /// Use Cow semantics for internal storage
    table: HashMap<u32, Cow<'a, str>>,
}

impl<'a> StringTable<'a> {
    /// Create a new string table
    pub fn new() -> Self {
        StringTable {
            table: HashMap::new(),
        }
    }

    /// Checks whether a given value exists within the table
    pub fn contains(&self, hash: u32) -> bool {
        self.table.contains_key(&hash)
    }

    /// Add a new value to the table and return the hash value associated with it
    pub fn insert(&mut self, s: &str) -> u32 {
        let hash = fxhash::hash32(s);
        self.table
            .entry(hash)
            .or_insert_with(|| s.to_string().into());
        hash
    }

    /// Retrieve a value from the string table
    pub fn get(&mut self, hash: u32) -> Option<&Cow<str>> {
        self.table.get(&hash)
    }

    /// Remove a value from the table
    pub fn remove(&mut self, s: &str) -> Option<Cow<str>> {
        self.table.remove(&fxhash::hash32(s))
    }

    /// Obtain the hash value for a specific value
    pub fn hash(&self, s: &str) -> u32 {
        fxhash::hash32(s)
    }
}
