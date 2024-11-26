mod map;

#[derive(Clone, PartialEq, Debug)]
#[repr(transparent)]
struct TagObjectRelation<const T: usize, const O: usize> {
    bitmap: roaring::RoaringBitmap,
}

impl<const T: usize, const O: usize> Default for TagObjectRelation<T, O> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<const T: usize, const O: usize> std::ops::BitAnd<Self> for TagObjectRelation<T, O> {
    type Output = Self;

    fn bitand(self, rhs: TagObjectRelation<T, O>) -> Self::Output {
        Self {
            bitmap: self.bitmap & rhs.bitmap
        }
    }
}

impl<const T: usize, const O:usize> std::ops::Not for TagObjectRelation<T, O> {
    type Output = Self;

    fn not(self) -> Self::Output {
        self ^ Self::full()
    }
}

impl<const T: usize, const O: usize> std::ops::BitOr for TagObjectRelation<T, O> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            bitmap: self.bitmap | rhs.bitmap
        }
    }
}

impl<const T: usize, const O: usize> std::ops::BitXor for TagObjectRelation<T, O> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            bitmap: self.bitmap ^ rhs.bitmap
        }
    }
}

impl<const T: usize, const O: usize> TagObjectRelation<T, O> {
    // check we use no more than 32 bits
    const _NO_MORE_THAN_32_BITS: usize = usize::MAX - (32 + T + O);
    // check we use no less than 32 bits
    const _NEED_ATLEAST_32_BITS: usize = usize::MAX + (32 - T - O);

    const OBJ_MASK: u32 = (1<<O)-1;
    const TAG_MASK: u32 = (1<<(T+1))-1;
    const BIT_SIZE: usize = T+O;

    pub fn empty() -> Self {
        let _ = Self::_NO_MORE_THAN_32_BITS;
        let _ = Self::_NEED_ATLEAST_32_BITS;
        Self {
            bitmap: roaring::RoaringBitmap::new(),
        }
    }

    pub fn full() -> Self {
        let _ = Self::_NO_MORE_THAN_32_BITS;
        let _ = Self::_NEED_ATLEAST_32_BITS;
        Self {
            bitmap: roaring::RoaringBitmap::full(),
        }
    }

    pub fn with_tag(tag: u32) -> Self {
        let mut mask = roaring::RoaringBitmap::new();
        mask.insert_range(
            Self::tuple_to_id(tag, u32::MIN)
            ..
            Self::tuple_to_id(tag, u32::MAX )
        );
        Self { bitmap: mask }
    }

    pub fn with_obj(obj: u32) -> Self {
        let mut mask = roaring::RoaringBitmap::new();
        for i in 0..=Self::TAG_MASK {
            mask.insert(Self::tuple_to_id(i, obj));
        }
        let mask = mask ^ roaring::RoaringBitmap::full();
        Self { bitmap: mask }
    }

    pub(self) const fn tuple_to_id(tag: u32, obj: u32) -> u32 {
        (tag & Self::TAG_MASK) << O | (obj & Self::OBJ_MASK)
    }

    /// Returns a tuple of (tag, obj)
    pub(self) const fn id_to_tuple(id: u32) -> (u32, u32) {
        let tag = (id >> O) & Self::TAG_MASK;
        let obj = id & (Self::OBJ_MASK);
        (tag, obj)
    }

    pub fn len(&self) -> usize {
        self.bitmap.len() as usize
    }

    pub fn bytes(&self) -> usize {
        self.bitmap.serialized_size()
    }

    /// Create a tag-object relation
    pub fn set(&mut self, tag: u32, object: u32) {
        assert!(tag < 1<<T);
        assert!(object < 1<<O);
        self.bitmap.insert(Self::tuple_to_id(tag, object));
    }

    /// Check if the Tag-Object are in relation
    pub fn check(&self, tag: u32, object: u32) -> bool {
        let id = Self::tuple_to_id(tag, object);
        self.bitmap.contains(id)
    }

    /// Returns all objects in the relation
    /// This operation is somewhat expensive compared to loading tags of an object
    /// or looking for objects with a tag
    pub fn objects(&self) -> Vec<u32> {
        let mut result = roaring::RoaringBitmap::new();
        for id in &self.bitmap {
            let (_, obj) = Self::id_to_tuple(id);
            result.insert(obj);
        }
        result.iter().collect()
    }

    /// Clear the Tag-Object association
    pub fn unset(&mut self, tag: u32, object: u32) {
        let id = Self::tuple_to_id(tag, object);
        self.bitmap.remove(id);
    }

