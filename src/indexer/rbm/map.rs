use crate::indexer::{quickmap::{QuickIntMap, QuickMap}, TORExprOperator};

use super::TagObjectRelation;


pub struct MappedTagObjectRelation<const T: usize, const O: usize> {
    rel: TagObjectRelation<T, O>,
    tag_map: QuickMap<32>,
    obj_map: QuickIntMap<32>,
}

impl<const T: usize, const O: usize> MappedTagObjectRelation<T, O> {
    /// Executes a number of TOR expression operators and returns all objects
    /// that are still present at the end of the operation
    fn execute_teo_obj(&self, teos: &[TORExprOperator]) -> Option<Vec<u32>> {
        let mut stack = Vec::new();
        for teo in teos {
            match teo {
                TORExprOperator::PushRelation => {
                    stack.push(self.rel.clone())
                },
                TORExprOperator::Dup => {
                    let s = stack.pop()?;
                    stack.push(s);
                }
                TORExprOperator::PushFull => {
                    stack.push(TagObjectRelation::full())
                },
                TORExprOperator::PushEmpty => {
                    stack.push(TagObjectRelation::empty())
                },
                TORExprOperator::PushFilter { tag } => {
                    stack.push(TagObjectRelation::with_tag(*tag))
                },
                TORExprOperator::PushObject { obj } => {
                    stack.push(TagObjectRelation::with_obj(*obj))
                }
                TORExprOperator::Xor => {
                    let a = stack.pop()?;
                    let b = stack.pop()?;
                    let r = a ^ b;
                    stack.push(r);
                },
                TORExprOperator::And => {
                    let a = stack.pop()?;
                    let b = stack.pop()?;
                    let r = a & b;
                    stack.push(r);
                },
                TORExprOperator::Or => {
                    let a = stack.pop()?;
                    let b = stack.pop()?;
                    let r = a | b;
                    stack.push(r);
                },
                TORExprOperator::Not => {
                    let a = stack.pop()?;
                    let r = ! a;
                    stack.push(r);
                },
                TORExprOperator::ExitAllRel => break,
                TORExprOperator::ExitWithRel { rel } => {
                    let result = stack.pop()?;
                    return Some(result.tagged(*rel))
                }
            }
        }
        Some(stack.pop()?.objects())
    }
}