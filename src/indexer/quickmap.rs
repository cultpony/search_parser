use std::collections::HashSet;

// Allows mapping a string to an integer with a given number of bits at most used
// U is the number of bits to be used
#[derive(Clone, Debug)]
pub struct QuickMap<const U: u32> {
    store: patricia_tree::GenericPatriciaMap<String, Entry>,
}

pub struct QuickIntMap<const U: u32> {
    store: patricia_tree::GenericPatriciaMap<Vec<u8>, Entry>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Entry {
    Allocated { id: u32 },
    Tombstone { id: u32 },
    Deallocated,
}

impl Entry {
    pub fn as_option(&self) -> Option<u32> {
        match self {
            Entry::Allocated { id } => Some(*id),
            Entry::Tombstone { id } => Some(*id),
            Entry::Deallocated => None,
        }
    }
}

impl Into<Option<u32>> for Entry {
    fn into(self) -> Option<u32> {
        self.as_option()
    }
}

impl<const U: u32> QuickIntMap<U> {
    const MAX_ID: u32 = (1<<(U+1))-1;

    fn alloc_free_id(&self) -> Option<u32> {
        let poss_free = (0..Self::MAX_ID).into_iter();
        let mut values: HashSet<u32> = self.store.values()
            .filter_map(Entry::as_option)
            .collect();
        let result = poss_free.filter(|x| {
            if values.contains(x) {
                values.remove(x);
                true
            } else {
                false
            }
        }).next()?;
        if result < Self::MAX_ID {
            return Some(result)
        }
        return None
    }

    pub fn allocate(&mut self, tag: u32) -> Option<u32> {
        let tag = tag.to_ne_bytes();

        if let Some(entry) = self.store.get(tag) {
            match entry {
                Entry::Allocated { id } => return Some(*id),
                Entry::Tombstone { id } => {
                    let id = *id;
                    self.store.insert(tag, Entry::Allocated { id });
                    return Some(id)
                },
                Entry::Deallocated => {},
            }
        }
        let Some(id) = self.alloc_free_id() else {
            return None;
        };
        self.store.insert(tag, Entry::Allocated { id });
        Some(id)
    }

    pub fn resolve(&self, tag: u32) -> Option<u32> {
        let Some(entry) = self.store.get(tag.to_ne_bytes()) else {
            return None
        };
        match entry {
            Entry::Allocated { id } => Some(*id),
            Entry::Tombstone { id: _ } => None,
            Entry::Deallocated => None,
        }
    }

    pub fn tombstone(&mut self, tag: u32) {
        let tag = tag.to_ne_bytes();
        let Some(entry) = self.store.get(tag) else {
            return
        };
        match entry {
            Entry::Allocated { id } => {
                let id = *id;
                self.store.insert(tag, Entry::Tombstone { id });
            },
            Entry::Tombstone { id: _ } => (),
            Entry::Deallocated => (),
        }
    }

    #[must_use = "must check if ID was not tombstoned"]
    pub fn deallocate(&mut self, tag: u32) -> Option<u32> {
        let tag = tag.to_ne_bytes();
        let Some(entry) = self.store.get(tag) else {
            return None
        };
        match entry {
            Entry::Tombstone { id } => {
                let _= id;
                self.store.insert(tag, Entry::Deallocated);
                None
            },
            Entry::Allocated { id } => Some(*id),
            Entry::Deallocated => None,
        }
    }
}

impl<const U: u32> QuickMap<U> {
    const MAX_ID: u32 = (1 << (U + 1)) - 1;

    pub fn new() -> Self {
        Self {
            store: GenericPatriciaMap::new(),
        }
    }

    fn alloc_free_id(&self) -> Option<u32> {
        let poss_free = (0..Self::MAX_ID).into_iter();
        let mut values: HashSet<u32> = self.store.values().filter_map(Entry::as_option).collect();
        let result = poss_free
            .filter(|x| {
                if values.contains(x) {
                    values.remove(x);
                    true
                } else {
                    false
                }
            })
            .next()?;
        if result < Self::MAX_ID {
            return Some(result);
        }
        return None;
    }

    /// Returns a new, free ID to be used in the map or nothing if no more IDs are free
    ///
    /// If the relation exists, returns it and reactivates it if it was tombstoned
    pub fn allocate<S: AsRef<str>>(&mut self, tag: S) -> Option<u32> {
        let tag: &str = tag.as_ref();
        if let Some(entry) = self.store.get(tag) {
            // check if it's tombstoned
            match entry {
                Entry::Allocated { id } => {
                    // Return previous allocation
                    return Some(*id);
                }
                Entry::Tombstone { id } => {
                    let id = *id;
                    // Reactivate the allocation
                    self.store.insert(tag, Entry::Allocated { id });
                    return Some(id);
                }
                Entry::Deallocated => {
                    // do nothing, we allocate as normal
                }
            }
        }
        let Some(id) = self.alloc_free_id() else {
            return None;
        };
        self.store.insert(tag, Entry::Allocated { id });
        Some(id)
    }

    /// Turns a given tag into it's ID, unless it's been tombstoned
    pub fn resolve<S: AsRef<str>>(&self, tag: S) -> Option<u32> {
        let Some(entry) = self.store.get(tag.as_ref()) else {
            return None;
        };
        match entry {
            Entry::Allocated { id } => Some(*id),
            Entry::Tombstone { id: _ } => None,
            Entry::Deallocated => None,
        }
    }

    pub fn tombstone<S: AsRef<str>>(&mut self, tag: S) {
        let Some(entry) = self.store.get(tag.as_ref()) else {
            return;
        };
        match entry {
            Entry::Allocated { id } => {
                let id = *id;
                self.store.insert(tag.as_ref(), Entry::Tombstone { id });
            }
            Entry::Tombstone { id: _ } => (),
            Entry::Deallocated => (),
        }
    }

    /// Removes a tombstoned entry and returns None.
    /// If the entry is already deallocated, it returns None.
    /// If the entry is not tombstoned, the tag ID is returned and the
    /// entry continues to exist
    #[must_use = "must check if ID was not tombstoned"]
    pub fn deallocate<S: AsRef<str>>(&mut self, tag: S) -> Option<u32> {
        let Some(entry) = self.store.get(tag.as_ref()) else {
            return None;
        };
        match entry {
            Entry::Tombstone { id } => {
                let _ = id;
                self.store.insert(tag.as_ref(), Entry::Deallocated);
                None
            }
            Entry::Allocated { id } => Some(*id),
            Entry::Deallocated => None,
        }
    }
}
