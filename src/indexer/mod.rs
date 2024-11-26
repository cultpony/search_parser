mod rbm;
mod quickmap;

/// TEOs are operations on the index that can be executed in a massively parallel fashion
/// by allowing each bitmap index to operate independently
/// 
/// Examples:
/// 
/// To look for all objects with the tags 0x1A and 0x9F, the following TEOs would execute:
/// 
///   PushRelation
///   PushFilter 0x1A
///   And
///   ExitWithRel 0x9F
/// 
/// Look for all objects with the tag 0x1A
/// 
///   PushRelation
///   ExitWithRel 0x1A
/// 
/// Look for all objects with the tag 0x1A and either 0xA2 or 2C:
/// 
///   PushRelation
///   PushFilter 0x1A
///   And
///   PushFilter 0xA2
///   PushFilter 0x2C
///   Or
///   And
///   ExitAllRel
/// 
/// Look for all objects with tag 0x1A and not tag 0xA2 or with tag 0x2C and not tag 0x43
/// -> first, optimize into AND relation
/// => not tag 0x1A and 
/// 
///   PushRelation
///   PushFilter 0x1A
///   PushFilter 0xA2
///   And
///   PushFilter 0x2C
///   And
///   PushFilter 0x43
///   

pub enum TORExprOperator {
    /// Push the current relation on the stack
    PushRelation,
    /// Push a full relation to the stack (all relations are true)
    PushFull,
    /// Push an empty relation to the stack (no relations are true)
    PushEmpty,
    /// pushes a filter for a specific tag into the stack
    PushFilter{ tag: u32 },
    /// pushes a filter for a specific object into the stack
    PushObject{ obj: u32 },
    /// XOR the two top frames
    Xor,
    /// AND the two top frames
    And,
    /// OR the two top frames
    Or,
    /// Invert the top frame
    Not,
    /// Duplicate top frame
    Dup,
    /// Pop top frame and grab all relations in it, this is the default operation
    /// but slightly more expensive
    ExitAllRel,
    /// Pop top frame and grabs all relations with this relationpartner in it
    /// Note that this means when in OBJ mode, it returns TAG, if in TAG
    /// mode, it returns OBJ
    ExitWithRel{ rel: u32 },
}