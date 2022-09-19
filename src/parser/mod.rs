use parsit::error::ParseError;
use parsit::parser::ParseIt;
use parsit::step::Step;
use parsit::token;
use parsit::parser::EmptyToken;
use crate::parser::ast::{Bool, Expression, Field, FieldKey, Id, Nil, Number, TableConst, Text};
use crate::parser::tokens::Token;

mod tokens;
mod ast;

struct LuaParser<'a> {
    p: ParseIt<'a, Token<'a>>,
}

impl<'a> LuaParser<'a> {
    fn id(&'a self, pos: usize) -> Step<'a, Id<'a>> {
        token!(self.t(pos) => Token::Id(v) => Id{v} )
    }
    fn text(&'a self, pos: usize) -> Step<'a, Text<'a>> {
        token!(self.t(pos) => Token::StringLit(v) => Text{text: v} )
    }
    fn nil(&'a self, pos: usize) -> Step<'a, Nil> {
        token!(self.t(pos) => Token::Nil => Nil )
    }
    fn bool(&'a self, pos: usize) -> Step<'a, Bool> {
        token!(self.t(pos) =>
                Token::True => Bool::True,
                Token::False => Bool::False
        )
    }
    fn number(&'a self, pos: usize) -> Step<'a, Number> {
        token!(self.t(pos) =>Token::Digit(n) => *n)
    }
    fn expr(&'a self, pos: usize) -> Step<'a, Expression<'a>> {
        token!(self.t(pos) =>Token::Ge => Expression::E(""))
    }

    fn table_const(&'a self, pos: usize) -> Step<'a, TableConst<'a>> {
        let sep = |p| {
            token!(self.t(p) => Token::Comma)
                .or(|p| token!(self.t(p) => Token::Semi))
        };

        let field = |p| {
            let pair_expr_as_key = |p| {
                token!(self.t(p) => Token::LBrack)
                    .then(|p| self.expr(p))
                    .then_zip(|p| token!(self.t(p) => Token::RBrack))
                    .take_left()
                    .then_zip(|p| token!(self.t(p) => Token::Assign))
                    .take_left()
                    .then_zip(|p| self.expr(p))
                    .map(|(k, v)| Field::Pair(FieldKey::Expr(k), v))
            };
            let pair_id_as_key = |p| {
                self.id(p)
                    .then_zip(|p| token!(self.t(p) => Token::Assign))
                    .take_left()
                    .then_zip(|p| self.expr(p))
                    .map(|(id, v)| Field::Pair(FieldKey::Id(id), v))
            };

            let value = |p| {
                self.expr(p).map(Field::Value)
            };

            let step: Step<'a, Field<'a>> = pair_expr_as_key(p).or_from(p)
                .or(pair_id_as_key)
                .or(value)
                .into();
            step
        };

        let fields = |p| {
            field(p)
                .then_multi_zip(|p| sep(p).then(field))
                .then_or_none_zip(|p| sep(p).or_none())
                .take_left()
                .merge()
        };

        token!(self.t(pos) => Token::LBrace)
            .then_or_val(fields, vec![])
            .then_zip(|p| token!(self.t(p) => Token::RBrace))
            .take_left()
            .map(|fields| TableConst { fields })
    }
}

impl<'a> LuaParser<'a> {
    pub fn new(src: &'a str) -> Result<Self, ParseError> {
        Ok(LuaParser {
            p: ParseIt::new(src)?,
        })
    }
    fn t(&'a self, pos: usize) -> Result<(&'a Token<'a>, usize), ParseError<'a>> {
        self.p.token(pos)
    }
}

#[cfg(test)]
mod tests {
    use parsit::step::Step;
    use parsit::test::parser_test::*;
    use crate::parser::ast::{Expression, Field, FieldKey, Id, TableConst, Text};
    use crate::parser::ast::Field::{Pair, Value};
    use crate::parser::LuaParser;
    use crate::parser::tokens::Token;

    fn p(src: &str) -> LuaParser {
        LuaParser::new(src).unwrap()
    }

    #[test]
    fn text_test() {
        expect(
            p("\"text\"").text(0),
            Text { text: "text" },
        );
        expect(
            p("\'text\'").text(0),
            Text { text: "text" },
        );
        expect(
            p(r#"[[
            sometext
            ]]"#).text(0),
            Text { text: "\n            sometext\n            " },
        );
        expect(
            p(r#"[=[
            sometext
            ]=]"#).text(0),
            Text { text: "\n            sometext\n            " },
        )
    }

    #[test]
    fn table_constructor_test() {
        expect(p("{}").table_const(0), TableConst { fields: vec![] });
        expect(p("{>=}").table_const(0), TableConst { fields: vec![Value(Expression::E(""))] });
        expect(p("{some_id = >=}").table_const(0),
               TableConst {
                   fields: vec![
                       Pair(FieldKey::Id(Id { v: "some_id" }), Expression::E(""))
                   ]
               });
        expect(p("{[>=] = >=}").table_const(0),
               TableConst {
                   fields: vec![
                       Pair(FieldKey::Expr(Expression::E("")), Expression::E(""))
                   ]
               });
        expect(p("{>= ; [>=] = >= ; [>=] = >=, [>=] = >=,some_id = >= }").table_const(0),
               TableConst {
                   fields: vec![
                       Value(Expression::E("")),
                       Pair(FieldKey::Expr(Expression::E("")), Expression::E("")),
                       Pair(FieldKey::Expr(Expression::E("")), Expression::E("")),
                       Pair(FieldKey::Expr(Expression::E("")), Expression::E("")),
                       Pair(FieldKey::Id(Id { v: "some_id" }), Expression::E(""))
                   ]
               });
    }

    #[test]
    fn expr_test() {
        expect(p(">=").expr(0), Expression::E(""))
    }
}