    /// Remove tag from relation bitmap
    pub fn clear_tag(&mut self, tag: u32) {
        let mut mask = roaring::RoaringBitmap::new();
        mask.insert_range(
            Self::tuple_to_id(tag, u32::MIN)
            ..
            Self::tuple_to_id(tag, u32::MAX )
        );
        let masked = (&self.bitmap) & mask;
        self.bitmap = masked;
    }

    /// Remove an obj from the relation
    pub fn clear_obj(&mut self, obj: u32) {
        let mut mask = roaring::RoaringBitmap::new();
        for i in 0..=Self::TAG_MASK {
            mask.insert(Self::tuple_to_id(i, obj));
        }
        let mask = mask ^ roaring::RoaringBitmap::full();
        let masked = (&self.bitmap) & mask;
        self.bitmap = masked;
    }

    /// Returns the tagged objects
    pub fn tagged(&self, tag: u32) -> Vec<u32> {
        let mut mask = roaring::RoaringBitmap::new();
        mask.insert_range(
            Self::tuple_to_id(tag, u32::MIN)
            ..
            Self::tuple_to_id(tag, u32::MAX )
        );
        let masked = (&self.bitmap) & mask;
        masked.iter().map(|x| Self::id_to_tuple(x).1).collect()
    }

    /// Returns a list of tags associated with the object
    pub fn object_tags(&self, obj: u32) -> Vec<u32> {
        let mut mask = roaring::RoaringBitmap::new();
        for i in 0..=Self::TAG_MASK {
            mask.insert(Self::tuple_to_id(i, obj));
        }
        let masked = (&self.bitmap) & mask;
        masked.iter().map(|x| Self::id_to_tuple(x).0).collect()
    }

    /// Counts how many objects a tag has
    pub fn tag_count(&self, tag: u32) -> u64 {
        let mut mask = roaring::RoaringBitmap::new();
        mask.insert_range(
            Self::tuple_to_id(tag, u32::MIN)
            ..
            Self::tuple_to_id(tag, u32::MAX)
        );
        self.bitmap.intersection_len(&mask)
    }

}

#[cfg(test)]
mod test {
    use super::TagObjectRelation;
    use tracing::info;

    #[test]
    pub fn test_id_mapping() {
        type TOR = TagObjectRelation<20, 12>;
        let tag_encoded = TOR::tuple_to_id(0x0FFFFF, 0);
        assert_eq!(0xFFFF_F000, tag_encoded, "Found {:08x}, wanted {:08x}", tag_encoded, 0xFFFF_F000u32);

        let tag_encoded = TOR::tuple_to_id(0xFFFFFF, 0);
        assert_eq!(0xFFFF_F000, tag_encoded, "Found {:08x}, wanted {:08x}", tag_encoded, 0xFFFF_F000u32);

        let obj_encoded = TOR::tuple_to_id(0, 0xFFF);
        assert_eq!(0x0000_0FFF, obj_encoded, "Found {:08x}, wanted {:08x}", obj_encoded, 0x0000_0FFFu32);

        let obj_encoded = TOR::tuple_to_id(0, 0xFFFF);
        assert_eq!(0x0000_0FFF, obj_encoded, "Found {:08x}, wanted {:08x}", obj_encoded, 0x0000_0FFFu32);

        let decoded = TOR::id_to_tuple(tag_encoded);
        assert_eq!((0xFFFFF, 0), decoded);
    }

    #[test]
    #[tracing_test::traced_test]
    pub fn test_tor_insert() {
        type TOR = TagObjectRelation<8, 24>;
        let mut tor = TOR::empty();

        tor.set(0xDD, 0xFF_FF_FF);
        tor.set(0x00, 0x00_00_00);
        tor.set(0xEE, 0x1A_FE_FE);

        assert!(tor.check(0xDD, 0xFF_FF_FF));
        assert!(tor.check(0x00, 0x00_00_00));
        assert!(tor.check(0xEE, 0x1A_FE_FE));

        let mut expected = Vec::new();
        for i in 0..0xFFFF {
            expected.push(0xA * i);
            tor.set(0x1F, 0xA * i);
            tor.set(0xA0, 0xA * i);
            tor.set(0xEA, 0xA * i);
            tor.set(0x99, 0xA * i);
        }
        info!("roaring bitmap with {} elements, using {} bytes", tor.len(), tor.bytes());
        assert_eq!(tor.tag_count(0x1F) as usize, expected.len());
        let tagged = tor.tagged(0x1F);
        assert_eq!(tagged.len(), 0xFFFF);
        assert_eq!(expected, tagged);
    }
}