use std::collections::HashMap;
use std::fmt::{Display, Formatter};
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
    Hex(i64),
    Binary(isize),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Text<'a> {
    pub text: &'a str,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Nil;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Bool { True, False }

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

#[derive(Debug, Clone, PartialEq)]
pub enum FnParams<'a> {
    Args(Vec<Id<'a>>),
    VarArgs,
    WithVarArgs(Vec<Id<'a>>),
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

