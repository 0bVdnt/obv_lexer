use regex::Regex;
use lazy_static::lazy_static;
use super::token::Token;
// use super::error::LexerError;

lazy_static! {
    static ref IDENTIFIER_RE: Regex = Regex::new(r"\A[a-zA-Z_]\w*\b").unwrap();
    static ref CONSTANT_RE: Regex = Regex::new(r"\A[0-9]+\b").unwrap();
    static ref OPEN_PAREN_RE: Regex = Regex::new(r"\A\(").unwrap();
    static ref CLOSE_PAREN_RE: Regex = Regex::new(r"\A\)").unwrap();
    static ref OPEN_BRACE_RE: Regex = Regex::new(r"\A\{").unwrap();
    static ref CLOSE_BRACE_RE: Regex = Regex::new(r"\A\}").unwrap();
    static ref SEMICOLON_RE: Regex = Regex::new(r"\A;").unwrap();
    static ref WHITESPACE_RE: Regex = Regex::new(r"\A\s+").unwrap();
}

const KEYWORDS: [(&str, Token); 3] = [
    ("int", Token::KwInt),
    ("void", Token::KwVoid),
    ("return", Token::KwReturn),
];

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}
