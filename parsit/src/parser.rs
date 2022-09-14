use logos::Logos;
use crate::lexer::LexIt;
use crate::ParseError;
use crate::ParseError::{ReachedEOF, UnreachedEOF};
use crate::step::StepResult;
use crate::step::StepResult::{Error, Fail, Success};

#[derive(Debug)]
pub struct ParseIt<'a, T> where T: Logos<'a, Source=str>, {
    lexer: LexIt<'a, T>,
}

impl<'a, Token> ParseIt<'a, Token>
    where Token: Logos<'a, Source=str> + PartialEq,
{
    pub fn new(src: &'a str) -> Result<Self, ParseError<'a>>
        where Token::Extras: Default
    {
        Ok(ParseIt {
            lexer: LexIt::new(src)?,
        })
    }

    pub fn token(&self, pos: usize) -> Result<(&Token, usize), ParseError<'a>> {
        self.lexer.token(pos)
    }
    pub fn one_or_more<T, Then>(&self, pos: usize, then: Then) -> StepResult<'a, Vec<T>>
        where
            Then: FnOnce(usize) -> StepResult<'a, T> + Copy,
    {
        match self.zero_or_more(pos, then) {
            Success(vals, _) if vals.is_empty() => Fail(pos),
            other => other,
        }
    }

    pub fn zero_or_more<T, Then>(&self, pos: usize, then: Then) -> StepResult<'a, Vec<T>>
        where
            Then: FnOnce(usize) -> StepResult<'a, T> + Copy,
    {
        match then(pos).then_multi_zip(|p| then(p)).merge() {
            Fail(_) => Success(vec![], pos),
            Error(ReachedEOF(_)) => Success(vec![], pos),
            success => success,
        }
    }

    pub fn validate_eof<T>(&self, res: StepResult<'a, T>) -> StepResult<'a, T> {
        match res {
            Success(_, pos) if self.lexer.len() != pos => Error(UnreachedEOF(pos)),
            other => other,
        }
    }
}

/// The token is used as a stub for the parsing operations when we need just a notion
/// that the token is parsed correctly but we don't need to process any values.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EmptyToken {}

/// Helps to parse a token on the lexer level
///  - Typically, it takes a token from the parser(defined in the structure of Logos)
///  - The structure should implement `PartialEq`
///
/// # Examples
/// - create a pattern matching for the given tokens
/// ```
///     use logos::Logos;
///     use crate::parsit::parser::ParseIt;
///     use crate::parsit::token;
///     use crate::parsit::step::StepResult;
///     use crate::parsit::parser::EmptyToken;
///     #[derive(Logos,PartialEq)]
///     pub enum TFQ {
///         #[token("true")]
///         True,
///         #[token("false")]
///         False,
///
///         #[token("?")]
///         Question,
///
///
///         #[error]
///         Error,
///     }
///
///     let p:ParseIt<TFQ> = ParseIt::new("true?").unwrap();
///     // create a pattern matching for the given tokens
///      token!(
///         p.token(0) =>
///             TFQ::True => true,
///             TFQ::False => false
///      );
///     // create a matching for only one token without a result (*it is used oftenly with then()*).
///     // The EmptyToken will be return
///      token!(p.token(1) => TFQ::Question);
///
/// ```
///
#[macro_export]
macro_rules! token {
  ($obj:expr => $($matcher:pat $(if $pred:expr)* => $result:expr),*) => {
      match $obj {
            Ok((t,p)) => match t {
                $($matcher $(if $pred)* => StepResult::Success($result, p + 1)),*,
                _ => StepResult::Fail(p)
            }
            Err(e) => StepResult::Error(e)
        }

   };
  ($obj:expr => $($matcher:pat $(if $pred:expr)*),*) => {
      match $obj {
            Ok((t,p)) => match t {
                $($matcher $(if $pred)* => StepResult::Success(EmptyToken{}, p + 1)),*,
                _ => StepResult::Fail(p)
            }
            Err(e) => StepResult::Error(e)
        }

   }

}