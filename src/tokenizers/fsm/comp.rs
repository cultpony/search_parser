use super::FSMStateMatcher;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparator {
    GreaterThanOrEqual,
    GreaterThan,
    Equal,
    LessThan,
    LessThanOrEqual,
    NotEqual,
    Contains,
}

impl FSMStateMatcher for Comparator {
    fn matches(self, inp: &str) -> Option<u8> {
        match self {
            Comparator::GreaterThanOrEqual if inp.len() >= 4 => match &inp[..4] {
                "gte:" => Some(4),
                "Gte:" => Some(4),
                "gTe:" => Some(4),
                "gtE:" => Some(4),
                "GTe:" => Some(4),
                "GtE:" => Some(4),
                "GTE:" => Some(4),
                _ => None,
            },
            Comparator::GreaterThan if inp.len() >= 3 => match &inp[..3] {
                "gt:" => Some(3),
                "Gt:" => Some(3),
                "gT:" => Some(3),
                "GT:" => Some(3),
                _ => None,
            },
            Comparator::Equal if inp.len() >= 3 => match &inp[..3] {
                "eq:" => Some(3),
                "eQ:" => Some(3),
                "Eq:" => Some(3),
                "EQ:" => Some(3),
                _ => None,
            },
            Comparator::LessThan if inp.len() >= 3 => match &inp[..3] {
                "lt:" => Some(3),
                "lT:" => Some(3),
                "Lt:" => Some(3),
                "LT:" => Some(3),
                _ => None,
            },
            Comparator::LessThanOrEqual if inp.len() >= 4 => match &inp[..4] {
                "lte:" => Some(4),
                "Lte:" => Some(4),
                "lTe:" => Some(4),
                "ltE:" => Some(4),
                "LTe:" => Some(4),
                "LtE:" => Some(4),
                "LTE:" => Some(4),
                _ => None,
            },
            Comparator::NotEqual if inp.len() >= 4 => match &inp[..4] {
                "neq:" => Some(4),
                "Neq:" => Some(4),
                "nEq:" => Some(4),
                "neQ:" => Some(4),
                "NEq:" => Some(4),
                "NeQ:" => Some(4),
                "NEQ:" => Some(4),
                _ => None,
            },
            Comparator::Contains if inp.len() >= 4 => match &inp[..4] {
                "has:" => Some(4),
                "Has:" => Some(4),
                "hAs:" => Some(4),
                "haS:" => Some(4),
                "HAs:" => Some(4),
                "HaS:" => Some(4),
                "HAS:" => Some(4),
                _ => None,
            },
            _ => None,
        }
    }

    fn maximum_bound(self) -> Option<u8> {
        Some(match self {
            Comparator::GreaterThanOrEqual => "gte:".len(),
            Comparator::GreaterThan => "gt:".len(),
            Comparator::Equal => "eq:".len(),
            Comparator::LessThan => "lt".len(),
            Comparator::LessThanOrEqual => "lte:".len(),
            Comparator::NotEqual => "neq:".len(),
            Comparator::Contains => "has:".len(),
        } as u8)
    }
}
