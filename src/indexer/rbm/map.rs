use crate::indexer::quickmap::{QuickIntMap, QuickMap};

use super::TagObjectRelation;


pub struct MappedTagObjectRelation<const T: usize, const O: usize> {
    rel: TagObjectRelation<T, O>,
    tag_map: QuickMap<32>,
    obj_map: QuickIntMap<32>,
}