use parsit::error::ParseError;
use parsit::parser::ParseIt;
use parsit::step::Step;
use parsit::{seq, token, wrap};
use parsit::parser::EmptyToken;
use crate::parser::ast::{Args, Bool, Expression, Field, FieldKey, FnCall, FnName, FnParams, Id, NameArgs, Nil, Number, Suffix, TableConst, Text, Var, VarHead, VarOrExpr, VarSuffix};
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

        let fields = |p| seq!(p => field, sep);

        let l_brace = |p: usize| token!(self.t(p) => Token::LBrace);
        let r_brace = |p: usize| token!(self.t(p) => Token::RBrace);
        let empt_vec = vec![];

        wrap!(pos => l_brace; fields or empt_vec; r_brace)
            .map(|fields| TableConst { fields })
    }

    fn names(&'a self, pos: usize) -> Step<'a, Vec<Id<'a>>> {
        let comma = |p: usize| token!(self.t(p) => Token::Comma);
        let id = |p: usize| self.id(p);
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

    fn expr_list(&'a self, pos: usize) -> Step<'a, Vec<Expression<'a>>> {
        let e = |p: usize| self.expr(p);
        let comma = |p: usize| token!(self.t(p) => Token::Comma);
        seq!(pos => e,comma)
    }
    fn id_list(&'a self, pos: usize) -> Step<'a, Vec<Id<'a>>> {
        let id = |p: usize| self.id(p);
        let comma = |p: usize| token!(self.t(p) => Token::Comma);
        seq!(pos => id,comma)
    }
    fn var_list(&'a self, pos: usize) -> Step<'a, Vec<Var<'a>>> {
        let v = |p: usize| self.var(p);
        let comma = |p: usize| token!(self.t(p) => Token::Comma);
        seq!(pos => v,comma)
    }

    fn fn_params(&'a self, pos: usize) -> Step<'a, FnParams<'a>> {
        let l = |p: usize| token!(self.t(p) => Token::LParen);
        let r = |p: usize| token!(self.t(p) => Token::RParen);
        let params = |p: usize| self.params(p);
        let def = FnParams::default();

        wrap!(pos => l;params or def;r)
    }
    fn name_args(&'a self, pos: usize) -> Step<'a, NameArgs<'a>> {
        let args = |p| {
            let expr_args = token!(self.t(p) => Token::LParen)
                .then_or_default(|p| self.expr_list(p))
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
            if let Some(v) = opt {
                NameArgs::NameArgs(v, args)
            } else {
                NameArgs::Args(args)
            }
        })
    }
    fn var_suffix(&'a self, pos: usize) -> Step<'a, VarSuffix<'a>> {
        let lb = |p: usize| token!(self.t(p) => Token::LBrack);
        let rb = |p: usize| token!(self.t(p) => Token::RBrack);
        let e = |p: usize| self.expr(p);

        let expr = |p: usize| wrap!(p => lb;e;rb).map(Suffix::Expr);
        let name = |p: usize| {
            token!(self.t(p) => Token::Dot).then(|p| self.id(p)).map(Suffix::Id)
        };


        self.p.zero_or_more(pos, |p| self.name_args(p))
            .then_zip(|p| expr(p).or_from(p).or(name).into())
            .map(|(a, r)| VarSuffix { var: a, suffix: r })
    }
    fn var(&'a self, pos: usize) -> Step<'a, Var<'a>> {
        let lp = |p: usize| token!(self.t(p) => Token::LParen);
        let rp = |p: usize| token!(self.t(p) => Token::RParen);
        let e = |p: usize| self.expr(p);
        let expr = |p: usize| {
            wrap!(p => lp;e;rp)
                .then_zip(|p| self.var_suffix(p))
                .map(|(e, s)| VarHead::Expr(e, s))
        };

        self.id(pos)
            .map(VarHead::Id)
            .or(expr)
            .then_zip(|p| self.p.zero_or_more(p, |p| self.var_suffix(p)))
            .map(|(head, tail)| Var { head, tail })
    }
    fn var_or_expr(&'a self, pos: usize) -> Step<'a, VarOrExpr<'a>> {
        let lp = |p: usize| token!(self.t(p) => Token::LParen);
        let rp = |p: usize| token!(self.t(p) => Token::RParen);
        let e = |p: usize| self.expr(p);
        let expr = |p: usize| {
            wrap!(p => lp;e;rp)
                .map(VarOrExpr::Expr)
        };

        self.var(pos)
            .map(VarOrExpr::Var)
            .or_from(pos)
            .or(expr)
            .into()
    }
    fn fn_call(&'a self, pos: usize) -> Step<'a, FnCall<'a>> {
        self.var_or_expr(pos)
            .then_zip(|p| self.p.one_or_more(p, |p| self.name_args(p)))
            .map(|(head, args)| FnCall { head, args })
    }
    fn fn_name(&'a self, pos: usize) -> Step<'a, FnName<'a>> {
        let id = |p: usize| self.id(p);
        let c = |p: usize| token!(self.t(p) => Token::Dot);
        let end = |p: usize| token!(self.t(p) => Token::Colon).then(id);

        seq!(pos => id,c)
            .then_or_none_zip(|p| end(p).or_none())
            .map(|(mut names, end)| {
                if let Some(v) = end {
                    names.push(v);
                    FnName { names, with_self: true }
                } else {
                    FnName { names, with_self: false }
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
    fn var_suffix() {
        expect_pos(p(": name (>=,>=) [>=]").var_suffix(0), 10);
        expect_pos(p(": name (>=,>=) .id").var_suffix(0), 9);
        expect_pos(p(": name (>=,>=) : name (>=,>=) [>=]").var_suffix(0), 17);
        expect_pos(p("[>=]").var_suffix(0), 3);
        expect_pos(p(".id").var_suffix(0), 2);
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
    #[test]
    fn fn_name_test() {
        expect_pos(p("a.b.c").fn_name(0), 5);
        expect_pos(p("a").fn_name(0), 1);
        expect_pos(p("a:b").fn_name(0), 3);
        expect_pos(p("a.b:c").fn_name(0), 5);
    }
}