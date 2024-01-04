use crate::{parsers::IParser, errors};

mod elastic;
mod token_seq;
mod ast;

pub trait ITransformerFactory: std::fmt::Debug {
    fn init() -> Box<dyn ITransformerFactory> where Self: Sized;
    fn new(&self, parser: Box<dyn IParser>) -> errors::Result<Box<dyn ITransformer>>;
}

pub trait ITransformer {
    fn new(parser: Box<dyn IParser>) -> errors::Result<Box<dyn ITransformer>> where Self: Sized;
    fn run(&mut self, output: Box<dyn std::io::Write>) -> errors::Result<()>;
}

pub struct Transformer {
    pub name: &'static str,
    pub imp: &'static (dyn Send + Sync + Fn() -> Box<dyn ITransformerFactory>),
}

impl Transformer {
    pub const fn new<I: ITransformerFactory>(name: &'static str) -> Self {
        Self {
            name,
            imp: &|| I::init(),
        }
    }
}

inventory::collect!(Transformer);

pub fn transformers() -> Vec<String> {
    inventory::iter::<Transformer>().map(|x| x.name.to_string()).collect()
}

pub fn transformer(name: &str, parser: Box<dyn IParser>) -> crate::errors::Result<Box<dyn ITransformer>> {
    for tra in inventory::iter::<Transformer> {
        if tra.name == name {
            return Ok((tra.imp)().new(parser)?)
        }
    }
    Err(errors::Error::UnknownTokenizer(name.to_string()))
}