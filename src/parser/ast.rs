use std::collections::HashMap;
use std::fmt::{Display, Formatter, write};
use std::iter::Map;
use BinaryType::*;
use crate::parser::expression::fold_with_priority;

trait Show {
    type Output;
    fn show(&self) -> Self::Output;
}


impl<T: Display> Show for Vec<T> {
    type Output = Vec<String>;

    fn show(&self) -> Self::Output {
        self.iter().map(|el| el.to_string()).collect()
    }
}

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
        match self {
            Expression::Nil => write!(f, "nil"),
            Expression::False => write!(f, "false"),
            Expression::True => write!(f, "true"),
            Expression::Number(n) => write!(f, "{}", n),
            Expression::Text(t) => write!(f, "{}", t),
            Expression::VarArgs => write!(f, "..."),
            Expression::FnDef(params, body) => {
                writeln!(f, "function {}", params)?;
                writeln!(f, "{}", body)?;
                write!(f, "end")
            }
            Expression::PrefixExpr(e) => write!(f, "{}", e),
            Expression::TableConstructor(constr) => write!(f, "{}", constr),
            Expression::Unary(s, e) => write!(f, "{}{}", s, e),
            Expression::Binary(lhs, s, rhs) => write!(f, "({} {} {})", lhs, s, rhs),
        }
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
                write!(f, "({},...)", args.show().join(","))
            }
            FnParams::VarArgs => write!(f, "(...)"),
            FnParams::WithVarArgs(args) => {
                write!(f, "({},...)", args.show().join(","))
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
        write!(f, "{{{}}}", self.fields.show().join(","))
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

impl<'a> Display for NameArgs<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut match_args = |args: &Args, prefix: &str| -> std::fmt::Result {
            match args {
                Args::Expressions(exprs) => {
                    write!(f, "{} (", prefix)?;
                    write!(f, "{}", exprs.show().join(","))?;
                    write!(f, ")")
                }
                Args::Constructor(constr) => write!(f, "{} {}", prefix, constr),
                Args::String(t) => write!(f, "{} {}", prefix, t.text),
            }
        };

        match self {
            NameArgs::Args(args) => match_args(args, ""),
            NameArgs::NameArgs(name, args) => {
                match_args(args, format!(":{}", name).as_str())
            }
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct VarSuffix<'a> {
    pub var: Vec<NameArgs<'a>>,
    pub suffix: Suffix<'a>,
}

impl<'a> Display for VarSuffix<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.var.show().join(" "))?;
        match &self.suffix {
            Suffix::Expr(e) => write!(f, "[{}]", e),
            Suffix::Id(id) => write!(f, ".{}", id)
        }
    }
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

impl<'a> Display for Var<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.head {
            VarHead::Expr(e, vs) => write!(f, "({}){}", e, vs)?,
            VarHead::Id(id) => write!(f, "{}", id)?,
        }
        write!(f, "{}", self.tail.show().join(" "))
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum VarOrExpr<'a> {
    Expr(Expression<'a>),
    Var(Var<'a>),
}

impl<'a> Display for VarOrExpr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            VarOrExpr::Expr(e) => write!(f, "({})", e),
            VarOrExpr::Var(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnCall<'a> {
    pub head: VarOrExpr<'a>,
    pub args: Vec<NameArgs<'a>>,
}

impl<'a> Display for FnCall<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.head)?;
        write!(f, "{}", self.args.show().join(""))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnName<'a> {
    pub names: Vec<Id<'a>>,
    pub last: Option<Id<'a>>,
}

impl<'a> Display for FnName<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let args: Vec<String> = self.names.show();

        match self.last {
            None => write!(f, "{}", args.join(".")),
            Some(last) => write!(f, "{}:{}", args.join("."), last)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrName<'a> {
    Name(Id<'a>),
    AttrName(Id<'a>, Id<'a>),
}

impl<'a> Display for AttrName<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AttrName::Name(name) => write!(f, "{}", name),
            AttrName::AttrName(name, attr) => write!(f, "{}<{}>", name, attr)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block<'a> {
    Void(Vec<Statement<'a>>),
    Return(Vec<Statement<'a>>, Vec<Expression<'a>>),
}

impl<'a> Display for Block<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Block::Void(sts) => {
                for s in sts.iter() {
                    writeln!(f, "{}", s)?;
                }
                write!(f, ";")
            }
            Block::Return(sts, exprs) => {
                for s in sts.iter() {
                    writeln!(f, "{}", s)?;
                }
                write!(f, "return {}", exprs.show().join(","))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct While<'a> {
    pub cond: Expression<'a>,
    pub body: Block<'a>,
}

impl<'a> Display for While<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "while {} do", self.cond)?;
        writeln!(f, "{}", self.body)?;
        write!(f, "end")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Repeat<'a> {
    pub until: Expression<'a>,
    pub body: Block<'a>,
}

impl<'a> Display for Repeat<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "repeat {} do", self.body)?;
        write!(f, "until {}", self.until)
    }
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

