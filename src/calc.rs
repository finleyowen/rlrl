#![allow(dead_code)]
use crate::prelude::*;

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Add,
    Sub,
    Mul,
    Div,
    Num(f64),
}

impl Token {
    fn get_num(&self) -> Option<f64> {
        match self {
            &Self::Num(val) => Some(val.clone()),
            _ => None,
        }
    }
}

type BoxedExpr = Box<Expr>;

#[derive(Debug, PartialEq)]
enum Op {
    Op(BoxedExpr, BoxedExpr),
    Inv(BoxedExpr, BoxedExpr),
}

impl Op {
    fn lhs(&self) -> &BoxedExpr {
        match self {
            Self::Op(lhs, _) => lhs,
            Self::Inv(lhs, _) => lhs,
        }
    }

    fn rhs(&self) -> &BoxedExpr {
        match self {
            Self::Op(lhs, _) => lhs,
            Self::Inv(lhs, _) => lhs,
        }
    }
}

#[derive(Debug, PartialEq)]
struct Mul(Op);

#[derive(Debug, PartialEq)]
struct Add(Op);

#[derive(Debug, PartialEq)]
enum Expr {
    Add(Add),
    Mul(Mul),
    Num(f64),
}

impl Expr {
    fn parse(tq: &TokenQueue<Token>) -> anyhow::Result<(Self, usize)> {
        let mut tq = tq.clone();

        let lhs =
            tq.consume()?
                .get_num()
                .ok_or::<anyhow::Error>(anyhow::anyhow!(
                    "Couldn't parse number where one was required!"
                ))?;

        // base case
        if tq.is_consumed() {
            return Ok((Expr::Num(lhs), tq.get_idx()));
        }

        // recursive case
        let op = tq.consume()?.clone(); // clone cheaply to avoid multiple mutable borrows
        let rhs: Expr = tq.parse(Expr::parse)?;

        // box both sides
        let lhs = Box::new(Expr::Num(lhs));
        let rhs = Box::new(rhs);

        match op {
            Token::Add => Ok((Expr::Add(Add(Op::Op(lhs, rhs))), tq.get_idx())),
            Token::Sub => Ok((Expr::Add(Add(Op::Inv(lhs, rhs))), tq.get_idx())),
            Token::Mul => Ok((Expr::Mul(Mul(Op::Op(lhs, rhs))), tq.get_idx())),
            Token::Div => Ok((Expr::Mul(Mul(Op::Inv(lhs, rhs))), tq.get_idx())),
            _ => Err(anyhow::anyhow!(
                "Couldn't parse operator where one was required!"
            )),
        }
    }

    fn get_num(&self) -> anyhow::Result<f64> {
        match self {
            Self::Num(val) => Ok(*val),
            _ => Err(anyhow::anyhow!(
                "Couldn't parse number when one was expected!"
            )),
        }
    }

    fn eval(&self) -> f64 {
        match self {
            Self::Num(val) => *val,
            _ => 0.0,
        }
    }
}

fn setup_lexer() -> Lexer<Token> {
    let mut lexer = Lexer::new();

    lexer.add_rule(r"[\s\t\n]+", |_| LexResult::Ignore);

    lexer.add_rule(r"\+", |_| LexResult::Token(Token::Add));
    lexer.add_rule(r"\-", |_| LexResult::Token(Token::Sub));
    lexer.add_rule(r"\*", |_| LexResult::Token(Token::Mul));
    lexer.add_rule(r"/", |_| LexResult::Token(Token::Div));

    lexer.add_rule(r"\-?[0-9]+(?:\.[0-9]+)?", |re_match| {
        match re_match.as_str().parse::<f64>() {
            Ok(val) => LexResult::Token(Token::Num(val)),
            Err(err) => LexResult::Error(err.into()),
        }
    });

    lexer
}

mod test {
    #![allow(unused_imports)]

    use super::*;
    use crate::prelude::*;

    fn parse_expr_from_str(s: &str) -> anyhow::Result<Expr> {
        let lexer = setup_lexer();
        let tokens = lexer.lex(s)?;
        let mut tq = TokenQueue::from(tokens);
        tq.parse(Expr::parse)
    }

    #[test]
    fn lexer_test() -> anyhow::Result<()> {
        let l = setup_lexer();

        let toks = l.lex("5 + 6")?;
        assert!(toks == vec![Token::Num(5.0), Token::Add, Token::Num(6.0)]);

        assert!(l.lex("5 & 6").is_err());

        Ok(())
    }

    #[test]
    fn parse_test() -> anyhow::Result<()> {
        let expr = parse_expr_from_str("5 + 6 - 2")?;

        assert!(
            expr == Expr::Add(Add(Op::Op(
                Expr::Num(5.0).into(),
                Expr::Add(Add(Op::Inv(
                    Expr::Num(6.0).into(),
                    Expr::Num(2.0).into()
                )))
                .into()
            )))
        );

        let expr = parse_expr_from_str("5 * 6 + 2")?;

        assert!(
            expr == Expr::Mul(Mul(Op::Op(
                Expr::Num(5.0).into(),
                Expr::Add(Add(Op::Op(
                    Expr::Num(6.0).into(),
                    Expr::Num(2.0).into()
                )))
                .into()
            )))
        );

        let expr = parse_expr_from_str("5 + 6 * 2")?;

        assert!(
            expr == Expr::Add(Add(Op::Op(
                Expr::Num(5.0).into(),
                Expr::Mul(Mul(Op::Op(
                    Expr::Num(6.0).into(),
                    Expr::Num(2.0).into()
                )))
                .into()
            )))
        );
        Ok(())
    }
}
