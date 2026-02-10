use crate::lex::LexResult;
use crate::lex::Lexer;
use crate::parse::ParseError;
use crate::parse::TokenQueue;
use std::collections::LinkedList;
use std::error::Error;
use std::i32;

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
    pub fn is_ident_tok(&self) -> bool {
        if let Self::Ident(_) = self {
            return true;
        }
        false
    }

    pub fn get_ident(&self) -> Result<&String, ParseError> {
        if let Self::Ident(ident) = self {
            return Ok(ident);
        }
        Err(ParseError::new(""))
    }
}

#[derive(PartialEq)]
enum DataTypeInfo {
    Int(Option<i32>, Option<i32>),
    Double(Option<f64>, Option<f64>),
    String(Option<i32>, Option<i32>),
}

impl TryFrom<&mut TokenQueue<Token>> for DataTypeInfo {
    type Error = ParseError;

    fn try_from(value: &mut TokenQueue<Token>) -> Result<Self, Self::Error> {
        let ident_tok = value.pop_matching(|tok| tok.is_ident_tok())?;
        let ident = ident_tok.get_ident()?;

        match ident.as_str() {
            "int" => Ok(DataTypeInfo::Int(None, None)),
            "double" => Ok(DataTypeInfo::Double(None, None)),
            "str" => Ok(DataTypeInfo::String(None, None)),
            _ => Err(ParseError::new("Couldn't parse type")),
        }
    }
}

// setup the lexer for testing
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
        LexResult::Error(format!("Unmatched input at position {}", re_match.start()).into())
    });

    lexer
}

// test the lexer
#[test]
fn lexer_test() {
    let lexer = setup_test_lexer();

    assert!(
        lexer.lex("({})").unwrap()
            == LinkedList::from([Token::OParen, Token::OBrace, Token::CBrace, Token::CParen])
    );

    assert!(
        lexer.lex("({}, {})").unwrap()
            == LinkedList::from([
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
            == LinkedList::from([
                Token::FnKwd,
                Token::Ident("my_function".into()),
                Token::OParen,
                Token::CParen,
                Token::OBrace,
                Token::CBrace
            ])
    );

    assert!(
        lexer.lex("type i1to5 = int<1,5>").unwrap()
            == LinkedList::from([
                Token::TypeKwd,
                Token::Ident("i1to5".into()),
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

#[test]
fn token_queue_test() -> Result<(), Box<dyn Error>> {
    let lexer = setup_test_lexer();

    let mut tq = TokenQueue::new(lexer.lex("int<,>")?);

    assert!(
        DataTypeInfo::try_from(&mut tq).expect("Didn't parse") == DataTypeInfo::Int(None, None)
    );

    Ok(())
}
