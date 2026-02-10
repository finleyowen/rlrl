use std::error::Error;
use std::fmt::Display;
use std::rc::Rc;

const SYNTAX_ERROR_MSG: &str = "Syntax error!";

/// Convenience type implementing [std::error::Error] storing an error message.
#[derive(Debug)]
pub struct ParseError(String);

impl ParseError {
    pub fn new(msg: &str) -> Self {
        Self(msg.into())
    }
}

impl<'a> Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError: {}", self.0)
    }
}

impl<'a> Error for ParseError {}

/// Wrapper around `Vec<T>` exposing the functionality needed for
/// parsing.
pub struct TokenQueue<T> {
    tokens: Rc<Vec<T>>,
    idx: usize,
}

impl<T> TokenQueue<T> {
    /// Create a new [TokenQueue].
    pub fn new(tokens: Vec<T>) -> Self {
        Self {
            tokens: Rc::new(tokens),
            idx: 0,
        }
    }

    /// Borrow the front token from the queue.
    pub fn peek(&self) -> Result<&T, ParseError> {
        self.tokens
            .get(self.idx)
            .ok_or(ParseError::new(SYNTAX_ERROR_MSG))
    }

    /// Borrow the front token if it returns `true` when passed to `f`,
    /// otherwise return an error.
    pub fn peek_matching(&self, f: fn(&T) -> bool) -> Result<&T, ParseError> {
        let token = self.peek()?;
        if !f(token) {
            return Err(ParseError::new(SYNTAX_ERROR_MSG));
        }
        Ok(token)
    }

    /// Go to the next token by incrementing the index.
    pub fn increment(&mut self) {
        self.idx += 1
    }

    /// Go to the token at position `i`.
    pub fn go_to(&mut self, i: usize) {
        self.idx = i;
    }
}

impl<T: PartialEq> TokenQueue<T> {
    /// Consume a token that is equal to token `token`, returning an error if the
    /// front token in the queue doesn't equal `token`.
    pub fn consume_token(&mut self, token: T) -> Result<(), ParseError> {
        if self.peek()? == &token {
            self.increment();
            return Ok(());
        }
        Err(ParseError::new(SYNTAX_ERROR_MSG))
    }
}

impl<T> Clone for TokenQueue<T> {
    fn clone(&self) -> Self {
        Self {
            tokens: self.tokens.clone(),
            idx: self.idx,
        }
    }
}

#[cfg(test)]
mod tests {}
