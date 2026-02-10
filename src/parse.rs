use std::collections::LinkedList;
use std::error::Error;
use std::fmt::Display;

const MISSING_VALUE_MSG: &str = "Missing required value!";
const COULD_NOT_MATCH_MSG: &str = "No match found!";

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

/// Wrapper around `LinkedList<T>` exposing the functionality needed for
/// parsing.
pub struct TokenQueue<T> {
    tokens: LinkedList<T>,
}

impl<T> TokenQueue<T> {
    /// Create a new [TokenQueue].
    pub fn new(tokens: LinkedList<T>) -> Self {
        Self { tokens: tokens }
    }

    /// Pop a token from the queue.
    pub fn pop(&mut self) -> Result<T, ParseError> {
        self.tokens
            .pop_front()
            .ok_or(ParseError::new(MISSING_VALUE_MSG))
    }

    /// Pop the first token if the function `f` returns true when called on the
    /// token, otherwise return an error value.
    pub fn pop_matching(&mut self, f: fn(&T) -> bool) -> Result<T, ParseError> {
        let token = self
            .tokens
            .pop_front()
            .ok_or(ParseError::new(MISSING_VALUE_MSG))?;
        if !f(&token) {
            return Err(ParseError::new(COULD_NOT_MATCH_MSG));
        }
        Ok(token)
    }

    /// Borrow the front token from the queue.
    pub fn peek(&self) -> Result<&T, ParseError> {
        self.tokens
            .front()
            .ok_or(ParseError::new("Couldn't peek a required value!"))
    }

    /// Borrow the token at index `i` from the queue.
    pub fn peek_ith(&self, i: usize) -> Option<&T> {
        self.tokens.iter().nth(i)
    }

    /// Borrow the first `i` tokens from the queue.
    pub fn peek_i(&self, i: usize) -> Vec<&T> {
        self.tokens.iter().take(i).collect()
    }
}

#[cfg(test)]
mod tests {}
