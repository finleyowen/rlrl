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
    pos: usize,
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
        let mut match_len: Vec<usize> = vec![0; s.len()];
        let mut matches: Vec<LexerMatch<T>> = Vec::new();

        // for each rule
        for rule in &self.rules {
            // for each match of the rule's regex against the input
            for re_match in rule.pat.find_iter(s) {
                let mut takes_priority = true;
                // for each position in the match
                for i in re_match.start()..re_match.end() {
                    // if longer or equal-length match already occupies this
                    // position
                    if match_len[i] >= re_match.len() {
                        // a match that was already found has a length gte this
                        // one
                        takes_priority = false;
                        // stop looking for overlapping matches because we're
                        // not keeping this match anyway
                        break;
                    }
                }
                if takes_priority {
                    // got through the loop without finding an overlapping
                    // match - update the match_len array
                    for i in re_match.start()..re_match.end() {
                        match_len[i] = re_match.len();
                    }
                    // try handling the match and adding it to the list
                    match rule.handle(re_match) {
                        LexResult::Token(t) => matches.push(LexerMatch {
                            token: t,
                            pos: re_match.start(),
                        }),
                        LexResult::Ignore => {}
                        LexResult::Error(e) => return Err(e),
                    }
                }
            }
        }

        // sort matches by start location
        matches.sort_by(|a, b| a.pos.cmp(&b.pos));

        Ok(matches
            .into_iter()
            .map(|lexer_match| lexer_match.token)
            .collect())
    }
}

#[cfg(test)]
mod tests {}
