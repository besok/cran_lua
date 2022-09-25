use logos::{FilterResult, Lexer, Logos};
use logos::skip;
use crate::parser::ast::Number;


#[derive(Logos, Clone, Copy, Debug, PartialEq)]
#[logos(subpattern digit = r"[0-9]([0-9_]*[0-9])?")]
#[logos(subpattern letter = r"[a-zA-Z_]")]
#[logos(subpattern exp = r"[eE][+-]?[0-9]+")]
pub enum Token<'a> {
    #[regex(r"(?&letter)((?&letter)|(?&digit))*")]
    Id(&'a str),

    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#,parse_qt_lit)]
    #[regex(r"\[=*\[", parse_block_text)]
    #[regex(r#"'([^'\\]|\\t|\\u|\\n|\\')*'"#,parse_qt_lit)]
    StringLit(&'a str),

    #[regex(r"-?(?&digit)", number)]
    #[regex(r"-?(?&digit)(?&exp)", number)]
    #[regex(r"-?(?&digit)?\.(?&digit)(?&exp)?[fFdD]?", float)]
    #[regex(r"0[bB][01][01]*", binary)]
    #[regex(r"-?0x[0-9a-f](([0-9a-f]|[_])*[0-9a-f])?", hex)]
    Digit(Number),

    #[token("and")]
    And,
    #[token("break")]
    Break,
    #[token("do")]
    Do,
    #[token("else")]
    Else,
    #[token("elseif")]
    Elseif,
    #[token("end")]
    End,
    #[token("false")]
    False,
    #[token("for")]
    For,
    #[token("function")]
    Function,
    #[token("goto")]
    Goto,
    #[token("if")]
    If,
    #[token("in")]
    In,
    #[token("local")]
    Local,
    #[token("nil")]
    Nil,
    #[token("not")]
    Not,
    #[token("or")]
    Or,
    #[token("repeat")]
    Repeat,
    #[token("return")]
    Return,
    #[token("then")]
    Then,
    #[token("true")]
    True,
    #[token("until")]
    Until,
    #[token("while")]
    While,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,

    #[token("*")]
    Mult,
    #[token("/")]
    Div,
    #[token("//")]
    FDiv,

    #[token("%")]
    Mod,
    #[token("^")]
    Caret,
    #[token("#")]
    Hash,

    #[token("&")]
    Ampersand,
    #[token("~")]
    Tilde,

    #[token("|")]
    Stick,
    #[token(">>")]
    RShift,

    #[token("<<")]
    LShift,
    #[token("==")]
    Eq,
    #[token("~=")]
    TEq,
    #[token(">")]
    Gt,
    #[token(">=")]
    Ge,
    #[token("<")]
    Lt,
    #[token("<=")]
    Le,
    #[token("=")]
    Assign,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBrack,
    #[token("]")]
    RBrack,
    #[token("::")]
    DColon,
    #[token(":")]
    Colon,
    #[token(";")]
    Semi,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("..")]
    EllipsisIn,
    #[token("...")]
    EllipsisOut,

    #[regex(r"#![^\r\n]*", skip)]
    #[regex(r"--\[?=*[^=|\[\r\n]*[\r\n]?", skip)]
    Comment,

    #[regex(r"--\[=*\[", parse_line_comment)]
    LineComment,

    #[regex(r"[ \t\u000C\r\n]+", skip)]
    WS,

    #[error]
    Error,
}


fn parse_line_comment<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> FilterResult<()> {
    let prefix: &str = lexer.slice();
    let suffix = prefix.replace("[", "]");
    let suffix = suffix.strip_prefix("--");
    let suffix: &str = suffix.expect("unreachable in parsing line comments");

    lexer
        .remainder()
        .find(suffix)
        .map(|i| lexer.bump(i + suffix.len()))
        .map(|_| FilterResult::Skip)
        .unwrap_or(FilterResult::Error)
}
fn parse_block_text<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> FilterResult<&'a str> {
    let prefix: &str = lexer.slice();
    let suffix = &prefix.replace("[", "]");

    lexer
        .remainder()
        .find(suffix)
        .map(|i| {
            let text = &lexer.remainder()[0..i];
            lexer.bump(i + suffix.len());
            text
        })
        .map(|s| FilterResult::Emit(s))
        .unwrap_or(FilterResult::Error)
}
fn parse_qt_lit<'a>(lexer: &mut Lexer<'a, Token<'a>>) ->  &'a str {
    let qt_lit: &str = lexer.slice();
    &qt_lit[1..qt_lit.len() - 1]
}


fn number<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, String> {
    lex.slice()
        .parse::<i64>()
        .map(|r| Number::Int(r))
        .map_err(|s| s.to_string())
}

fn float<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, String> {
    lex.slice()
        .parse::<f64>()
        .map(|r| Number::Float(r))
        .map_err(|s| s.to_string())
}

fn binary<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, String> {
    isize::from_str_radix(&lex.slice()[2..], 2)
        .map(Number::Binary)
        .map_err(|s| s.to_string())
}

fn hex<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, String> {
    i64::from_str_radix(lex.slice().trim_start_matches("0x"), 16)
        .map(|r| Number::Hex(r))
        .map_err(|s| s.to_string())
}



#[cfg(test)]
mod tests {
    use parsit::test::lexer_test as lt;
    use crate::parser::ast::Number;
    use crate::parser::tokens::Token;

    #[test]
    fn comments() {
        lt::expect::<Token>(r#"
        #! some
        text"#, vec![Token::Id("text")]);

        lt::expect_succeed::<Token>(r#"--A"#);
        lt::expect_succeed::<Token>(r#"--A
        "#);
        lt::expect_succeed::<Token>(r#"--[==
        "#);
        lt::expect_succeed::<Token>(r#"--[==AA"#);
        lt::expect_succeed::<Token>(r#"--AA"#);


        lt::expect::<Token>(
            r#"--[[hjasgdkjasd
            askldhfklsdf
            ]]
            a"#, vec![Token::Id("a")]);

        lt::expect::<Token>(
            r#"--[==[hjasgdkjasd
            askldhfklsdf
            ]==]
            a"#, vec![Token::Id("a")])
    }
    #[test]
    fn text() {
        lt::expect::<Token>(r#"
        #! some
        "text""#, vec![Token::StringLit("text")]);

        lt::expect::<Token>("\"te\\\"xt\"", vec![Token::StringLit("te\\\"xt")]);
        lt::expect::<Token>("'te\\'xt'", vec![Token::StringLit("te\\'xt")]);

        lt::expect::<Token>(
            r#"[==[hjasgdkjasd
            askldhfklsdf
            ]==]"#, vec![Token::StringLit("hjasgdkjasd\n            askldhfklsdf\n            ")])
    }
    #[test]
    fn number() {
        lt::expect::<Token>(r#"1"#, vec![Token::Digit(Number::Int(1))]);
        lt::expect::<Token>(r#"1.1"#, vec![Token::Digit(Number::Float(1.1))]);
        lt::expect::<Token>(r#"1000000.000001"#, vec![Token::Digit(Number::Float(1000000.000001))]);
        lt::expect::<Token>(r#"1e-1"#, vec![Token::Digit(Number::Float(1000000.000001))]);

    }



}

