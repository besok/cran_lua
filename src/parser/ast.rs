use std::collections::HashMap;
use std::fmt::{Display, Formatter, write};
use std::iter::Map;
use BinaryType::*;
use crate::parser::expression::fold_with_priority;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Id<'a> {
    pub v: &'a str,
}

impl<'a> Id<'a> {
    pub fn new(v: &'a str) -> Self {
        Self { v }
    }
}

impl<'a> Display for Id<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.v)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
    Hex(i64),
    Binary(isize),
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Int(v) => write!(f, "{}", v),
            Number::Float(v) => write!(f, "{}", v),
            Number::Hex(v) => write!(f, "0x{}", v),
            Number::Binary(v) => write!(f, "b{}", v),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Text<'a> {
    pub text: &'a str,
}

impl<'a> Display for Text<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.text)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Nil;

impl Display for Nil {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "nil")
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Bool { True, False }

impl Display for Bool {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression<'a> {
    Nil,
    False,
    True,
    Number(Number),
    Text(Text<'a>),
    VarArgs,
    FnDef(FnParams<'a>, Block<'a>),
    PrefixExpr(Box<FnCall<'a>>),
    TableConstructor(TableConst<'a>),
    Unary(UnaryType, Box<Expression<'a>>),
    Binary(Box<Expression<'a>>, BinaryType, Box<Expression<'a>>),

}

impl<'a> Display for Expression<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
       write!(f,"!")
    }
}

impl<'a> Expression<'a> {
    pub fn fold(first: Expression<'a>, elems: Vec<(BinaryType, Expression<'a>)>) -> Expression<'a> {
        fold_with_priority(first, elems)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UnaryType {
    Not,
    Hash,
    Minus,
    Tilde,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BinaryType {
    Mult,
    Div,
    Mod,
    FDiv,
    Add,
    Sub,
    Pov,
    Concat,
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    TEq,
    And,
    Amper,
    Stick,
    Tilde,
    LShift,
    RShift,
    Or,
}


#[derive(Debug, Clone, PartialEq)]
pub enum FieldKey<'a> {
    Expr(Expression<'a>),
    Id(Id<'a>),
}


#[derive(Debug, Clone, PartialEq)]
pub enum Field<'a> {
    Pair(FieldKey<'a>, Expression<'a>),
    Value(Expression<'a>),
}

impl<'a> Display for Field<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Field::Pair(FieldKey::Id(id), e) => write!(f, "{} = {}", id, e),
            Field::Pair(FieldKey::Expr(e), ev) => write!(f, "[{}] = {}", e, ev),
            Field::Value(e) => write!(f, "{}", e)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FnParams<'a> {
    Args(Vec<Id<'a>>),
    VarArgs,
    WithVarArgs(Vec<Id<'a>>),
}

impl<'a> Display for FnParams<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FnParams::Args(args) => {
                let args: Vec<String> = args.iter().map(|id| format!("{}", id)).collect();
                write!(f, "({},...)", args.join(","))
            }
            FnParams::VarArgs => write!(f, "(...)"),
            FnParams::WithVarArgs(args) => {
                let args: Vec<String> = args.iter().map(|id| format!("{}", id)).collect();
                write!(f, "({},...)", args.join(","))
            }
        }
    }
}

impl<'a> Default for FnParams<'a> {
    fn default() -> Self {
        FnParams::Args(vec![])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableConst<'a> {
    pub fields: Vec<Field<'a>>,
}

impl<'a> Display for TableConst<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let args: Vec<String> = self.fields.iter().map(|id| format!("{}", id)).collect();
        write!(f, "{{{}}}", args.join(","))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Args<'a> {
    Expressions(Vec<Expression<'a>>),
    Constructor(TableConst<'a>),
    String(Text<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NameArgs<'a> {
    Args(Args<'a>),
    NameArgs(Id<'a>, Args<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarSuffix<'a> {
    pub var: Vec<NameArgs<'a>>,
    pub suffix: Suffix<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Suffix<'a> {
    Expr(Expression<'a>),
    Id(Id<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarHead<'a> {
    Expr(Expression<'a>, VarSuffix<'a>),
    Id(Id<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Var<'a> {
    pub head: VarHead<'a>,
    pub tail: Vec<VarSuffix<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarOrExpr<'a> {
    Expr(Expression<'a>),
    Var(Var<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnCall<'a> {
    pub head: VarOrExpr<'a>,
    pub args: Vec<NameArgs<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnName<'a> {
    pub names: Vec<Id<'a>>,
    pub with_self: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrName<'a> {
    Name(Id<'a>),
    AttrName(Id<'a>, Id<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block<'a> {
    Void(Vec<Statement<'a>>),
    Return(Vec<Statement<'a>>, Vec<Expression<'a>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct While<'a> {
    pub cond: Expression<'a>,
    pub body: Block<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Repeat<'a> {
    pub until: Expression<'a>,
    pub body: Block<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfBranch<'a> {
    pub cond: Expression<'a>,
    pub body: Block<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlainFor<'a> {
    pub init: (Id<'a>, Expression<'a>),
    pub border: Expression<'a>,
    pub step: Option<Expression<'a>>,
    pub body: Block<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprFor<'a> {
    pub names: Vec<Id<'a>>,
    pub expressions: Vec<Expression<'a>>,
    pub body: Block<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum If<'a> {
    If(IfBranch<'a>, Vec<IfBranch<'a>>),
    IfElse(IfBranch<'a>, Vec<IfBranch<'a>>, Block<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum For<'a> {
    Plain(PlainFor<'a>),
    ForCol(ExprFor<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDef<'a> {
    pub name: FnName<'a>,
    pub params: FnParams<'a>,
    pub body: Block<'a>,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Statement<'a> {
    Empty,
    Assignment(Vec<Var<'a>>, Vec<Expression<'a>>),
    FnCall(FnCall<'a>),
    Label(Id<'a>),
    Break,
    Goto(Id<'a>),
    Do(Block<'a>),
    While(While<'a>),
    Repeat(Repeat<'a>),
    If(If<'a>),
    For(For<'a>),
    FnDef(FnDef<'a>),
    LocalFnDef(FnDef<'a>),
    LocalAttrNames(Vec<AttrName<'a>>, Vec<Expression<'a>>),
}

#[cfg(test)]
mod tests {
    use std::fmt::Display;
    use crate::parser::ast::{Expression, Field, FieldKey, FnParams, Id, TableConst, Text};

    fn display<T: Display>(v: &T, expect: &str) {
        assert_eq!(format!("{}", v), expect)
    }

    #[test]
    fn fn_param_display_test() {
        display(
            &FnParams::WithVarArgs(vec![Id { v: "a" }, Id { v: "b" }]),
            "(a,b,...)",
        )
    }
    #[test]
    fn table_constr_display_test() {
        display(
            &TableConst{ fields: vec![
                Field::Value(Expression::Nil),
                Field::Pair(FieldKey::Id(Id{ v: "a" }),Expression::Text(Text{ text: "t" })),
                Field::Pair(FieldKey::Expr(Expression::True),Expression::Text(Text{ text: "t" })),
            ] },
            "{!,a = !,[!] = !}",
        )
    }
}