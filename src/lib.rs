use lalrpop_util::lalrpop_mod;

pub mod ast;
pub mod lexer;
pub mod string_table;
pub mod diagnostic;
pub mod parser;
pub mod utils;
pub mod semantic_analysis;

lalrpop_mod!(
    #[allow(clippy::all)]
    #[rustfmt::skip]
    pub grammar
);
