use std::collections::HashMap;
use std::fmt::{Display, format, Formatter};
use std::vec::IntoIter;
use crate::parser::ast::{BinaryType, Expression, Number, UnaryType};
use crate::parser::ast::BinaryType::*;


const fn expr_priority(tp: &BinaryType) -> (i32, i32) {
    match tp {
        Pov => (14, 13),
        Mult | Div | FDiv | Mod => (11, 11),
        Add | Sub => (10, 10),
        Concat => (9, 8),
        LShift | RShift => (7, 7),
        Amper => (6, 6),
        Tilde => (5, 5),
        Stick => (4, 4),
        Eq | Le | Lt | Gt | Ge | TEq => (3, 3),
        And => (2, 2),
        Or => (1, 1)
    }
}


pub(crate) fn fold_with_priority<'a>(first: Expression<'a>, elems: Vec<(BinaryType, Expression<'a>)>) -> Expression<'a> {
    fold(first, &mut Elems { elems }, 0)
}

// TODO reverse the vec
struct Elems<'a> {
    elems: Vec<(BinaryType, Expression<'a>)>,
}

impl<'a> Elems<'a> {
    fn peek(&self) -> Option<&(BinaryType, Expression<'a>)> {
        self.elems.get(0)
    }
    fn next(&mut self) -> (BinaryType, Expression<'a>) {
        self.elems.remove(0)
    }
}

/// pratt parsing algorithm
fn fold<'a>(lhs: Expression<'a>, elems: &mut Elems<'a>, min_priority: i32) -> Expression<'a> {
    let mut lhs = lhs;

    while let Some((tp, _)) = elems.peek() {
        let (l_prior, r_prior) = expr_priority(tp);
        if l_prior >= min_priority {
            let (tp, rhs) = elems.next();
            let rhs = fold(rhs, elems, r_prior);
            lhs = Expression::Binary(Box::new(lhs), tp, Box::new(rhs));
        } else { break; }
    }

    lhs
}


pub(crate) fn print(expr: &Expression) -> String {
    match expr {
        Expression::Nil => "nil".to_string(),
        Expression::False => "false".to_string(),
        Expression::True => "true".to_string(),
        Expression::Number(n) => format!("{}", n),
        Expression::Text(t) => t.text.to_string(),
        Expression::VarArgs => "...".to_string(),
        Expression::FnDef(_, _) => "fn_def".to_string(),
        Expression::PrefixExpr(_) => "pref".to_string(),
        Expression::TableConstructor(_) => "table".to_string(),
        Expression::Unary(s, e) => format!("{}{}", s, print(e)),
        Expression::Binary(lhs, op, rhs) => format!("({} {} {})", print(lhs), op, print(rhs))
    }
}


impl Display for BinaryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Mult => f.write_str("*"),
            Div => f.write_str("/"),
            FDiv => f.write_str("//"),
            Mod => f.write_str("%"),
            Add => f.write_str("+"),
            Sub => f.write_str("-"),
            Pov => f.write_str("^"),
            Concat => f.write_str(".."),
            Gt => f.write_str(">"),
            Ge => f.write_str(">="),
            Le => f.write_str("<="),
            Lt => f.write_str("<"),
            Eq => f.write_str("=="),
            TEq => f.write_str("~="),
            And => f.write_str("and"),
            Amper => f.write_str("&"),
            Stick => f.write_str("|"),
            Tilde => f.write_str("~"),
            LShift => f.write_str("<<"),
            RShift => f.write_str(">>"),
            Or => f.write_str("or"),
        }
    }
}

impl Display for UnaryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryType::Not => f.write_str("!"),
            UnaryType::Hash => f.write_str("#"),
            UnaryType::Minus => f.write_str("-"),
            UnaryType::Tilde => f.write_str("~"),
        }
    }
}

