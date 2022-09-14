mod lexer;
pub mod step;
pub mod parser;

use std::ops::Range;

/// Parsing error.
#[derive(Debug, Clone)]
pub enum ParseError<'a> {
    /// The token is bad and apparently the error is on the level of lexing
    BadToken(&'a str, Range<usize>),
    /// When the validation is not working
    ///
    /// # Examples
    ///
    /// ```
    /// use parsit::ParseError;
    /// use parsit::step::StepResult;
    ///
    ///  let res = StepResult::Success("", 1);
    ///  let res = res.validate(|v| {
    ///        if v.len() > 0 { Ok(()) } else { Err("empty") }
    ///  });
    ///
    ///  if let Some(v) = res.map_error(|e| match e {
    ///        ParseError::FailedOnValidation(v,_) => v,
    ///        _other => ""
    ///  }){ assert_eq!(v, "empty")} else { assert!(false) };
    ///
    ///
    ///
    /// ```
    ///
    ///
    FailedOnValidation(&'a str, usize),
    /// When the last token is fail. It happens when the backtracking does not have a positive variant.
    FinishedOnFail,
    /// When the token stream is empty but the parser expects other tokens
    ReachedEOF(usize),
    /// When the token stream si not empty and parser does not expect anything.
    UnreachedEOF(usize),
}


#[cfg(test)]
mod tests {
    use crate::parser::ParseIt;
    use crate::token;
    use crate::step::StepResult;
    use crate::parser::EmptyToken;
    use crate::ParseError;
    use logos::Logos;

    #[derive(Logos, Debug, Copy, Clone, PartialEq)]
    pub enum Token {
        #[token("as")]
        As,
        #[token("break")]
        Break,
        #[regex(r"[ \t\r\n\u000C\f]+", logos::skip)]
        Whitespace,
        #[error]
        Error,
    }


    #[test]
    fn it_works() {
        let result = ParseIt::<Token>::new("as break");
        println!("{:?}", result.unwrap());
    }

    #[derive(Logos,PartialEq)]
    pub enum TrueFalse {
        #[token("true")]
        True,
        #[token("false")]
        False,

        #[error]
        Error,
    }

    #[test]
    fn it_works2() {


          let p:ParseIt<TrueFalse> = ParseIt::new("true").unwrap();

            token!(
                p.token(0) =>
                    TrueFalse::True => true,
                    TrueFalse::False => false
            );
    }

    #[test]
    fn it_works1() {
        let res = StepResult::Success("", 1);
        let res = res.validate(|v| {
            if v.len() > 0 { Ok(()) } else { Err("empty") }
        });

        if let Some(v) = res.map_error(|e| match e {
            ParseError::FailedOnValidation(v, _) => v,
            _other => ""
        }) { assert_eq!(v, "empty") } else { assert!(false) };
    }


}
