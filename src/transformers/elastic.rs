use std::collections::HashMap;

use crate::{ast::{Expr, ApplyOp, Comp, CombOp, Value}, errors};

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
        let et = ElasticTerm::from(expr);
        let ets = serde_json::to_string_pretty(&et)?;
        let mut ets = std::io::Cursor::new(ets.as_bytes());
        std::io::copy(&mut ets, &mut output)?;
        output.write_all(&[b'\n'])?;
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ElasticTerm {
    #[serde(rename = "bool")]
    Bool {
        must: Vec<ElasticTerm>,
        should: Vec<ElasticTerm>,
        must_not: Vec<ElasticTerm>,
    },
    #[serde(rename = "boosting")]
    Boosting {
        todo: (),
    },
    Fuzzy {
        todo: (),
    },
    Range(HashMap<String, RangeField>),
    #[serde(rename = "term")]
    Term {
        tags: Vec<String>,
    },
    #[serde(rename = "term")]
    ExactTerm(HashMap<String, Value>),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct RangeField {
    gte: Option<Value>,
    lte: Option<Value>,
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

impl From<Expr> for ElasticTerm {
    fn from(value: Expr) -> Self {
        match value {
            Expr::Field(f) => ElasticTerm::Term {
                tags: vec![f.as_str().to_owned()],
            },
            Expr::Apply(ApplyOp::Boost, _) => todo!(),
            Expr::Apply(ApplyOp::Fuzz, _) => todo!(),
            Expr::Apply(ApplyOp::Not, _) => todo!(),
            Expr::Comparison(left, Comp::Equal, right) => {
                ElasticTerm::ExactTerm(hash(left.as_str().to_string(), right))
            }
            Expr::Combine(CombOp::And, v) => ElasticTerm::Bool {
                must: v
                    .into_iter()
                    .map(|x| {
                        let y: ElasticTerm = x.into();
                        y
                    })
                    .collect(),
                should: vec![],
                must_not: vec![],
            },
            Expr::Combine(CombOp::Or, v) => ElasticTerm::Bool {
                should: v
                    .into_iter()
                    .map(|x| {
                        let y: ElasticTerm = x.into();
                        y
                    })
                    .collect(),
                must: vec![],
                must_not: vec![],
            },
            Expr::Group(_) => todo!("should not appear in optimal output"),
            Expr::Tag(v) => ElasticTerm::Term { tags: vec![v.to_string()], },
            v => todo!("not implemented: {v:?}"),
        }
    }
}
