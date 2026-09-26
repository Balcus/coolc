use lalrpop_util::lalrpop_mod;

pub mod ast;
pub mod diagnostic;
pub mod lexer;
pub mod parse_tree;
pub mod parser;
pub mod semantic_analysis;
pub mod string_table;
pub mod utils;

lalrpop_mod!(
    #[allow(clippy::all)]
    #[rustfmt::skip]
    pub grammar
);
