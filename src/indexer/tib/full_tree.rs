use std::collections::BTreeMap;

use crate::indexer::{DeleteResult, InsertResult, QueryResult};

use super::TagIndex;


pub struct FullTIBTree {
    tree: BTreeMap<u32, TagIndex>,
}

impl super::super::Index for FullTIBTree {
    fn insert(&mut self, obj: u32, tag: u32) -> crate::indexer::InsertResult {
        if let Some(index) = self.tree.get_mut(&tag) {
            index.insert(obj, tag)
        } else {
            let mut new_index = TagIndex::new(tag);
            new_index.insert(obj, tag);
            self.tree.insert(tag, new_index);
            InsertResult::Inserted
        }
    }

    fn delete(&mut self, obj: u32, tag: u32) -> crate::indexer::DeleteResult {
        if let Some(index) = self.tree.get_mut(&tag) {
            index.delete(obj, tag);
            if index.bitmap.is_empty() {
                drop(index);
                self.tree.remove(&tag);
            }
            DeleteResult::Deleted
        } else {
            DeleteResult::Ignored
        }
    }

    fn query_obj(&self, obj: u32) -> crate::indexer::QueryResult {
        todo!()
    }

    fn query_tag(&self, tag: u32) -> crate::indexer::QueryResult {
        if self.tree.contains_key(&tag) {
            QueryResult::Match
        } else {
            QueryResult::NoMatch
        }
    }

    fn query_rel(&self, obj: u32, tag: u32) -> crate::indexer::QueryResult {
        if let Some(index) = self.tree.get(&tag) {
            index.query_rel(obj, tag)
        } else {
            QueryResult::NoMatch
        }
    }
}