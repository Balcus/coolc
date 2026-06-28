use crate::lexer;
use ariadne::{Label, Report, Source};

pub struct Diagnostic {
    file: String,
    source: String,
}

impl Diagnostic {
    pub fn new(file: String, source: String) -> Self {
        Self { file, source }
    }

    pub fn emit_lexing_error(&self, error: lexer::ErrorToken) {
        Report::build(ariadne::ReportKind::Error, error.span.clone())
            .with_message(error.message.clone())
            .with_label(
                Label::new(error.span)
                    .with_message(error.message)
                    .with_color(ariadne::Color::Red),
            )
            .finish()
            .print((self.file.clone(), Source::from(self.source.clone())))
            .expect("Failed to print error message for lexing error");
    }

    pub fn emit_parsing_error(
        &self,
        _error: lalrpop_util::ParseError<usize, lexer::Token, lexer::ErrorToken>,
    ) {
        todo!()
    }
}
