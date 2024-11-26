
mod full_tree;

pub struct TagIndex {
    tag: u32,
    bitmap: roaring::RoaringBitmap,
}

impl TagIndex {
    pub fn new(tag: u32) -> Self {
        Self {
            tag,
            bitmap: roaring::RoaringBitmap::new(),
        }
    }
}

impl super::Index for TagIndex {
    fn insert(&mut self, obj: u32, tag: u32) -> super::InsertResult {
        if tag == self.tag {
            self.bitmap.insert(obj);
            super::InsertResult::Inserted
        } else {
            super::InsertResult::Ignored
        }
    }

    fn delete(&mut self, obj: u32, tag: u32) -> super::DeleteResult {
        if tag == self.tag {
            self.bitmap.remove(obj);
            super::DeleteResult::Deleted
        } else {
            super::DeleteResult::Ignored
        }
    }

    fn query_obj(&self, obj: u32) -> super::QueryResult {
        if self.bitmap.contains(obj) {
            super::QueryResult::Match
        } else {
            super::QueryResult::NoMatch
        }
    }

    fn query_tag(&self, tag: u32) -> super::QueryResult {
        if tag == self.tag {
            super::QueryResult::Match
        } else {
            super::QueryResult::NoMatch
        }
    }

    fn query_rel(&self, obj: u32, tag: u32) -> super::QueryResult {
        if tag == self.tag {
            self.query_obj(obj)
        } else {
            super::QueryResult::NoMatch
        }
    }
}