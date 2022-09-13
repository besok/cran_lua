mod lexer;
pub mod step;
pub mod parser;

use std::ops::Range;

#[derive(Debug, Clone)]
pub enum ParseError<'a> {
    BadToken(&'a str, Range<usize>),
    FailedOnValidation(&'a str, usize),
    FinishedOnFail,
    ReachedEOF(usize),
    UnreachedEOF(usize),
}




#[cfg(test)]
mod tests {
    use crate::parser::ParseIt;
    use logos::Logos;
    use crate::token;
    use crate::step::StepResult;
    use crate::parser::EmptyToken;
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
        let result= ParseIt::<Token>::new("as break");
        println!("{:?}",result.unwrap());

    }
}
