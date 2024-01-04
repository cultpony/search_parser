
use std::io::BufWriter;

use clap::Parser;

#[derive(Debug, clap::Parser)]
pub struct App {
    /// Search term to parse
    term: String,

    #[clap(long, short)]
    /// The argument is a file to read the search term from instead. If it's "-", read from stdin.
    file: bool,

    #[clap(long, short, default_value = "fsm")]
    /// Select the tokenizer to use
    tokenizer: String,
    #[clap(long, short = 'o', default_value = "esq")]
    /// Output Data
    transformer: String,
    #[clap(long, short, default_value = "shift_reduce")]
    parser: String,
}

fn main() -> search_parser::errors::Result<()> {
    let app = App::parse();
    let term = if app.file {
        let st = std::fs::read_to_string(app.term).unwrap();
        println!("loaded file ({} bytes)", st.len());
        st
    } else {
        app.term
    };
    let output = std::io::stdout();
    let output = BufWriter::new(output);
    let output = Box::new(output);
    let tokenizer = search_parser::tokenizer(&app.tokenizer, &*term)?;
    let parser = search_parser::parser(&app.parser, tokenizer)?;
    let mut transformer = search_parser::transformer(&app.transformer, parser)?;

    transformer.run(output)
}
