use std::cmp::min;
use std::fmt::Debug;
use std::rc::Rc;

const TOKEN_QUEUE_EMPTY_MSG: &str = "Couldn't get token from empty TokenQueue!";
const TOKEN_DID_NOT_MATCH_MSG: &str = "Token didn't match required format!";

/// A function that parses an item of type `T` from a queue of tokens with type
/// `L`
pub type ParseFn<L, T> = fn(&TokenQueue<L>) -> ParseResult<T>;

pub type ParseWithFn<L, C, T> = fn(&TokenQueue<L>, &C) -> ParseResult<T>;

/// Convenience type to return from parse functions
pub type ParseResult<T> = anyhow::Result<(T, usize)>;

/// Wrapper around `Vec<T>` exposing the functionality needed for
/// parsing.
#[derive(Clone)]
pub struct TokenQueue<T> {
    tokens: Rc<Vec<T>>,
    idx: usize,
}

impl<T> TokenQueue<T> {
    /// Borrow the front token from the queue.
    pub fn peek(&self) -> anyhow::Result<&T> {
        self.tokens
            .get(self.idx)
            .ok_or(anyhow::anyhow!(TOKEN_QUEUE_EMPTY_MSG))
    }

    /// Consume the front token in the queue.
    pub fn consume(&mut self) -> anyhow::Result<&T> {
        self.increment();
        self.prev()
    }

    /// Borrow the front token if it returns `true` when passed to `f`,
    /// otherwise return an error.
    pub fn peek_matching(&self, f: fn(&T) -> bool) -> anyhow::Result<&T> {
        let token = self.peek()?;
        if !f(token) {
            return Err(anyhow::anyhow!(TOKEN_DID_NOT_MATCH_MSG));
        }
        Ok(token)
    }

    /// Borrow the last token consumed.
    pub fn prev(&self) -> anyhow::Result<&T> {
        self.tokens
            .get(self.idx - 1)
            .ok_or(anyhow::anyhow!("Couldn't read prev token in TokenQueue."))
    }

    /// Consume the front token if it returns `true` when passed to `f`,
    /// otherwise return an error.
    pub fn consume_matching(
        &mut self,
        f: fn(&T) -> bool,
    ) -> anyhow::Result<&T> {
        if !self.peek().map_or(false, f) {
            return Err(anyhow::anyhow!(TOKEN_DID_NOT_MATCH_MSG));
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
    /// Parse a value of type `T` from the token queue with tokens of type `L`.
    /// Update the token queue's index with the index returned by the
    /// `parse_fn`.
    pub fn parse<T>(&mut self, parse_fn: ParseFn<L, T>) -> anyhow::Result<T> {
        let (val, index) = parse_fn(self)?;
        self.go_to(index);
        Ok(val)
    }

    /// Parse a value of type `T` from the token queue with tokens of type `L`,
    /// supporting a borrowed context parameter of type `C` which is passed.
    /// Update the token queue's index with the index returned by the
    /// `parse_fn`.
    pub fn parse_with<T, C>(
        &mut self,
        parse_with_fn: ParseWithFn<L, C, T>,
        context: &C,
    ) -> anyhow::Result<T> {
        let (val, index) = parse_with_fn(self, context)?;
        self.go_to(index);
        Ok(val)
    }
}

impl<T: PartialEq> TokenQueue<T> {
    /// Consume a token that is equal to token `token`, returning an error if the
    /// front token in the queue doesn't equal `token`.
    pub fn consume_eq(&mut self, token: T) -> anyhow::Result<()> {
        if self.peek()? == &token {
            self.increment();
            return Ok(());
        }
        Err(anyhow::anyhow!("Couldn't consume a "))
    }
}

impl<T> From<Vec<T>> for TokenQueue<T> {
    fn from(value: Vec<T>) -> Self {
        Self {
            tokens: Rc::new(value),
            idx: 0,
        }
    }
}

impl<T> Debug for TokenQueue<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for token in
            &self.tokens[self.idx..min(self.tokens.len(), self.idx + 20)]
        {
            write!(f, "{:?}", token)?;
        }
        Ok(())
    }
}
