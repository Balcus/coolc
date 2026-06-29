use crate::{s_table::StringTable, utils};
use logos::{FilterResult, Lexer, Logos, SpannedIter};

const MAX_STR_CONST_LEN: usize = 1024;

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

/// The LexerWrapper is boilerplate used as a workaround for `lalrpop` to accept
/// external lexing libraries such as `Logos`.
///
/// Reference: https://lalrpop.github.io/lalrpop/lexer_tutorial/005_external_lib.html.
pub struct LexerWrapper<'input, 's: 'input> {
    token_stream: SpannedIter<'input, Token>,
    _marker: std::marker::PhantomData<&'s ()>,
}

impl<'input, 's: 'input> LexerWrapper<'input, 's> {
    pub fn new(input: &'input str, s_table: &'s mut StringTable, file: String) -> Self {
        Self {
            token_stream: Token::lexer_with_extras(input, LexerExtras { s_table, file }).spanned(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'input, 's: 'input> Iterator for LexerWrapper<'input, 's> {
    type Item = Spanned<Token, usize, ErrorToken>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream
            .next()
            .map(|(token, span)| Ok((span.start, token?, span.end)))
    }
}

pub struct LexerExtras<'s> {
    pub s_table: &'s mut StringTable,
    pub file: String,
}

/// The default constructor for LexerExtras `MUST NOT` be used.
/// Instead use the `lexer_with_extras`
/// option when constructing the lexer and pass it a mutable reference to a string table
impl Default for LexerExtras<'_> {
    fn default() -> Self {
        unreachable!("LexerExtras::default() should never be called")
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum ErrorKind {
    InvalidCharacter,

    // String constant lexing errors
    UnterminatedStringConstant,
    StringContainsNullCharacter,
    StringConstantTooLong,
    EofInString,

    // Comment lexing errors
    EofInComment,
    UnmatchedCloseComment,

    #[default]
    Other,
}

// Errors are communicated to the parser by returning a special error token
// called ErrorTokenthe. The lexer does not print anything
#[derive(Default, Debug, PartialEq, Clone)]
pub struct ErrorToken {
    pub kind: ErrorKind,
    pub message: String,
    pub span: utils::Span,
}

impl ErrorToken {
    pub fn new(kind: ErrorKind, message: String, span: utils::Span) -> Self {
        Self {
            kind,
            message,
            span,
        }
    }
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(subpattern alpha = r"[a-zA-Z]")]
#[logos(subpattern digit = r"[0-9]")]
#[logos(subpattern alphanum = r"(?&alpha)|(?&digit)")]
#[logos(extras = LexerExtras<'s>)]
#[logos(skip(r"[ \v\r\t\f\n]+"))]
#[logos(error = ErrorToken)]
pub enum Token {
    #[token(".")]
    Dot,

    #[token("@")]
    At,

    #[token("~")]
    Tilde,

    #[token("/")]
    Slash,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("=>")]
    FatArrow,

    #[token("<")]
    Lt,

    #[token("<=")]
    Le,

    #[token(">")]
    Gt,

    #[token(">=")]
    Ge,

    #[token("*")]
    Star,

    #[token("<-")]
    Assign,

    #[token(";")]
    Semicolon,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("=")]
    Eq,

    #[token("self")]
    SelfValue,

    #[token("SELF_TYPE")]
    SelfType,

    #[regex("(?i)class")]
    Class,

    #[regex("(?i)else")]
    Else,

    #[regex("(?i)fi")]
    Fi,

    #[regex("(?i)if")]
    If,

    #[regex("(?i)in")]
    In,

    #[regex("(?i)inherits")]
    Inherits,

    #[regex("(?i)isvoid")]
    IsVoid,

    #[regex("(?i)let")]
    Let,

    #[regex("(?i)loop")]
    Loop,

    #[regex("(?i)pool")]
    Pool,

    #[regex("(?i)then")]
    Then,

    #[regex("(?i)while")]
    While,

    #[regex("(?i)case")]
    Case,

    #[regex("(?i)esac")]
    Esac,

    #[regex("(?i)new")]
    New,

    #[regex("(?i)of")]
    Of,

    #[regex("(?i)not")]
    Not,

    #[regex("t(?i:rue)", |_| true)]
    #[regex("f(?i:alse)", |_| false)]
    BoolConst(bool),

    #[regex("(?&digit)+", |lex| lex.slice().parse().ok())]
    Integers(i64),

    #[regex("[A-Z]((?&alphanum)|_)*", |lex| lex.extras.s_table.insert(lex.slice().to_string()))]
    TypeIdentifier(usize),

    #[regex("[a-z]((?&alphanum)|_)*", |lex| lex.extras.s_table.insert(lex.slice().to_string()))]
    ObjectIdentifier(usize),

    #[token("\"", string_callback)]
    StringConstant(usize),

    #[regex(r"--[^\n]*", logos::skip, allow_greedy = true)]
    #[token("(*", block_comment_callback)]
    BlockComment,

    #[token("*)", unmatched_close_comment_callback)]
    UnmatchedCloseComment,

    #[regex(".", invalid_character_callback, priority = 1)]
    Err(ErrorToken),
}

