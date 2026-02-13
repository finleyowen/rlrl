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

    /// Consume the front token in the queue.
    pub fn consume(&mut self) -> Result<&T, ParseError> {
        self.increment();
        self.prev()
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

    /// Borrow the last token consumed.
    pub fn prev(&self) -> Result<&T, ParseError> {
        self.tokens
            .get(self.idx - 1)
            .ok_or(ParseError::new(SYNTAX_ERROR_MSG))
    }

    /// Consume the front token if it returns `true` when passed to `f`,
    /// otherwise return an error.
    pub fn consume_matching(
        &mut self,
        f: fn(&T) -> bool,
    ) -> Result<&T, ParseError> {
        if !self.peek().map_or(false, f) {
            return Err(ParseError::new(SYNTAX_ERROR_MSG));
        }
        self.increment();
        Ok(self.prev()?)
    }

    /// Go to the next token by incrementing the index.
    pub fn increment(&mut self) {
        self.idx += 1
    }

    /// Go to the token at position `i`.
    pub fn go_to(&mut self, i: usize) {
        self.idx = i;
    }

    /// Get the index of the current token.
    pub fn get_idx(&self) -> usize {
        self.idx
    }

    /// Return true when the token queue has no tokens left.
    pub fn is_consumed(&self) -> bool {
        self.idx == self.tokens.len()
    }
}

impl<L> TokenQueue<L> {
    /// Parse a value of type `T` from the token queue. Update the token queue's
    /// index with the index returned by the `parse_fn`.
    pub fn parse<T>(
        &mut self,
        parse_fn: ParseFn<L, T>,
    ) -> Result<T, Box<dyn Error>> {
        let (val, index) = parse_fn(self)?;
        self.go_to(index);
        Ok(val)
    }
}

impl<T: PartialEq> TokenQueue<T> {
    /// Consume a token that is equal to token `token`, returning an error if the
    /// front token in the queue doesn't equal `token`.
    pub fn consume_eq(&mut self, token: T) -> Result<(), ParseError> {
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

/// Convenience type to return from parse functions
pub type ParseResult<T> = Result<(T, usize), Box<dyn Error>>;

/// A function that parses an item of type `T` from a queue of tokens with type
/// `L`
pub type ParseFn<L, T> = fn(&TokenQueue<L>) -> ParseResult<T>;

#[cfg(test)]
mod tests {}
