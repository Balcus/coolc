use lalrpop_util::lalrpop_mod;

pub mod ast;
pub mod lexer;
pub mod s_table;
pub mod diagnostic;
pub mod parser;

lalrpop_mod!(
    #[allow(clippy::all)]
    #[rustfmt::skip]
    pub grammar
);