#[macro_export]
macro_rules! expr {
  () => {Expression::Nil};
  (f) => {Expression::False};
  (t) => {Expression::True};
  (i$e:literal) => {Expression::Number(Number::Int($e))};
  (f$e:literal) => {Expression::Number(Number::Float($e))};
  (text $e:literal) => {Expression::Text(Text{text:$e})};
  (...) => {Expression::VarArgs};
  (!$expr:expr) => {Expression::Unary(UnaryType::Not,Box::new($expr))};
  (#$expr:expr) => {Expression::Unary(UnaryType::Hash,Box::new($expr))};
  (-$expr:expr) => {Expression::Unary(UnaryType::Minus,Box::new($expr))};
  (~$expr:expr) => {Expression::Unary(UnaryType::Tilde,Box::new($expr))};
  ($lhs:expr, *, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Mult, Box::new($rhs))};
  ($lhs:expr, /, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Div, Box::new($rhs))};
  ($lhs:expr, d/, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::FDiv, Box::new($rhs))};
  ($lhs:expr, %, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Mod, Box::new($rhs))};
  ($lhs:expr, +, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Add, Box::new($rhs))};
  ($lhs:expr, -, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Sub, Box::new($rhs))};
  ($lhs:expr, ^, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Pov, Box::new($rhs))};
  ($lhs:expr, .., $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Concat, Box::new($rhs))};
  ($lhs:expr, >, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Gt, Box::new($rhs))};
  ($lhs:expr, >=, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Ge, Box::new($rhs))};
  ($lhs:expr, <=, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Le, Box::new($rhs))};
  ($lhs:expr, <, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Lt, Box::new($rhs))};
  ($lhs:expr, ==, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Eq, Box::new($rhs))};
  ($lhs:expr, ~=, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::TEq, Box::new($rhs))};
  ($lhs:expr, and, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::And, Box::new($rhs))};
  ($lhs:expr, or, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Or, Box::new($rhs))};
  ($lhs:expr, &, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Amper, Box::new($rhs))};
  ($lhs:expr, |, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Stick, Box::new($rhs))};
  ($lhs:expr, ~, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::Tilde, Box::new($rhs))};
  ($lhs:expr, <<, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::LShift, Box::new($rhs))};
  ($lhs:expr, >>, $rhs:expr) => {Expression::Binary(Box::new($lhs),BinaryType::RShift, Box::new($rhs))};

}

#[cfg(test)]
mod test {
    use crate::parser::expression::{fold, fold_with_priority, print};
    use crate::parser::ast::*;

    fn assert_expr<'a>(actual: &'a Expression<'a>, expected: &'a Expression<'a>) {
        assert_eq!(print(actual), print(expected));
    }

    fn assert_expr_str<'a>(actual: &'a Expression<'a>, expected: &'a str) {
        assert_eq!(print(actual), expected);
    }

    #[test]
    fn expr_test() {
        assert_expr(&expr!(), &Expression::Nil);
        assert_expr(&expr!(t), &Expression::True);
        assert_expr(&expr!(f), &Expression::False);
        assert_expr_str(&expr!(i 1), "1");
        assert_expr_str(&expr!(text "abc"), "abc");
        assert_expr_str(&expr!(...), "...");
        assert_expr_str(&expr!(! expr!(- expr!(i 1))), "!-1");
        assert_expr_str(&expr!(# expr!(i 1)), "#1");
        assert_expr_str(&expr!(- expr!(i 1)), "-1");
        assert_expr_str(&expr!(~ expr!(i 1)), "~1");
        assert_expr_str(&expr!(~ expr!(i 1)), "~1");
        assert_expr_str(&expr!(expr!(f),and, expr!(expr!(i 1), >, expr!(i 0))), "(false and (1 > 0))");
    }

    #[test]
    fn fold_test() {
        assert_expr_str(&fold_with_priority(
            expr!(f),
            vec![],
        ), "false");
        assert_expr_str(&fold_with_priority(
            expr!(f),
            vec![
                (BinaryType::And, expr!(i 1)),
                (BinaryType::Gt, expr!(i 0)),
            ],
        ), "(false and (1 > 0))");

        assert_expr_str(&fold_with_priority(
            expr!(i 1),
            vec![
                (BinaryType::Add, expr!(i 1)),
                (BinaryType::Mult, expr!(i 0)),
            ],
        ), "(1 + (1 * 0))");
        assert_expr_str(&fold_with_priority(
            expr!(i 1),
            vec![
                (BinaryType::Add, expr!(i 1)),
                (BinaryType::Mult, expr!(i 0)),
                (BinaryType::Sub, expr!(i 0)),
            ],
        ), "(1 + ((1 * 0) - 0))")
    }
}