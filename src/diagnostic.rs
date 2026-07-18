use crate::{lexer, semantic_analysis, utils};
use ariadne::{Label, Report, Source};

pub enum CompileError {
    LexicalError(lexer::ErrorToken),
    InvalidToken {
        span: utils::Span,
    },
    UnrecognizedEof {
        span: utils::Span,
        expected: Vec<String>,
    },
    UnrecognizedToken {
        span: utils::Span,
        token: lexer::Token,
        expected: Vec<String>,
    },
    ExtraToken {
        span: utils::Span,
        token: lexer::Token,
    },
    Semantic(semantic_analysis::SemanticError)
}

impl CompileError {
    fn message(&self) -> String {
        match self {
            CompileError::LexicalError(error_token) => error_token.message.clone(),
            CompileError::InvalidToken { .. } => "Invalid token".to_string(),
            CompileError::UnrecognizedEof { expected, .. } => {
                format!("Unrecognized EOF, expected one of: {}", expected.join(", "))
            }
            CompileError::UnrecognizedToken {
                expected, token, ..
            } => format!(
                "Unexpected token: {:?}, expected one of: {}",
                token,
                expected.join(", ")
            ),
            CompileError::ExtraToken { token, .. } => format!("Extra token: {:?}", token),
            _ => todo!()
        }
    }

    fn span(&self) -> utils::Span {
        match self {
            CompileError::LexicalError(error_token) => error_token.span.clone(),
            CompileError::InvalidToken { span } => span.clone(),
            CompileError::UnrecognizedEof { span, .. } => span.clone(),
            CompileError::UnrecognizedToken { span, .. } => span.clone(),
            CompileError::ExtraToken { span, .. } => span.clone(),
            _ => todo!()
        }
    }
}

impl From<lexer::ErrorToken> for CompileError {
    fn from(e: lexer::ErrorToken) -> Self {
        Self::LexicalError(e)
    }
}

impl
    From<(
        lalrpop_util::ParseError<usize, lexer::Token, lexer::ErrorToken>,
        &str,
    )> for CompileError
{
    fn from(
        (err, file): (
            lalrpop_util::ParseError<usize, lexer::Token, lexer::ErrorToken>,
            &str,
        ),
    ) -> Self {
        match err {
            lalrpop_util::ParseError::InvalidToken { location } => Self::InvalidToken {
                span: utils::Span::new(file.to_string(), location, location),
            },
            lalrpop_util::ParseError::UnrecognizedEof { location, expected } => {
                Self::UnrecognizedEof {
                    span: utils::Span::new(file.to_string(), location, location),
                    expected,
                }
            }
            lalrpop_util::ParseError::UnrecognizedToken { token, expected } => {
                Self::UnrecognizedToken {
                    span: utils::Span::new(file.to_string(), token.0, token.2),
                    token: token.1,
                    expected,
                }
            }
            lalrpop_util::ParseError::ExtraToken { token } => Self::ExtraToken {
                span: utils::Span::new(file.to_string(), token.0, token.2),
                token: token.1,
            },
            lalrpop_util::ParseError::User { error } => Self::LexicalError(error),
        }
    }
}

pub struct Diagnostic {
    file: String,
    source: String,
    errors: Vec<CompileError>,
}

impl Diagnostic {
    pub fn new(
        file: String,
        source: String,
        errors: Vec<lalrpop_util::ErrorRecovery<usize, lexer::Token, lexer::ErrorToken>>,
    ) -> Self {
        Self {
            file: file.clone(),
            source,
            errors: errors
                .iter()
                .cloned()
                .map(|e| CompileError::from((e.error, file.as_str())))
                .collect(),
        }
    }

    fn emit_error(&self, error: &CompileError) {
        Report::build(ariadne::ReportKind::Error, error.span().clone())
            .with_message(error.message())
            .with_label(
                Label::new(error.span())
                    .with_message(error.message())
                    .with_color(ariadne::Color::Red),
            )
            .finish()
            .print((self.file.clone(), Source::from(self.source.clone())))
            .expect("Failed to print error message");
    }

    pub fn emit_errors(&self) {
        for error in &self.errors {
            self.emit_error(error);
        }
    }
}
