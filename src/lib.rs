use lalrpop_util::lalrpop_mod;

pub mod lexer;
pub mod parser;

lalrpop_mod!(
    #[allow(clippy::all)]
    #[rustfmt::skip]
    pub grammar
);
