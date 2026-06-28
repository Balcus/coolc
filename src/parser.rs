use crate::{
    ast,
    lexer::{ErrorToken, Spanned, Token},
    grammar,
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

    pub fn parse(
        &mut self,
        tokens: TokenStream<'_>,
    ) -> Result<ast::Program, &[ErrorRecovery<usize, Token, ErrorToken>]> {
        self.errors.clear();

        match grammar::ProgramParser::new().parse(self.errors, tokens) {
            Ok(program) if self.errors.is_empty() => Ok(program),
            Err(e) => {
                self.errors.push(ErrorRecovery {
                    error: e,
                    dropped_tokens: vec![],
                });
                Err(self.errors.as_slice())
            }
            _ => Err(self.errors.as_slice()),
        }
    }
}