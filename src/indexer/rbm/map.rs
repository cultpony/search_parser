use super::TagObjectRelation;


pub struct MappedTagObjectRelation<const T: usize, const O: usize> {
    rel: TagObjectRelation<T, O>,
    tag_map: Map<String>,
    obj_map: IntMap,
}