impl<'a> Display for PlainFor<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let step = self.step.as_ref().map(|e| e.to_string()).unwrap_or_default();
        writeln!(f, "for {} = {}, {}, {} do", self.init.0, self.init.1, self.border, step)?;
        writeln!(f, "{}", self.body)?;
        write!(f, "end")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprFor<'a> {
    pub names: Vec<Id<'a>>,
    pub expressions: Vec<Expression<'a>>,
    pub body: Block<'a>,
}

impl<'a> Display for ExprFor<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "for {} in {}  do",
                 self.names.show().join(","),
                 self.expressions.show().join(","))?;
        writeln!(f, "{}", self.body)?;
        write!(f, "end")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum If<'a> {
    If(IfBranch<'a>, Vec<IfBranch<'a>>),
    IfElse(IfBranch<'a>, Vec<IfBranch<'a>>, Block<'a>),
}

impl<'a> Display for If<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            If::If(if_main, ifelses) => {
                writeln!(f, "if {} then ", if_main.cond)?;
                writeln!(f, "{} ", if_main.body)?;
                for other in ifelses.iter() {
                    writeln!(f, "elseif {} then ", other.cond)?;
                    writeln!(f, "{} ", other.body)?;
                }
            }
            If::IfElse(if_main, ifelses, else_block) => {
                writeln!(f, "if {} then ", if_main.cond)?;
                writeln!(f, "{} ", if_main.body)?;
                for other in ifelses.iter() {
                    writeln!(f, "elseif {} then ", other.cond)?;
                    writeln!(f, "{} ", other.body)?;
                }
                writeln!(f, "else ")?;
                writeln!(f, "{} ", else_block)?;
            }
        }
        write!(f, "end")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum For<'a> {
    Plain(PlainFor<'a>),
    ForCol(ExprFor<'a>),
}

impl<'a> Display for For<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            For::Plain(plain) => write!(f, "{}", plain),
            For::ForCol(expr_for) => write!(f, "{}", expr_for),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDef<'a> {
    pub name: FnName<'a>,
    pub params: FnParams<'a>,
    pub body: Block<'a>,
}

impl<'a> Display for FnDef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "function {}{}", self.name, self.params)?;
        writeln!(f, "{}", self.body)?;
        write!(f, "end")
    }
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

impl<'a> Display for Statement<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Empty => write!(f, ";"),
            Statement::Assignment(lhs, rhs) => {
                write!(f, "{} = {} ", lhs.show().join(","), rhs.show().join(","))
            }
            Statement::FnCall(fn_call) => write!(f, "{}", fn_call),
            Statement::Label(id) => write!(f, "::{}::", id),
            Statement::Break => write!(f, "break"),
            Statement::Goto(id) => write!(f, "goto {}", id),
            Statement::Do(body) => {
                writeln!(f, "do ")?;
                writeln!(f, "{}", body)?;
                write!(f, "end")
            }
            Statement::While(body) => write!(f, "{}", body),
            Statement::Repeat(repeat) => write!(f, "{}", repeat),
            Statement::If(body) => write!(f, "{}", body),
            Statement::For(body) => write!(f, "{}", body),
            Statement::FnDef(body) => write!(f, "{}", body),
            Statement::LocalFnDef(body) => write!(f, "local {}", body),
            Statement::LocalAttrNames(names, exprs) => {
                let names: Vec<String> = names.show();
                let exprs: Vec<String> = exprs.show();
                let exprs = if exprs.is_empty() { String::new() } else { format!("= {}", exprs.join(",")) };
                write!(f, "local {}{}", names.join(","), exprs)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Display;
    use crate::parser::ast::{Args, Expression, Field, FieldKey, FnParams, Id, NameArgs, TableConst, Text};

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
            &TableConst {
                fields: vec![
                    Field::Value(Expression::Nil),
                    Field::Pair(FieldKey::Id(Id { v: "a" }), Expression::Text(Text { text: "t" })),
                    Field::Pair(FieldKey::Expr(Expression::True), Expression::Text(Text { text: "t" })),
                ]
            },
            "{nil,a = \"t\",[true] = \"t\"}",
        )
    }

    #[test]
    fn name_args_display_test() {
        display(
            &NameArgs::Args(
                Args::Constructor(TableConst {
                    fields: vec![
                        Field::Value(Expression::Nil),
                        Field::Pair(FieldKey::Id(Id { v: "a" }), Expression::Text(Text { text: "t" })),
                        Field::Pair(FieldKey::Expr(Expression::True), Expression::Text(Text { text: "t" })),
                    ]
                })
            ),
            " {!,a = !,[!] = !}",
        );
        display(
            &NameArgs::NameArgs(Id { v: "name" },
                                Args::Constructor(TableConst {
                                    fields: vec![
                                        Field::Value(Expression::Nil),
                                        Field::Pair(FieldKey::Id(Id { v: "a" }), Expression::Text(Text { text: "t" })),
                                        Field::Pair(FieldKey::Expr(Expression::True), Expression::Text(Text { text: "t" })),
                                    ]
                                }),
            ),
            ":name {!,a = !,[!] = !}",
        )
    }

}