#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Id<'a>{
    pub v:&'a str
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Expression<'a>{
    E(&'a str)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FieldKey<'a>{
    Expr(Expression<'a>),
    Id(Id<'a>)
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Field<'a>{
    Pair(FieldKey<'a>, Expression<'a>),
    Value(Expression<'a>)
}
#[derive(Debug, Clone, PartialEq)]
pub enum FnParams<'a>{
    Args(Vec<Id<'a>>),
    VarArgs,
    WithVarArgs(Vec<Id<'a>>)
}

impl<'a> Default for FnParams<'a> {
    fn default() -> Self {
        FnParams::Args(vec![])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableConst<'a>{
    pub fields: Vec<Field<'a>>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Args<'a>{
    Expressions(Vec<Expression<'a>>),
    Constructor(TableConst<'a>),
    String(Text<'a>)
}
#[derive(Debug, Clone, PartialEq)]
pub enum NameArgs<'a>{
    Args(Args<'a>),
    NameArgs(Id<'a>,Args<'a>),
}
#[derive(Debug, Clone, PartialEq)]
pub struct  VarSuffix<'a>{
    pub var:Vec<NameArgs<'a>>,
    pub suffix:Suffix<'a>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Suffix<'a>{
    Expr( Expression<'a>),
    Id( Id<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarHead<'a>{
    Expr(Expression<'a>,VarSuffix<'a>),
    Id( Id<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct  Var<'a>{
    pub head:VarHead<'a>,
    pub tail:Vec<VarSuffix<'a>>
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarOrExpr<'a>{
    Expr(Expression<'a>),
    Var(Var<'a>),
}
#[derive(Debug, Clone, PartialEq)]
pub struct FnCall<'a>{
    pub head:VarOrExpr<'a>,
    pub args:Vec<NameArgs<'a>>
}

pub struct FnName<'a>{
    pub names: Vec<Id<'a>>,
    pub with_self:bool
}