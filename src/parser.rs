use crate::{
    parse_tree, grammar,
    lexer::{ErrorToken, Spanned, Token},
};
use lalrpop_util::ErrorRecovery;

type TokenStream<'t> = Box<dyn Iterator<Item = Spanned<Token, usize, ErrorToken>> + 't>;

pub struct Parser<'a> {
    errors: &'a mut Vec<ErrorRecovery<usize, Token, ErrorToken>>,
}

impl<'a> Parser<'a> {
    pub fn new(errors: &'a mut Vec<ErrorRecovery<usize, Token, ErrorToken>>) -> Self {
        Self { errors }
    }

    pub fn parse(&mut self, tokens: TokenStream<'_>) -> Option<parse_tree::Program> {
        self.errors.clear();
        match grammar::ProgramParser::new().parse(self.errors, tokens) {
            Ok(program) if self.errors.is_empty() => Some(program),
            Ok(_) => None,
            Err(e) => {
                self.errors.push(ErrorRecovery {
                    error: e,
                    dropped_tokens: vec![],
                });
                None
            }
        }
    }
}
