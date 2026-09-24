use crate::{
    grammar,
    lexer::{ErrorToken, Spanned, Token},
    parse_tree,
};
use lalrpop_util::ErrorRecovery;

type TokenStream<'t> = Box<dyn Iterator<Item = Spanned<Token, usize, ErrorToken>> + 't>;

pub struct Parser<'a> {
    file: &'a str,
    errors: &'a mut Vec<ErrorRecovery<usize, Token, ErrorToken>>,
}

impl<'a> Parser<'a> {
    pub fn new(
        file: &'a str,
        errors: &'a mut Vec<ErrorRecovery<usize, Token, ErrorToken>>,
    ) -> Self {
        Self { file, errors }
    }

    pub fn parse(&mut self, tokens: TokenStream<'_>) -> Option<parse_tree::Program> {
        self.errors.clear();
        match grammar::ProgramParser::new().parse(self.file, self.errors, tokens) {
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
