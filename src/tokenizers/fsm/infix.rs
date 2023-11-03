use super::FSMStateMatcher;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfixOperator {
    And,
    Or,
}

impl FSMStateMatcher for InfixOperator {
    fn matches(self, inp: &str) -> Option<u8> {
        match self {
            InfixOperator::And => {
                if inp.starts_with(',') {
                    Some(1)
                } else if inp.starts_with("&&") {
                    Some(2)
                } else if inp.starts_with("AND") {
                    Some(3)
                } else {
                    None
                }
            }
            InfixOperator::Or => {
                if inp.starts_with("||") || inp.starts_with("OR") {
                    Some(2)
                } else {
                    None
                }
            }
        }
    }

    fn maximum_bound(self) -> Option<u8> {
        match self {
            InfixOperator::And => Some(3),
            InfixOperator::Or => Some(2),
        }
    }
}
