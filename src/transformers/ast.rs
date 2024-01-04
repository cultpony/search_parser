use crate::{errors, ast::Expr};

use super::{ITransformerFactory, ITransformer};

inventory::submit! { super::Transformer::new::<ASTDumpFactory>("ast") }

pub struct ASTDump(Expr);

#[derive(Debug)]
pub struct ASTDumpFactory;

impl ITransformerFactory for ASTDumpFactory {
    fn init() -> Box<dyn ITransformerFactory> where Self: Sized {
        Box::new(Self)
    }

    fn new(&self, parser: Box<dyn crate::parsers::IParser>) -> crate::errors::Result<Box<dyn super::ITransformer>> {
        Ok(ASTDump::new(parser)?)
    }
}

impl ITransformer for ASTDump {
    fn new(mut parser: Box<dyn crate::parsers::IParser>) -> errors::Result<Box<dyn ITransformer>> where Self: Sized {
        Ok(Box::new(Self(parser.produce_tree()?)))
    }

    fn run(&mut self, mut output: Box<dyn std::io::Write>) -> crate::errors::Result<()> {
        let out = format!("{:#?}", self.0);
        let mut out = std::io::Cursor::new(out.as_bytes());
        std::io::copy(&mut out, &mut output)?;
        output.write_all(&[b'\n'])?;
        Ok(())
    }
}