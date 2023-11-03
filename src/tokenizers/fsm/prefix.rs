use super::FSMStateMatcher;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixOperator {
    Not,
    Boost,
    Fuzz,
}

impl FSMStateMatcher for PrefixOperator {
    fn matches(self, inp: &str) -> Option<u8> {
        match self {
            PrefixOperator::Not => {
                if inp.starts_with("NOT") {
                    Some(3)
                } else if inp.starts_with('!') || inp.starts_with('-') {
                    Some(1)
                } else {
                    None
                }
            }
            PrefixOperator::Boost => {
                if inp.starts_with('^') {
                    Some(1)
                } else {
                    None
                }
            }
            PrefixOperator::Fuzz => {
                if inp.starts_with('~') {
                    Some(1)
                } else {
                    None
                }
            }
        }
    }

    fn maximum_bound(self) -> Option<u8> {
        Some(match self {
            PrefixOperator::Not => "NOT".len(),
            PrefixOperator::Boost => "^".len(),
            PrefixOperator::Fuzz => "~".len(),
        } as u8)
    }
}
