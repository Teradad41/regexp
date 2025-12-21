//! 正規表現をパースし、抽象構文木(AST)に変換する
use std::{
    error::Error,
    fmt::{self, Display},
};

/// 抽象構文木を表現するための型
#[derive(Debug)]
pub enum AST {
    Char(char),
    Plus(Box<AST>),
    Star(Box<AST>),
    Question(Box<AST>),
    Or(Box<AST>, Box<AST>),
    Seq(Vec<AST>),
}

/// parse_plus_star_question 関数で利用するための列挙型
enum PSQ {
    Plus,
    Star,
    Question,
}

/// パースエラーを表すための型
#[derive(Debug)]
pub enum ParserError {
    InvalidEscape(usize, char), // 誤ったエスケープシーケンス
    InvalidRightParen(usize),   //開き括弧なし
    NoPrev(usize),              // +, |, *, ? の前に式がない
    NoRightParen,               // 閉じ括弧なし
    Empty,                      // 空のパターン
}

impl Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::InvalidEscape(pos, c) => {
                write!(f, "ParseError: invalid escape: pos = {pos}, char = '{c}'")
            }
            ParserError::InvalidRightParen(pos) => {
                write!(f, "ParseError: invalid right parenthesis: pos = {pos}")
            }
            ParserError::NoPrev(pos) => {
                write!(f, "ParseError: no previous expression: pos = {pos}")
            }
            ParserError::NoRightParen => write!(f, "ParseError: no right parenthesis"),
            ParserError::Empty => write!(f, "ParseError: empty expression"),
        }
    }
}

impl Error for ParserError {} // エラー用に Error トレイトを実装

/// 特殊文字のエスケープ処理を行う
fn parse_escape(pos: usize, c: char) -> Result<AST, ParserError> {
    match c {
        '\\' | '(' | ')' | '|' | '+' | '*' | '?' => Ok(AST::Char(c)),
        _ => {
            let err = ParserError::InvalidEscape(pos, c);
            Err(err)
        }
    }
}

/// +, *, ? を AST に変換する
///
/// 後置記法で +, *, ? の前にパターンがない場合はエラー
fn parse_plus_star_question(
    seq: &mut Vec<AST>,
    ast_type: PSQ,
    pos: usize,
) -> Result<(), ParserError> {
    if let Some(prev) = seq.pop() {
        let ast = match ast_type {
            PSQ::Plus => AST::Plus(Box::new(prev)),
            PSQ::Star => AST::Star(Box::new(prev)),
            PSQ::Question => AST::Question(Box::new(prev)),
        };
        seq.push(ast);
        Ok(())
    } else {
        Err(ParserError::NoPrev(pos))
    }
}

/// OR で結合された複数の式を AST に変換する
fn fold_or(mut seq_or: Vec<AST>) -> Option<AST> {
    if seq_or.len() > 1 {
        // seq_or の要素が複数ある場合は、OR で式を結合
        let mut ast = seq_or.pop().unwrap();
        seq_or.reverse();
        for s in seq_or {
            ast = AST::Or(Box::new(s), Box::new(ast));
        }
        Some(ast)
    } else {
        // seq_or の要素が1つのみの場合は、最初の式を返す
        seq_or.pop()
    }
}
