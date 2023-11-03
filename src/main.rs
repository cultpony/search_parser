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
    #[clap(name = "dlln")]
    /// A LLN(n) Parser with adaptive Lookahead (Perfect Accuracy, Slower)
    DynamicLLN,
    #[clap(name = "fsm")]
    #[default]
    /// A Finite State Machine Tokenizer (Permissive, Faster)
    FiniteStateMachine,
    #[clap(name = "sr")]
    ShiftReduce,
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
    // DLLN takes a lot of memory to work, so this is probably a good guess for everything else
    let memory_estimate = search_parser::tokenizers::dlln::maximum_memory_estimate(&app.term);
    let alloc = Bump::with_capacity(memory_estimate);
    let input_size = app.term.as_bytes().len();
    if app.output_memory_usage {
        println!("EST:     {:8>} Bytes ( {:4>} Bytes per Character )", memory_estimate, memory_estimate / input_size);
    }
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
    match app.output {
        OutputType::ElasticsearchQuery => {
            let tree = match app.tokenizer {
                TokenizerMode::FiniteStateMachine => search_parser::parse_string::<
                    search_parser::tokenizers::fsm::AllocTokenizer,
                >(&alloc, &app.term, app.optimizer >= Optimizer::SimpleTreeFoldingAndPruning),
                TokenizerMode::DynamicLLN => search_parser::parse_string::<
                    search_parser::tokenizers::dlln::Tokenizer,
                >(&alloc, &app.term, app.optimizer >= Optimizer::SimpleTreeFoldingAndPruning),
                TokenizerMode::ShiftReduce => todo!(),
            };
            let out: ElasticTerm = tree.into();
            alloc_debug(&alloc);
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }
        OutputType::TokenSequence => {
            let tokens = match app.tokenizer {
                TokenizerMode::FiniteStateMachine => {
                    let t = search_parser::tokenizers::fsm::AllocTokenizer::new(&alloc, &app.term);
                    t.tokens().unwrap()
                }
                TokenizerMode::DynamicLLN => {
                    let t = search_parser::tokenizers::dlln::Tokenizer::new(&alloc, &app.term);
                    t.tokens().unwrap()
                }
                TokenizerMode::ShiftReduce => {
                    todo!("implement AST -> Token Sequence")
                },
            };
            alloc_debug(&alloc);
            println!("{:#?}", tokens);
        }
        OutputType::Spans => {
            let tokens = match app.tokenizer {
                TokenizerMode::FiniteStateMachine => {
                    let t = search_parser::tokenizers::fsm::AllocTokenizer::new(&alloc, &app.term);
                    t.token_spans().unwrap()
                }
                TokenizerMode::DynamicLLN => {
                    let t = search_parser::tokenizers::dlln::Tokenizer::new(&alloc, &app.term);
                    t.token_spans().unwrap()
                }
                TokenizerMode::ShiftReduce => todo!("implement AST -> Spans"),
            };
            alloc_debug(&alloc);
            println!("{:#?}", tokens);
        }
        OutputType::SyntaxTree => {
            let tree = match app.tokenizer {
                TokenizerMode::FiniteStateMachine => search_parser::parse_string::<
                    search_parser::tokenizers::fsm::AllocTokenizer,
                >(&alloc, &app.term, app.optimizer >= Optimizer::SimpleTreeFoldingAndPruning),
                TokenizerMode::DynamicLLN => search_parser::parse_string::<
                    search_parser::tokenizers::dlln::Tokenizer,
                >(&alloc, &app.term, app.optimizer >= Optimizer::SimpleTreeFoldingAndPruning),
                TokenizerMode::ShiftReduce => search_parser::tokenizers::lalr::parse(&app.term),
                    
            };
            alloc_debug(&alloc);
            println!("{:#?}", tree);
        },
        OutputType::OpensearchQuery => todo!("OpenSearch Queries not supported yet"),
        OutputType::QuickwitQuery => todo!("Quickwit Queries not supported yet"),
        OutputType::PostgreSQLQuery => todo!("PostgreSQL Queries not supported yet"),
    }
}
