use logos::{FilterResult, Lexer, Logos, Skip};
use std::collections::HashMap;

const MAX_STR_CONST_LEN: usize = 1024;

#[derive(Default)]
pub struct LexerExtras {
    line: usize,
    column: usize,
    pub s_table: StringTable,
}

#[derive(Default)]
pub struct StringTable {
    pub map: HashMap<String, usize>,
    strings: Vec<String>,
}

impl StringTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            strings: Vec::new(),
        }
    }

    pub fn insert(&mut self, string: String) -> usize {
        match self.map.get(&string) {
            Some(id) => *id,
            None => {
                let id = self.strings.len();
                self.map.insert(string.clone(), id);
                self.strings.push(string);
                id
            }
        }
    }

    pub fn get(&self, id: usize) -> Option<&String> {
        self.strings.get(id)
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

#[derive(Default, Debug, PartialEq, Clone)]
pub struct ErrorToken {
    kind: ErrorKind,
    message: Option<String>,
}

impl ErrorToken {
    pub fn new(kind: ErrorKind, message: Option<String>) -> Self {
        Self { kind, message }
    }
}

#[derive(Logos, Debug, PartialEq)]
#[logos(subpattern alpha = r"[a-zA-Z]")]
#[logos(subpattern digit = r"[0-9]")]
#[logos(subpattern alphanum = r"(?&alpha)|(?&digit)")]
#[logos(extras = LexerExtras)]
#[logos(skip(r"\n", newline_callback))]
#[logos(skip(r"[ \v\r\t\f]+"))]
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

fn newline_callback(lex: &mut Lexer<Token>) -> Skip {
    lex.extras.line += 1;
    lex.extras.column = lex.span().end;
    Skip
}

fn block_comment_callback(lex: &mut Lexer<Token>) -> FilterResult<(), ErrorToken> {
    let mut depth = 1usize;
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

    lex.bump(lex.remainder().len());
    FilterResult::Error(ErrorToken::new(
        ErrorKind::EofInComment,
        Some(String::from("EOF in comment")),
    ))
}

fn string_callback(lex: &mut Lexer<Token>) -> Result<usize, ErrorToken> {
    let mut result = String::new();
    let mut chars = lex.remainder().chars().peekable();
    let mut consumed_bytes = 0;

    while let Some(c) = chars.next() {
        consumed_bytes += c.len_utf8();

        match c {
            '\0' => {
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
                lex.bump(consumed_bytes);
                return Err(ErrorToken::new(
                    ErrorKind::StringContainsNullCharacter,
                    Some(String::from("String contains null character")),
                ));
            }
            '\n' => {
                lex.bump(consumed_bytes - c.len_utf8());
                return Err(ErrorToken::new(
                    ErrorKind::UnterminatedStringConstant,
                    Some(String::from("Unterminated string constant")),
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
                    lex.bump(consumed_bytes);
                    return Err(ErrorToken::new(
                        ErrorKind::EofInString,
                        Some(String::from("EOF in string constant")),
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
            lex.bump(consumed_bytes);
            return Err(ErrorToken::new(
                ErrorKind::StringConstantTooLong,
                Some(String::from("String constant too long")),
            ));
        }
    }

    lex.bump(consumed_bytes);
    Err(ErrorToken::new(
        ErrorKind::EofInString,
        Some(String::from("EOF in string constant")),
    ))
}

fn invalid_character_callback(lex: &mut Lexer<Token>) -> ErrorToken {
    ErrorToken::new(ErrorKind::InvalidCharacter, Some(lex.slice().to_string()))
}

fn unmatched_close_comment_callback(_lex: &mut Lexer<Token>) -> Result<logos::Skip, ErrorToken> {
    Err(ErrorToken::new(
        ErrorKind::UnmatchedCloseComment,
        Some(String::from("Unmatched *)")),
    ))
}
