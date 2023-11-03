use tracing::trace;

use crate::tokens::Token;

use super::{
    comp::Comparator, data::DataValueType, infix::InfixOperator, prefix::PrefixOperator,
    FSMStateMatcher, FSM, token_and_field,
};

macro_rules! states {
    ( $fsm:ty : $tok:expr ) => { {
        const STATES: [$fsm; 1] = [ $tok ];
        &STATES
    } };
    ( $fsm:ty : $n:expr , $( $tok:expr ,)+ ) => { {
        const STATES: [$fsm; $n] = [ $($tok ,)* ];
        &STATES
    } };
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum StateMachine {
    /// Initial State of the Tokenizer
    #[default]
    Start,
    /// Starts a new Group of values (such as field)
    GroupStart,
    /// End of a Group
    GroupEnd,
    /// An Infix Operator such as AND/OR
    InfixOperator(InfixOperator),
    /// gte:/gt:/neq:/eq:/lte:
    Comparator(Comparator),
    /// A prefix operator such as NOT, BOOST or FUZZ
    PrefixOperator(PrefixOperator),
    /// An IP, Date (relative or Absolute), Integer of
    DataValue(DataValueType),
    /// A field ends with a "." followed by a comparator
    Field,
    /// A tag is a string occuring on the element
    Tag,
    /// State matches only end of input
    EndOfInput,
}

impl FSM for StateMachine {
    type NextStateType = Self;

    fn to_token(self) -> crate::tokens::Token {
        match self {
            StateMachine::Start => Token::ROOT,
            StateMachine::GroupStart => Token::LPAREN,
            StateMachine::GroupEnd => Token::RPAREN,
            StateMachine::InfixOperator(InfixOperator::And) => Token::AND,
            StateMachine::InfixOperator(InfixOperator::Or) => Token::OR,
            StateMachine::Comparator(Comparator::Contains) => Token::RANGE,
            StateMachine::Comparator(Comparator::Equal) => Token::RANGE,
            StateMachine::Comparator(Comparator::GreaterThan) => Token::RANGE,
            StateMachine::Comparator(Comparator::GreaterThanOrEqual) => Token::RANGE,
            StateMachine::Comparator(Comparator::LessThan) => Token::RANGE,
            StateMachine::Comparator(Comparator::LessThanOrEqual) => Token::RANGE,
            StateMachine::Comparator(Comparator::NotEqual) => Token::RANGE,
            StateMachine::PrefixOperator(PrefixOperator::Boost) => Token::BOOST,
            StateMachine::PrefixOperator(PrefixOperator::Fuzz) => Token::FUZZ,
            StateMachine::PrefixOperator(PrefixOperator::Not) => Token::NOT,
            StateMachine::DataValue(DataValueType::AbsoluteDate) => Token::ABSOLUTE_DATE,
            StateMachine::DataValue(DataValueType::Boolean) => Token::BOOLEAN,
            StateMachine::DataValue(DataValueType::CIDR) => Token::IP_CIDR,
            StateMachine::DataValue(DataValueType::Float) => Token::FLOAT,
            StateMachine::DataValue(DataValueType::IP) => Token::IP_CIDR,
            StateMachine::DataValue(DataValueType::Integer) => Token::INTEGER,
            StateMachine::DataValue(DataValueType::RelativeDate) => Token::RELATIVE_DATE,
            StateMachine::DataValue(DataValueType::String) => Token::QUOTED_TERM,
            StateMachine::Field => Token::FIELD,
            StateMachine::Tag => Token::TAG,
            StateMachine::EndOfInput => Token::EOI,
        }
    }

    fn next_states(self) -> &'static [Self::NextStateType] {
        match self {
            StateMachine::Start => states!(StateMachine : 7,
                StateMachine::GroupStart,
                StateMachine::Field,
                StateMachine::Tag,
                StateMachine::PrefixOperator(PrefixOperator::Boost),
                StateMachine::PrefixOperator(PrefixOperator::Fuzz),
                StateMachine::PrefixOperator(PrefixOperator::Not),
                StateMachine::EndOfInput,
            ),
            StateMachine::GroupStart => states!(StateMachine : 7,
                StateMachine::GroupStart,
                StateMachine::Field,
                StateMachine::Tag,
                StateMachine::PrefixOperator(PrefixOperator::Boost),
                StateMachine::PrefixOperator(PrefixOperator::Fuzz),
                StateMachine::PrefixOperator(PrefixOperator::Not),
                StateMachine::GroupEnd,
            ),
            StateMachine::GroupEnd => states!(StateMachine : 4,
                StateMachine::InfixOperator(InfixOperator::And),
                StateMachine::InfixOperator(InfixOperator::Or),
                StateMachine::GroupEnd,
                StateMachine::EndOfInput,
            ),
            StateMachine::InfixOperator(_) => states!(StateMachine : 6,
                StateMachine::GroupStart,
                StateMachine::Field,
                StateMachine::Tag,
                StateMachine::PrefixOperator(PrefixOperator::Boost),
                StateMachine::PrefixOperator(PrefixOperator::Fuzz),
                StateMachine::PrefixOperator(PrefixOperator::Not),
            ),
            StateMachine::Comparator(Comparator::Equal | Comparator::NotEqual) => {
                states!(StateMachine : 8,
                    StateMachine::DataValue(DataValueType::AbsoluteDate),
                    StateMachine::DataValue(DataValueType::Boolean),
                    StateMachine::DataValue(DataValueType::CIDR),
                    StateMachine::DataValue(DataValueType::Float),
                    StateMachine::DataValue(DataValueType::IP),
                    StateMachine::DataValue(DataValueType::Integer),
                    StateMachine::DataValue(DataValueType::RelativeDate),
                    StateMachine::DataValue(DataValueType::String),
                )
            }
            StateMachine::Comparator(
                Comparator::GreaterThan
                | Comparator::GreaterThanOrEqual
                | Comparator::LessThan
                | Comparator::LessThanOrEqual,
            ) => states!(StateMachine : 4,
                StateMachine::DataValue(DataValueType::Float),
                StateMachine::DataValue(DataValueType::Integer),
                StateMachine::DataValue(DataValueType::RelativeDate),
                StateMachine::DataValue(DataValueType::AbsoluteDate),
            ),
            StateMachine::Comparator(Comparator::Contains) => states!(StateMachine : 3,
                StateMachine::DataValue(DataValueType::String),
                StateMachine::DataValue(DataValueType::CIDR),
                StateMachine::DataValue(DataValueType::IP),
            ),
            StateMachine::PrefixOperator(PrefixOperator::Not) => states!(StateMachine : 5,
                StateMachine::GroupStart,
                StateMachine::Tag,
                StateMachine::Field,
                StateMachine::DataValue(DataValueType::CIDR),
                StateMachine::DataValue(DataValueType::Boolean),
            ),
            StateMachine::PrefixOperator(PrefixOperator::Boost | PrefixOperator::Fuzz) => {
                states!(StateMachine : 3,
                    StateMachine::GroupStart,
                    StateMachine::Tag,
                    StateMachine::Field,
                )
            }
            StateMachine::DataValue(_) => states!(StateMachine : 4,
                StateMachine::GroupEnd,
                StateMachine::InfixOperator(InfixOperator::And),
                StateMachine::InfixOperator(InfixOperator::Or),
                StateMachine::EndOfInput,
            ),
            StateMachine::Field => states!(StateMachine : 6,
                StateMachine::Comparator(Comparator::Equal),
                StateMachine::Comparator(Comparator::NotEqual),
                StateMachine::Comparator(Comparator::LessThan),
                StateMachine::Comparator(Comparator::LessThanOrEqual),
                StateMachine::Comparator(Comparator::GreaterThan),
                StateMachine::Comparator(Comparator::GreaterThanOrEqual),
            ),
            StateMachine::Tag => states!(StateMachine :  4,
                StateMachine::InfixOperator(InfixOperator::And),
                StateMachine::InfixOperator(InfixOperator::Or),
                StateMachine::GroupEnd,
                StateMachine::EndOfInput,
            ),
            StateMachine::EndOfInput => states!(StateMachine : StateMachine::EndOfInput ),
        }
    }
}

