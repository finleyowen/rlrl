#![allow(dead_code)]

use crate::lex::LexResult;
use crate::lex::Lexer;
use crate::parse::ParseError;
use crate::parse::TokenQueue;
use std::error::Error;
use std::i32;

/// An enum representing the tokens available to the lexer
#[derive(PartialEq)]
enum Token {
    // chars
    OParen,
    CParen,
    OBrace,
    CBrace,
    OAngle,
    CAngle,
    Comma,
    Equals,

    // kwds
    TypeKwd,
    FnKwd,

    // ident
    Ident(String),

    // literals
    IntLiteral(i32),
    DoubleLiteral(f64),
    StringLiteral(String),
}

impl Token {
    // helper function for handling identifiers
    pub fn is_ident_tok(&self) -> bool {
        if let Self::Ident(_) = self {
            return true;
        }
        false
    }

    // helper function for handling identifiers
    pub fn get_ident(&self) -> Result<&String, ParseError> {
        if let Self::Ident(ident) = self {
            return Ok(ident);
        }
        Err(ParseError::new(""))
    }
}

/// Function to setup the lexer for testing
fn setup_test_lexer() -> Lexer<Token> {
    let mut lexer: Lexer<Token> = Lexer::new();

    lexer.add_rule(r"[\s\n\t]+", |_| LexResult::Ignore);

    // chars
    lexer.add_rule(r"\(", |_| LexResult::Token(Token::OParen));
    lexer.add_rule(r"\)", |_| LexResult::Token(Token::CParen));
    lexer.add_rule(r"\{", |_| LexResult::Token(Token::OBrace));
    lexer.add_rule(r"\}", |_| LexResult::Token(Token::CBrace));
    lexer.add_rule(r"<", |_| LexResult::Token(Token::OAngle));
    lexer.add_rule(r">", |_| LexResult::Token(Token::CAngle));
    lexer.add_rule(r"\,", |_| LexResult::Token(Token::Comma));
    lexer.add_rule(r"=", |_| LexResult::Token(Token::Equals));

    // kwds
    lexer.add_rule(r"type", |_| LexResult::Token(Token::TypeKwd));
    lexer.add_rule(r"fn", |_| LexResult::Token(Token::FnKwd));

    // idents
    lexer.add_rule(r"[a-zA-Z][a-zA-Z0-9_]*", |re_match| {
        LexResult::Token(Token::Ident(re_match.as_str().into()))
    });

    // literals
    lexer.add_rule(r"\-?[0-9]+", |re_match| {
        match re_match.as_str().parse::<i32>() {
            Ok(v) => LexResult::Token(Token::IntLiteral(v)),
            Err(e) => LexResult::Error(e.into()),
        }
    });
    lexer.add_rule(r"\-?[0-9](\.[0-9]+)?", |re_match| {
        match re_match.as_str().parse::<f64>() {
            Ok(v) => LexResult::Token(Token::DoubleLiteral(v)),
            Err(e) => LexResult::Error(e.into()),
        }
    });
    lexer.add_rule("\"[^\"]*\"", |re_match| {
        LexResult::Token(Token::StringLiteral(
            re_match.as_str().trim_matches('"').into(),
        ))
    });

    lexer.add_rule(".", |re_match| {
        LexResult::Error(
            format!("Unmatched input at position {}", re_match.start()).into(),
        )
    });

    lexer
}

/// Test the lexer
#[test]
fn lex_test() {
    let lexer = setup_test_lexer();

    assert!(
        lexer.lex("({})").unwrap()
            == Vec::from([
                Token::OParen,
                Token::OBrace,
                Token::CBrace,
                Token::CParen
            ])
    );

    assert!(
        lexer.lex("({}, {})").unwrap()
            == Vec::from([
                Token::OParen,
                Token::OBrace,
                Token::CBrace,
                Token::Comma,
                Token::OBrace,
                Token::CBrace,
                Token::CParen
            ])
    );

    assert!(
        lexer.lex("fn my_function() {}").unwrap()
            == Vec::from([
                Token::FnKwd,
                Token::Ident("my_function".into()),
                Token::OParen,
                Token::CParen,
                Token::OBrace,
                Token::CBrace
            ])
    );

    assert!(
        lexer.lex("type int1to5 = int<1,5>").unwrap()
            == Vec::from([
                Token::TypeKwd,
                Token::Ident("int1to5".into()),
                Token::Equals,
                Token::Ident("int".into()),
                Token::OAngle,
                Token::IntLiteral(1),
                Token::Comma,
                Token::IntLiteral(5),
                Token::CAngle
            ])
    );
}

/// A struct we will try and parse from strings like "<5, 10>" or "<, 10>"
#[derive(PartialEq)]
struct IntRange {
    min: Option<i32>,
    max: Option<i32>,
}

impl TryFrom<&mut TokenQueue<Token>> for IntRange {
    type Error = ParseError;

    fn try_from(tq: &mut TokenQueue<Token>) -> Result<Self, Self::Error> {
        // consume '<'
        tq.consume_eq(Token::OAngle)?;

        // consume optional integer (min)
        let min = match *tq.peek()? {
            Token::IntLiteral(val) => {
                tq.increment();
                Some(val)
            }
            Token::Comma => None,
            _ => return Err(ParseError::new("")),
        };

        // consume comma
        tq.consume_eq(Token::Comma)?;

        // consume optional integer (max)
        let max = match *tq.peek()? {
            Token::IntLiteral(val) => {
                tq.increment();
                Some(val)
            }
            Token::CAngle => None,
            _ => return Err(ParseError::new("")),
        };

        // consume '>'
        tq.consume_eq(Token::CAngle)?;

        return Ok(Self { min: min, max: max });
    }
}

/// Test the parsing functionality
#[test]
fn parse_test() -> Result<(), Box<dyn Error>> {
    let lexer = setup_test_lexer();

    let mut tq = TokenQueue::new(lexer.lex("<5,10>")?);

    assert!(
        IntRange::try_from(&mut tq)?
            == IntRange {
                min: Some(5),
                max: Some(10)
            }
    );

    Ok(())
}
