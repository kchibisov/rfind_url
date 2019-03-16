//! This crate provides a parser to search a string for URLs **in reverse order**.
//!
//! All functionality is handled by the [`Parser`] struct which takes [`chars`] as input.
//!
//! # Examples
//!
//! Text can be fed into the parser in reverse order:
//!
//! ```
//! use rfind_url::Parser;
//!
//! let mut parser = Parser::new();
//!
//! for c in "There is no URL here.".chars().rev() {
//!     assert_eq!(parser.advance(c), None);
//! }
//! ```
//!
//! The parser returns the length of the URL as soon as the last character of the URL is pushed
//! into it. Otherwise it will return [`None`]:
//!
//! ```
//! use rfind_url::Parser;
//!
//! let mut parser = Parser::new();
//!
//! // Parser did not find any URLs
//! assert_eq!(parser.advance(' '), None);
//!
//! // URLs are only returned once they are complete
//! for c in "ttps://example.org".chars().rev() {
//!     assert_eq!(parser.advance(c), None);
//! }
//!
//! // Parser has detected a URL spanning the last 19 characters
//! assert_eq!(parser.advance('h'), Some(19));
//! ```
//!
//! [`Parser`]: struct.Parser.html
//! [`chars`]: https://doc.rust-lang.org/std/primitive.char.html
//! [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None

#![cfg_attr(all(test, feature = "bench"), feature(test))]
#![no_std]

#[cfg(test)]
mod tests;

/// Allowed URL schemes.
pub(crate) const SCHEMES: [&str; 8] = [
    "http", "https", "mailto", "news", "file", "git", "ssh", "ftp",
];

/// URL parser states.
enum State {
    Default,
    Path,
    SchemeFirstSlash,
    SchemeSecondSlash,
    Scheme,
}

impl Default for State {
    #[inline]
    fn default() -> Self {
        State::Default
    }
}

/// State machine for finding URLs.
///
/// The URL parser takes characters of a string **in reverse order** and returns the length of the
/// URL whenever finding one.
#[derive(Default)]
pub struct Parser {
    pub(crate) scheme_indices: [u8; 8],
    state: State,
    len: u16,
}

impl Parser {
    /// Creates a new URL parser.
    ///
    /// # Examples
    ///
    /// ```
    /// use rfind_url::Parser;
    ///
    /// let mut parser = Parser::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Advances the parser by one character.
    ///
    /// # Examples
    ///
    /// ```
    /// use rfind_url::Parser;
    ///
    /// let mut parser = Parser::new();
    ///
    /// // Parser did not find any URLs
    /// assert_eq!(parser.advance(' '), None);
    ///
    /// // URLs are only returned once they are complete
    /// for c in "ttps://example.org".chars().rev() {
    ///     assert_eq!(parser.advance(c), None);
    /// }
    ///
    /// // Parser has detected a URL spanning the last 19 characters
    /// assert_eq!(parser.advance('h'), Some(19));
    /// ```
    #[inline]
    pub fn advance(&mut self, c: char) -> Option<u16> {
        self.len += 1;

        if is_illegal(c) {
            self.reset();
            return None;
        }

        match self.state {
            State::Default => self.advance_default(c),
            State::Path => self.advance_path(c),
            State::SchemeFirstSlash => self.advance_scheme_first_slash(c),
            State::SchemeSecondSlash => self.advance_scheme_second_slash(c),
            State::Scheme => return self.advance_scheme(c),
        }

        None
    }

    #[inline]
    fn reset(&mut self) {
        self.scheme_indices = [0; 8];
        self.state = State::Default;
        self.len = 0;
    }

    #[inline]
    fn advance_default(&mut self, c: char) {
        match c {
            '.'...'/' | ',' | ':'...';' | '?' | '!' | '(' => self.reset(),
            _ => self.state = State::Path,
        }
    }

    #[inline]
    fn advance_path(&mut self, c: char) {
        if c == '/' {
            self.state = State::SchemeFirstSlash
        }
    }

    #[inline]
    fn advance_scheme_first_slash(&mut self, c: char) {
        if c == '/' {
            self.state = State::SchemeSecondSlash;
        } else {
            self.state = State::Path;
        }
    }

    #[inline]
    fn advance_scheme_second_slash(&mut self, c: char) {
        if c == ':' {
            self.state = State::Scheme;
        } else {
            self.state = State::Path;
        }
    }

    #[inline]
    fn advance_scheme(&mut self, c: char) -> Option<u16> {
        match c {
            'a'...'z' | 'A'...'Z' => {
                for (i, index) in self.scheme_indices.iter_mut().enumerate() {
                    let scheme_len = SCHEMES[i].len() as u8;

                    if *index >= scheme_len {
                        continue;
                    }

                    if SCHEMES[i].chars().rev().nth(*index as usize) != Some(c) {
                        *index = scheme_len + 1;
                    } else {
                        *index += 1;
                    }

                    // Returning early here is only possible because no scheme ends with another
                    // scheme. This is covered by the `no_scheme_conflicts` test.
                    if *index == scheme_len {
                        return Some(self.len);
                    }
                }
            }
            _ => self.reset(),
        }

        None
    }
}

#[inline]
fn is_illegal(c: char) -> bool {
    match c {
        '\u{00}'...'\u{1F}'
        | '\u{7F}'...'\u{9F}'
        | '<'
        | '>'
        | '"'
        | ' '
        | '{'...'}'
        | '\\'
        | '^'
        | '`' => true,
        _ => false,
    }
}
