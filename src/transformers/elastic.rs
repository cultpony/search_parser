use std::collections::HashMap;

use crate::{ast::{Expr, ApplyOp, Comp, CombOp, Value}, errors};
use elasticsearch_dsl::search::queries;

use super::{ITransformerFactory, ITransformer};

inventory::submit! { super::Transformer::new::<ElasticFactory>("esq") }

#[derive(Debug, Clone, Copy)]
pub struct ElasticFactory;

impl ITransformerFactory for ElasticFactory {
    fn init() -> Box<dyn ITransformerFactory> where Self: Sized {
        Box::new(Self)
    }

    fn new(&self, parser: Box<dyn crate::parsers::IParser>) -> errors::Result<Box<dyn super::ITransformer>> {
        Ok(ElasticTermProducer::new(parser)?)
    }
}

pub struct ElasticTermProducer { parser: Box<dyn crate::parsers::IParser> }

impl super::ITransformer for ElasticTermProducer {
    fn new(parser: Box<dyn crate::parsers::IParser>) -> errors::Result<Box<dyn super::ITransformer>> where Self: Sized {
        Ok(Box::new(Self{ parser }))
    }

    fn run(&mut self, mut output: Box<dyn std::io::Write>) -> errors::Result<()> {
        let expr = self.parser.produce_tree()?;
        let et = elasticsearch_dsl::Search::new()
            .source(true)
            .stats("statistics")
            .from(0)
            .size(30)
            .query(ElasticTerm::from(expr).0);
        let ets = serde_json::to_string_pretty(&et)?;
        let mut ets = std::io::Cursor::new(ets.as_bytes());
        std::io::copy(&mut ets, &mut output)?;
        output.write_all(&[b'\n'])?;
        Ok(())
    }
}

pub fn hash<K: std::hash::Hash + Eq + PartialEq, V>(key: K, value: V) -> HashMap<K, V> {
    let hm = HashMap::new();
    extend_hash(hm, key, value)
}

pub fn extend_hash<K: std::hash::Hash + Eq + PartialEq, V>(
    mut hm: HashMap<K, V>,
    key: K,
    value: V,
) -> HashMap<K, V> {
    hm.insert(key, value);
    hm
}

macro_rules! qm_range {
    (field_trim($field:expr)) => { {
        let field: &str = $field.as_str();
        assert!(field.len() > 1, "fields must have atleast 2 characters, including dot");
        assert!(field.ends_with('.'), "fields must end with a dot");
        field[..(field.len()-1)].to_string()
    } };
    ($inp:expr => cmp $q_type:ident $field:expr) => { {
        let field = qm_range!(field_trim($field));
        let inp: Value = $inp;
        ElasticTerm(match inp {
            Value::Integer(v) => {
                assert!(v < (i64::MAX as i128), "integer value too large for range query");
                assert!(v > (i64::MIN as i128), "integer value too large for range query");
                queries::Query::range(field).$q_type(v as i64).into()
            },
            Value::Float(v) => queries::Query::range(field).$q_type(v).into(),
            Value::Bool(v) => queries::Query::range(field).$q_type(v).into(),
            Value::IP(v) => queries::Query::range(field).$q_type(v).into(),
            Value::RelativeDate(v) => todo!("relative date match"),
            Value::AbsoluteDate(v) => todo!("absolute date match"),
            Value::Undefined => panic!("undefined value in range query"),
        })
    } };
    ($inp:expr => eq $field:expr) => { {
        let field = qm_range!(field_trim($field));
        let right = $inp;
        ElasticTerm(match right {
            Value::Integer(v) => queries::Query::term(field, v).into(),
            Value::Float(v) => queries::Query::term(field, v.to_string()).into(),
            Value::Bool(v) => queries::Query::term(field, v.to_string()).into(),
            Value::IP(v) => queries::Query::term(field, v.to_string()).into(),
            Value::RelativeDate(v) => todo!("relative date match"),
            Value::AbsoluteDate(v) => todo!("absolute date match"),
            Value::Undefined => queries::Query::match_none().into(),
        })
    } };
    ($inp:expr => neq $field:expr) => { {
        let field = qm_range!(field_trim($field));
        let right = $inp;
        ElasticTerm(queries::Query::bool().must_not(match right {
            Value::Integer(v) => queries::Query::Term(queries::Query::term(field, v)),
            Value::Float(v) => queries::Query::Term(queries::Query::term(field, v.to_string())),
            Value::Bool(v) => queries::Query::Term(queries::Query::term(field, v.to_string())),
            Value::IP(v) => queries::Query::Term(queries::Query::term(field, v.to_string())),
            Value::RelativeDate(v) => todo!("relative date match"),
            Value::AbsoluteDate(v) => todo!("absolute date match"),
            Value::Undefined => queries::Query::MatchAll(queries::Query::match_all()),
        }).into())
    } };
}

#[derive(serde::Serialize)]
#[repr(transparent)]
pub struct ElasticTerm(queries::Query);

impl From<Expr> for ElasticTerm {
    fn from(value: Expr) -> Self {
        match value {
            Expr::Apply(ApplyOp::Boost, _) => todo!(),
            Expr::Apply(ApplyOp::Fuzz, _) => todo!(),
            Expr::Apply(ApplyOp::Not, v) => ElasticTerm(queries::Query::bool().must_not({
                let vq: ElasticTerm = (*v).into();
                vq.0
            }).into()),
            Expr::Comparison(left, Comp::Equal, right) => qm_range!(right => eq left),
            Expr::Comparison(left, Comp::NotEqual, right) => qm_range!(right => neq left),
            Expr::Comparison(left, Comp::LessThan, right) => qm_range!(right => cmp lt left),
            Expr::Comparison(left, Comp::LessThanOrEqual, right) => qm_range!(right => cmp lte left),
            Expr::Comparison(left, Comp::GreaterThan, right) => qm_range!(right => cmp gt left),
            Expr::Comparison(left, Comp::GreaterThanOrEqual, right) => qm_range!(right => cmp gte left),
            Expr::Combine(CombOp::And, v) => ElasticTerm(queries::Query::bool().must(v.into_iter().map(|x| {
                let y: ElasticTerm = x.into();
                y.0
            })).into()),
            Expr::Combine(CombOp::Or, v) => ElasticTerm(queries::Query::bool().should(v.into_iter().map(|x| {
                let y: ElasticTerm = x.into();
                y.0
            })).into()),
            Expr::Group(_) => todo!("should not appear in optimal output"),
            Expr::Tag(v) => ElasticTerm(queries::Query::term("tag", v.to_string()).into()),
            v => todo!("not implemented: {v:?}"),
        }
    }
}