impl FSMStateMatcher for StateMachine {
    #[tracing::instrument]
    fn matches(self, inp: &str) -> Option<u8> {
        match self {
            // Start never matches as it's immediately transitioned into a different state
            StateMachine::Start => None,
            StateMachine::GroupStart if inp.starts_with('(') => Some(1),
            StateMachine::GroupEnd if inp.starts_with(')') => Some(1),
            StateMachine::InfixOperator(v) => v.matches(inp),
            StateMachine::Comparator(v) => v.matches(inp),
            StateMachine::PrefixOperator(v) => v.matches(inp),
            StateMachine::DataValue(v) => v.matches(inp),
            StateMachine::Field => {
                let o = token_and_field::FieldLexem::new(inp).find_end();
                trace!("field match got {} chars: {:?}", o, &inp[..o]);
                if o < 2 {
                    None
                } else {
                    Some(o as u8)
                }
            }
            StateMachine::Tag => {
                // TODO: Tags can contain stuff, this should try to peek
                // ahead and stop when it finds control words
                let o = token_and_field::TagLexem::new(inp).find_end();
                trace!("tag match got {} chars: {:?}", o, &inp[..o]);
                if o < 2 {
                    None
                } else {
                    Some(o as u8)
                }
            }
            // EoI will only match if the input is zero-sized
            StateMachine::EndOfInput if inp.is_empty() => Some(0),
            _ => None,
        }
    }

    #[tracing::instrument]
    fn maximum_bound(self) -> Option<u8> {
        match self {
            StateMachine::Start => Some(0),
            StateMachine::GroupStart => Some(1),
            StateMachine::GroupEnd => Some(1),
            StateMachine::InfixOperator(v) => v.maximum_bound(),
            StateMachine::Comparator(v) => v.maximum_bound(),
            StateMachine::PrefixOperator(v) => v.maximum_bound(),
            StateMachine::DataValue(v) => v.maximum_bound(),
            // Maximum field name size is 64 characters (plus dot)
            // A field must be minimum 3 characters
            StateMachine::Field => Some(65),
            // Maximum tag name size is 255 characters
            // A tag must be minimum 2 characters
            StateMachine::Tag => Some(255),
            StateMachine::EndOfInput => None,
        }
    }
}
