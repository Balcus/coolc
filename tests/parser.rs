use coolc::{grammar, lexer::LexerWrapper};

#[test]
fn matched_paren_with_term() {
    assert!(
        grammar::TermParser::new()
            .parse(LexerWrapper::new("(123)"))
            .is_ok()
    );
    assert!(
        grammar::TermParser::new()
            .parse(LexerWrapper::new("((123))"))
            .is_ok()
    );
    assert!(
        grammar::TermParser::new()
            .parse(LexerWrapper::new("1"))
            .is_ok()
    );
    assert!(
        grammar::TermParser::new()
            .parse(LexerWrapper::new("((((1)))"))
            .is_err()
    )
}
