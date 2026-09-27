use crate::{lexer, semantic_analysis, string_table::StringTable, utils};
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
    Semantic(semantic_analysis::SemanticError),
}

impl CompileError {
    fn message(&self, s_table: &StringTable) -> String {
        match self {
            CompileError::LexicalError(error_token) => error_token.message.clone(),
            CompileError::InvalidToken { .. } => "Invalid token".to_string(),
            CompileError::UnrecognizedEof { expected, .. } => 
                format!("Unrecognized EOF, expected one of: {}", expected.join(", ")),
            

            CompileError::UnrecognizedToken {
                expected, token, ..
            } => format!(
                "Unexpected token: {:?}, expected one of: {}",
                token,
                expected.join(", ")
            ),
            CompileError::ExtraToken { token, .. } => format!("Extra token: {:?}", token),
            CompileError::Semantic(e) => Self::semantic_message(e, s_table),
        }
    }

    fn semantic_message(error: &semantic_analysis::SemanticError, s_table: &StringTable) -> String {
        use semantic_analysis::SemanticErrorKind::*;

        let name = |id: &usize| s_table.string_from_id(*id).map(|s| s.as_str()).unwrap_or("<unknown>").to_string();

        match &error.kind {
            InheritanceCycle => "Inheritance cycle detected".to_string(),
            DuplicateClass { name: n } => format!("Duplicate class definition: {}", name(n)),
            UndefinedClass { name: n } => format!("Undefined class: {}", name(n)),
            RedefinedMethod { class, method } => format!(
                "Method {} is redefined in class {}",
                name(method),
                name(class)
            ),
            WrongOverrideSignature { class, method } => format!(
                "Method {} in class {} has a signature that doesn't match its parent's",
                name(method),
                name(class)
            ),
            UndefinedMethod { class, method } => 
                format!("Undefined method {} in class {}", name(method), name(class)),
            

            AttributeMismatch { attribute, found } => format!(
                "Attribute {} has type {:?}, which doesn't match its declaration",
                name(attribute),
                found
            ),
            AssignmentToSelf => "Cannot assign to 'self'".to_string(),
            UndeclaredIdentifier { name: n } => format!("Undeclared identifier: {}", name(n)),
            InvalidArithmeticOperandType { found } => format!(
                "Invalid operand type for arithmetic expression: {:?}",
                found
            ),

            InvalidNegationType { found } => 
                format!("Invalid operand type for negation: {:?}", found),
            

            TypeMismatch { expected, found } => 
                format!("Type mismatch: expected {:?}, found {:?}", expected, found),
            

            WrongNumberOfArguments { expected, found } => format!(
                "Wrong number of arguments: expected {}, found {}",
                expected, found
            ),
            DuplicateCaseBranchType { ty } => 
                format!("Duplicate branch type in case expression: {}", name(ty)),
            

            InvalidBlockConstruct => "Invalid block construct".to_string(),
        }
    }

    fn span(&self, file: &str) -> utils::Span {
        match self {
            CompileError::LexicalError(error_token) => error_token.span.clone(),
            CompileError::InvalidToken { span } => span.clone(),
            CompileError::UnrecognizedEof { span, .. } => span.clone(),
            CompileError::UnrecognizedToken { span, .. } => span.clone(),
            CompileError::ExtraToken { span, .. } => span.clone(),
            CompileError::Semantic(e) => match &e.span {
                Some(span) => span.clone(),
                None => utils::Span::new(file.to_string(), 0, 0),
            },
        }
    }
}

impl From<lexer::ErrorToken> for CompileError {
    fn from(e: lexer::ErrorToken) -> Self {
        Self::LexicalError(e)
    }
}

impl From<semantic_analysis::SemanticError> for CompileError {
    fn from(e: semantic_analysis::SemanticError) -> Self {
        Self::Semantic(e)
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

    pub fn from_semantic_errors(
        file: String,
        source: String,
        errors: Vec<semantic_analysis::SemanticError>,
    ) -> Self {
        Self {
            file,
            source,
            errors: errors.into_iter().map(CompileError::from).collect(),
        }
    }

    fn emit_error(&self, error: &CompileError, s_table: &StringTable) {
        use ariadne::Span as _;

        let span = error.span(&self.file);
        let message = error.message(s_table);
        let file = span.source().clone();

        Report::build(ariadne::ReportKind::Error, span.clone())
            .with_message(&message)
            .with_label(
                Label::new(span)
                    .with_message(&message)
                    .with_color(ariadne::Color::Red),
            )
            .finish()
            .print((file, Source::from(self.source.clone())))
            .expect("Failed to print error message");
    }

    pub fn emit_errors(&self, s_table: &StringTable) {
        for error in &self.errors {
            self.emit_error(error, s_table);
        }
    }
}
