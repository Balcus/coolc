use crate::{parse_tree::Program, lexer::LexerWrapper, parser, string_table::StringTable};

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Span {
    file: String,
    start: usize,
    end: usize,
}

impl ariadne::Span for Span {
    type SourceId = String;

    fn source(&self) -> &Self::SourceId {
        &self.file
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

// The span is inclusive for start but exclusive for end [start, end)
// Used to hold location information in case of lexing errors
impl Span {
    pub fn new(file: String, start: usize, end: usize) -> Self {
        Self { file, start, end }
    }
}

// used for unit-testing
pub fn parse_program(input: &str) -> (StringTable, Program) {
    let mut string_table = StringTable::new();
    let mut errors = Vec::new();

    let tokens = Box::new(LexerWrapper::new(input, &mut string_table, "".to_string()));

    let mut parser = parser::Parser::new(&mut errors);
    let program = parser.parse(tokens).unwrap();

    (string_table, program)
}
