use parsit::error::ParseError;
use parsit::parser::ParseIt;
use parsit::step::Step;
use parsit::{seq, token, wrap};
use parsit::parser::EmptyToken;
use crate::parser::ast::{Args, Bool, Expression, Field, FieldKey, FnParams, Id, NameArgs, Nil, Number, TableConst, Text};
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

        let fields = |p| seq!(p => field, sep,);

        let l_brace = |p: usize| token!(self.t(p) => Token::LBrace);
        let r_brace = |p: usize| token!(self.t(p) => Token::RBrace);
        let empt_vec = vec![];

        wrap!(pos => l_brace; fields or empt_vec; r_brace)
            .map(|fields| TableConst { fields })
    }

    fn names(&'a self, pos: usize) -> Step<'a, Vec<Id<'a>>> {
        let comma = |p:usize|   token!(self.t(p) => Token::Comma);
        let id = |p:usize|   self.id(p);
        seq!(pos => id, comma)
    }

    fn params(&'a self, pos: usize) -> Step<'a, FnParams<'a>> {
        let varags = |p: usize|
            token!(self.t(p) => Token::Comma)
                .then(|p| token!(self.t(p) => Token::EllipsisOut))
                .or_none();

        let transform = |(names, vargs): (Vec<Id<'a>>, Option<EmptyToken>)| {
            if vargs.is_some() {
                FnParams::WithVarArgs(names)
            } else { FnParams::Args(names) }
        };


        self.names(pos)
            .then_or_none_zip(varags)
            .map(transform)
            .or_from(pos)
            .or(|p| token!(self.t(p) => Token::EllipsisOut).map(|_| FnParams::VarArgs))
            .into()
    }
    fn fn_params(&'a self, pos: usize) -> Step<'a, FnParams<'a>> {
        let l = |p:usize|    token!(self.t(p) => Token::LParen);
        let r = |p:usize|    token!(self.t(p) => Token::RParen);
        let params = |p:usize| self.params(p);
        let def = FnParams::default();

        wrap!(pos => l;params or def;r)
    }
    fn name_args(&'a self, pos: usize) -> Step<'a, NameArgs<'a>> {
        let args = |p| {
            let exprs = |p| {
                self.expr(p)
                    .then_multi_zip(|p|
                        token!(self.t(p) => Token::Comma)
                            .then(|p| self.expr(p)))
                    .merge()
            };


            let expr_args = token!(self.t(p) => Token::LParen)
                .then_or_val(exprs, vec![])
                .then_zip(|p| token!(self.t(p) => Token::RParen))
                .take_left()
                .map(Args::Expressions);


            let step: Step<'a, Args> = expr_args
                .or_from(p)
                .or(|p| self.table_const(p).map(Args::Constructor))
                .or(|p| self.text(p).map(Args::String))
                .into();
            step
        };

        let name = token!(self.t(pos) => Token::Colon).then(|p| self.id(p));
        name.or_none().then_zip(args).map(|(opt, args)| {
            if opt.is_some() {
                NameArgs::NameArgs(opt.unwrap(), args)
            } else {
                NameArgs::Args(args)
            }
        })
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
    use crate::parser::ast::{Expression, Field, FieldKey, FnParams, Id, TableConst, Text};
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
                       Pair(FieldKey::Id(Id { v: "some_id" }), Expression::E("")),
                   ]
               });
    }

    #[test]
    fn params() {
        expect(p("...").params(0), FnParams::VarArgs);
        expect(p("a").params(0), FnParams::Args(vec![Id::new("a")]));
        expect(p("a,b").params(0), FnParams::Args(vec![Id::new("a"), Id::new("b")]));
        expect(p("a,b, ... ").params(0), FnParams::WithVarArgs(vec![Id::new("a"), Id::new("b")]));
    }

    #[test]
    fn fn_params() {
        expect(p("()").fn_params(0), FnParams::Args(vec![]));
        expect(p("(...)").fn_params(0), FnParams::VarArgs);
        expect(p("(a)").fn_params(0), FnParams::Args(vec![Id::new("a")]));
        expect(p("(a,b)").fn_params(0), FnParams::Args(vec![Id::new("a"), Id::new("b")]));
        expect(p("(a,b, ... )").fn_params(0), FnParams::WithVarArgs(vec![Id::new("a"), Id::new("b")]));
    }

    #[test]
    fn name_args() {
        expect_pos(p(": name \"a\"").name_args(0), 3);
        expect_pos(p("\"a\"").name_args(0), 1);
        expect_pos(p(": name (>=,>=)").name_args(0), 7);
        expect_pos(p(" (>=,>=)").name_args(0), 5);
        expect_pos(p(": name (>=,>=)").name_args(0), 7);
        expect_pos(p(" (>=,>=)").name_args(0), 5);
        expect_pos(p(": name {[>=] = >=}").name_args(0), 9);
        expect_pos(p("{[>=] = >=}").name_args(0), 7);
    }

    #[test]
    fn expr_test() {
        expect(p(">=").expr(0), Expression::E(""))
    }
}