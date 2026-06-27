use crate::lexer::ErrorToken;

pub enum CoolError {
    Lexer(ErrorToken),
    Custom(&'static str),
}