// Block comments can be nested so we need a special callback function
// to manually track and match each '(*' with it's corresponding '*)'
fn block_comment_callback(lex: &mut Lexer<Token>) -> FilterResult<(), ErrorToken> {
    let mut depth = 1;
    let mut chars = lex.remainder().char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        match c {
            '(' if chars.peek().map(|&(_, c)| c) == Some('*') => {
                chars.next();
                depth += 1;
            }
            '*' if chars.peek().map(|&(_, c)| c) == Some(')') => {
                chars.next();
                depth -= 1;
                if depth == 0 {
                    lex.bump(i + 2);
                    return FilterResult::Skip;
                }
            }
            _ => {}
        }
    }

    let start = lex.span().start;
    let end = lex.span().end + lex.remainder().len();

    lex.bump(lex.remainder().len());

    FilterResult::Error(ErrorToken::new(
        ErrorKind::EofInComment,
        String::from("EOF in comment"),
        utils::Span::new(lex.extras.file.clone(), start, end),
    ))
}

fn string_callback(lex: &mut Lexer<Token>) -> Result<usize, ErrorToken> {
    let mut result = String::new();
    let mut chars = lex.remainder().chars().peekable();
    let mut consumed_bytes = 0;

    let start = lex.span().start;

    while let Some(c) = chars.next() {
        consumed_bytes += c.len_utf8();

        match c {
            '\0' => {
                while let Some(&next_c) = chars.peek() {
                    if next_c == '\n' {
                        break;
                    }
                    if next_c == '"' {
                        chars.next();
                        consumed_bytes += next_c.len_utf8();
                        break;
                    }
                    chars.next();
                    consumed_bytes += next_c.len_utf8();
                }

                let end = lex.span().end + consumed_bytes;
                lex.bump(consumed_bytes);
                return Err(ErrorToken::new(
                    ErrorKind::StringContainsNullCharacter,
                    String::from("String contains null character"),
                    utils::Span::new(lex.extras.file.clone(), start, end),
                ));
            }
            '\n' => {
                let end = lex.span().end + consumed_bytes - '\n'.len_utf8();
                lex.bump(consumed_bytes);

                return Err(ErrorToken::new(
                    ErrorKind::UnterminatedStringConstant,
                    String::from("Unterminated string constant"),
                    utils::Span::new(lex.extras.file.clone(), start, end),
                ));
            }
            '"' => {
                lex.bump(consumed_bytes);
                let id = lex.extras.s_table.insert(result);
                return Ok(id);
            }
            '\\' => {
                if let Some(next_c) = chars.next() {
                    consumed_bytes += next_c.len_utf8();
                    match next_c {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'b' => result.push('\x08'),
                        'f' => result.push('\x0C'),
                        _ => result.push(next_c),
                    }
                } else {
                    let end = lex.span().end + consumed_bytes;
                    lex.bump(consumed_bytes);
                    return Err(ErrorToken::new(
                        ErrorKind::EofInString,
                        String::from("EOF in string constant"),
                        utils::Span::new(lex.extras.file.clone(), start, end),
                    ));
                }
            }
            _ => {
                result.push(c);
            }
        }

        if result.len() > MAX_STR_CONST_LEN {
            while let Some(&next_c) = chars.peek() {
                if next_c == '"' || next_c == '\n' {
                    if next_c == '"' {
                        chars.next();
                        consumed_bytes += next_c.len_utf8();
                    }
                    break;
                }
                chars.next();
                consumed_bytes += next_c.len_utf8();
            }

            let end = lex.span().end + consumed_bytes;
            lex.bump(consumed_bytes);
            return Err(ErrorToken::new(
                ErrorKind::StringConstantTooLong,
                String::from("String constant too long"),
                utils::Span::new(lex.extras.file.clone(), start, end),
            ));
        }
    }

    let end = lex.span().end + consumed_bytes;
    lex.bump(consumed_bytes);
    Err(ErrorToken::new(
        ErrorKind::EofInString,
        String::from("EOF in string constant"),
        utils::Span::new(lex.extras.file.clone(), start, end),
    ))
}

fn invalid_character_callback(lex: &mut Lexer<Token>) -> ErrorToken {
    ErrorToken::new(
        ErrorKind::InvalidCharacter,
        lex.slice().to_string(),
        utils::Span::new(lex.extras.file.clone(), lex.span().start, lex.span().end),
    )
}

fn unmatched_close_comment_callback(lex: &mut Lexer<Token>) -> Result<logos::Skip, ErrorToken> {
    Err(ErrorToken::new(
        ErrorKind::UnmatchedCloseComment,
        String::from("Unmatched *)"),
        utils::Span::new(lex.extras.file.clone(), lex.span().start, lex.span().end),
    ))
}
