pub mod elastic;
pub mod parsers;

use bumpalo::Bump;
use clap::Parser;
use search_parser::ITokenizer;

use crate::elastic::ElasticTerm;


//#[global_allocator]
//static GLOBAL: Bump = Bump::new();

#[derive(Debug, clap::Parser)]
pub struct App {
    /// Search term to parse
    term: String,
    #[clap(long, short, default_value = "fsm")]
    /// Select the tokenizer to use
    tokenizer: TokenizerMode,
    #[clap(long, short, default_value = "esq")]
    /// Output Data
    output: OutputType,
    #[clap(long, short = 'p', default_value = "none")]
    optimizer: Optimizer,
    #[clap(long = "debug-memory", short = 'm')]
    output_memory_usage: bool,
    #[clap(long, short)]
    /// The argument is a file to read the search term from instead. If it's "-", read from stdin.
    file: bool,
}

#[derive(Debug, clap::ValueEnum, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Optimizer {
    #[clap(name = "none")]
    /// No Optimization
    None,
    /// Simple, Single-Iteration Syntax Tree Folding & Pruning
    #[clap(name = "stfap")]
    #[default]
    SimpleTreeFoldingAndPruning,
}

#[derive(Debug, clap::ValueEnum, Default, Clone, Copy)]
pub enum TokenizerMode {
    #[clap(name = "fsm")]
    #[default]
    /// A Finite State Machine Tokenizer (deterministic, O(n) runtime)
    FiniteStateMachine,
}

#[derive(Debug, clap::ValueEnum, Default, Clone, Copy)]
pub enum OutputType {
    #[clap(name = "tokens")]
    /// The Token Sequence Output of the Tokenizer
    TokenSequence,
    #[clap(name = "spans")]
    /// The Token Sequence Output of the Tokenizer with Span Information
    Spans,
    #[clap(name = "ast")]
    #[default]
    /// The Abstract Syntax Tree from the Parser
    SyntaxTree,
    #[clap(name = "esq")]
    /// JSON-formatted Elasticsearch Query
    ElasticsearchQuery,
    #[clap(name = "osq")]
    /// JSON-formatted Opensearch Query
    OpensearchQuery,
    #[clap(name = "qwq")]
    /// JSON-formatted Quickwit Query
    QuickwitQuery,
    #[clap(name = "pgq")]
    /// SQL Query against Postgres
    PostgreSQLQuery,
}

fn main() {
    let app = App::parse();
    let alloc = Bump::new();
    let input_size = app.term.as_bytes().len();
    let alloc_debug = |alloc: &Bump| {
        if app.output_memory_usage {
            let alloced = alloc.allocated_bytes();
            let allocmed = alloc.allocated_bytes_including_metadata();
            println!("ALLOC:   {:8>} Bytes ( {:4>} Bytes per Character )", alloced, alloced / input_size);
            println!("ALLOC+M: {:8>} Bytes ( {:4>} Bytes per Character )", allocmed, allocmed / input_size);
            if let Some(limit) = alloc.allocation_limit() {
                println!("LIMIT:   {:8>} Bytes ( {:4>} Bytes per Character )", limit, limit / input_size)
            }
        }
    };
    let term = if app.file {
        let st = std::fs::read_to_string(app.term).unwrap();
        println!("loaded file ({} bytes)", st.len());
        st
    } else {
        app.term
    };
    match app.output {
        OutputType::ElasticsearchQuery => {
            let tree = match app.tokenizer {
                TokenizerMode::FiniteStateMachine => search_parser::parse_string::<
                    search_parser::tokenizers::fsm::AllocTokenizer,
                >(&alloc, &term, app.optimizer >= Optimizer::SimpleTreeFoldingAndPruning),
            };
            let out: ElasticTerm = tree.into();
            alloc_debug(&alloc);
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }
        OutputType::TokenSequence => {
            let tokens = match app.tokenizer {
                TokenizerMode::FiniteStateMachine => {
                    let t = search_parser::tokenizers::fsm::AllocTokenizer::new(&alloc, &term);
                    t.tokens().unwrap()
                }
            };
            alloc_debug(&alloc);
            println!("{:#?}", tokens);
        }
        OutputType::Spans => {
            let tokens = match app.tokenizer {
                TokenizerMode::FiniteStateMachine => {
                    let t = search_parser::tokenizers::fsm::AllocTokenizer::new(&alloc, &term);
                    t.token_spans().unwrap()
                }
            };
            alloc_debug(&alloc);
            println!("{:#?}", tokens);
        }
        OutputType::SyntaxTree => {
            let tree = match app.tokenizer {
                TokenizerMode::FiniteStateMachine => search_parser::parse_string::<
                    search_parser::tokenizers::fsm::AllocTokenizer,
                >(&alloc, &term, app.optimizer >= Optimizer::SimpleTreeFoldingAndPruning),
            };
            alloc_debug(&alloc);
            println!("{:#?}", tree);
        },
        OutputType::OpensearchQuery => todo!("OpenSearch Queries not supported yet"),
        OutputType::QuickwitQuery => todo!("Quickwit Queries not supported yet"),
        OutputType::PostgreSQLQuery => todo!("PostgreSQL Queries not supported yet"),
    }
}
