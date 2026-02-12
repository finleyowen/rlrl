use regex::Match;
use regex::Regex;
use std::error::Error;
use std::fmt::Display;

/// Convenience type implementing [std::error::Error] storing an error message.
#[derive(Debug)]
pub struct LexError<'a>(&'a str);

impl<'a> Display for LexError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LexError: {}", self.0)
    }
}

impl<'a> Error for LexError<'a> {}

/// Represents possible outcomes when trying to lex a token of type `T`.
pub enum LexResult<T> {
    /// A token was successfully lexed from the input
    Token(T),
    /// The input was ignored
    Ignore,
    /// An error occurred lex the token
    Error(Box<dyn Error>),
}

/// Function that accepts a [regex::Match] and tries to lex a token of type `T`
/// from it.
pub type Handler<T> = fn(Match) -> LexResult<T>;

/// Represents a rule in a lexer that lexes tokens of type `T`.
pub struct LexerRule<T> {
    pat: Regex,
    handler: Handler<T>,
}

impl<T> LexerRule<T> {
    fn handle(&self, re_match: Match) -> LexResult<T> {
        (self.handler)(re_match)
    }
}

/// Represents a match discovered during lexing.
pub struct LexerMatch<T> {
    token: T,
    start: usize,
    len: usize,
}

/// Represents a lexer that lexes tokens of type `T`.
pub struct Lexer<T> {
    rules: Vec<LexerRule<T>>,
}

impl<T> Lexer<T> {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, pat: &str, handler: Handler<T>) {
        self.rules.push(LexerRule {
            pat: Regex::new(pat)
                .expect("Invalid regexp passed to Lexer::add_rule"),
            handler,
        });
    }

    pub fn lex(&self, s: &str) -> Result<Vec<T>, Box<dyn Error>> {
        let mut match_info: Vec<(usize, usize)> = vec![(0, 0); s.len()];
        let mut matches: Vec<LexerMatch<T>> = Vec::new();

        // for each rule
        for rule in &self.rules {
            // for each match of the rule's regex against the input
            for re_match in rule.pat.find_iter(s) {
                let mut takes_priority = true;
                // for each position in the match
                for i in re_match.start()..re_match.end() {
                    // extract info about conflicting match
                    let (confl_start, confl_len) = match_info[i];
                    // note confl_len = 0 if no conflicting match exists
                    if confl_len >= re_match.len() {
                        // a match that was already found has a length gte this
                        // one
                        takes_priority = false;
                        // stop looking for overlapping matches because we're
                        // not keeping this match anyway
                        break;
                    } else if confl_len > 0 {
                        // a match already exists and it's shorter than this
                        // one => remove it from the arrays
                        for i in confl_start..confl_start + confl_len {
                            match_info[i] = (0, 0);
                        }
                        matches = matches
                            .into_iter()
                            .filter(|lexer_match| {
                                !(lexer_match.start == confl_start
                                    && lexer_match.len == confl_len)
                            })
                            .collect();
                    }
                }
                if takes_priority {
                    // got through the loop without finding an overlapping
                    // match - update the match_len array
                    for i in re_match.start()..re_match.end() {
                        match_info[i] = (re_match.start(), re_match.len());
                    }
                    // try handling the match and adding it to the list
                    match rule.handle(re_match) {
                        LexResult::Token(t) => matches.push(LexerMatch {
                            token: t,
                            start: re_match.start(),
                            len: re_match.len(),
                        }),
                        LexResult::Ignore => {}
                        LexResult::Error(e) => return Err(e),
                    }
                }
            }
        }

        // sort matches by start location
        matches.sort_by(|a, b| a.start.cmp(&b.start));

        Ok(matches
            .into_iter()
            .map(|lexer_match| lexer_match.token)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::lex::{LexResult, Lexer};
    use std::error::Error;

    #[derive(PartialEq, Debug)]
    enum Token {
        IntLiteral(i32),
        DblLiteral(f64),
    }

    fn setup_lexer() -> Lexer<Token> {
        let mut lexer = Lexer::new();

        lexer.add_rule(r"[\s\t\n]", |_| LexResult::Ignore);
        lexer.add_rule(r"\-?[0-9]+", |int_match| {
            match int_match.as_str().parse::<i32>() {
                Ok(val) => LexResult::Token(Token::IntLiteral(val)),
                Err(err) => LexResult::Error(err.into()),
            }
        });
        lexer.add_rule(r"\-?[0-9]+(\.[0-9]+)", |dbl_match| {
            match dbl_match.as_str().parse::<f64>() {
                Ok(val) => LexResult::Token(Token::DblLiteral(val)),
                Err(err) => LexResult::Error(err.into()),
            }
        });

        lexer
    }

    #[test]
    fn test_lexer() -> Result<(), Box<dyn Error>> {
        let lexer = setup_lexer();

        assert!(
            lexer.lex("9 0.9 1.0")?
                == vec![
                    Token::IntLiteral(9),
                    Token::DblLiteral(0.9),
                    Token::DblLiteral(1.0)
                ]
        );

        Ok(())
    }
}
