use lalrpop_util::lalrpop_mod;

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod s_table;

lalrpop_mod!(
    #[allow(clippy::all)]
    #[rustfmt::skip]
    pub grammar
);
