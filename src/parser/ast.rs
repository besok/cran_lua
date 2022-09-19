#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Id<'a>{
    pub v:&'a str
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
pub struct TableConst<'a>{
    pub fields: Vec<Field<'a>>
}