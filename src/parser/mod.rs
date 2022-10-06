use parsit::error::ParseError;
use parsit::parser::ParseIt;
use parsit::step::Step;
use parsit::{seq, token, wrap};
use parsit::parser::EmptyToken;
use crate::parser::ast::*;
use crate::parser::tokens::Token;

mod tokens;
mod ast;
mod expression;

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
        let atom = |p: usize| { self.atom_expression(p) };
        let sign = |p: usize| {
            token!(self.t(p) =>
                    Token::Mult => BinaryType::Mult,
                    Token::Div => BinaryType::Div,
                    Token::FDiv => BinaryType::FDiv,
                    Token::Mod => BinaryType::Mod,
                    Token::Plus => BinaryType::Add,
                    Token::EllipsisIn => BinaryType::Concat,
                    Token::Gt => BinaryType::Gt,
                    Token::Lt => BinaryType::Lt,
                    Token::Ge => BinaryType::Ge,
                    Token::Le => BinaryType::Le,
                    Token::Eq => BinaryType::Eq,
                    Token::TEq => BinaryType::TEq,
                    Token::And => BinaryType::And,
                    Token::Or => BinaryType::Or,
                    Token::LShift => BinaryType::LShift,
                    Token::RShift => BinaryType::RShift,
                    Token::Ampersand => BinaryType::Amper,
                    Token::Stick => BinaryType::Stick,
                    Token::Tilde => BinaryType::Tilde,
                    Token::Minus => BinaryType::Sub,
                    Token::Caret => BinaryType::Pov
                )
        };

        atom(pos)
            .then_multi_zip(|p| sign(p).then_zip(atom))
            .map(|(first, others)| Expression::fold(first, others))
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
                    .then_skip(|p| token!(self.t(p) => Token::RBrack))
                    .then_skip(|p| token!(self.t(p) => Token::Assign))
                    .then_zip(|p| self.expr(p))
                    .map(|(k, v)| Field::Pair(FieldKey::Expr(k), v))
            };
            let pair_id_as_key = |p| {
                self.id(p)
                    .then_skip(|p| token!(self.t(p) => Token::Assign))
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
    fn attr_name_list(&'a self, pos: usize) -> Step<'a, Vec<AttrName<'a>>> {
        let attr = |p: usize| {
            let l = |p: usize| { token!(self.t(p) => Token::Lt) };
            let r = |p: usize| { token!(self.t(p) => Token::Gt) };
            let id = |p: usize| { self.id(p) };

            id(p)
                .then_or_none_zip(|p| wrap!(p => l;id;r).or_none())
                .map(|(id, opt)| {
                    if let Some(a) = opt {
                        AttrName::AttrName(id, a)
                    } else {
                        AttrName::Name(id)
                    }
                })
        };
        let comma = |p: usize| token!(self.t(p) => Token::Comma);

        seq!(pos => attr, comma)
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
                .then_skip(|p| token!(self.t(p) => Token::RParen))
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

    fn block(&'a self, pos: usize) -> Step<'a, Block<'a>> {
        let return_s = |p: usize| {
            token!(self.t(p) => Token::Return)
                .then_or_default(|p| self.expr_list(p))
                .then_or_none_zip(|p| token!(self.t(p) => Token::Semi).or_none())
                .take_left()
        };

        self.p.zero_or_more(pos, |p| self.statement(p))
            .then_or_none_zip(|p| return_s(p).or_none())
            .map(|(sts, ret)| {
                if let Some(r) = ret {
                    Block::Return(sts, r)
                } else {
                    Block::Void(sts)
                }
            })
    }

    fn statement(&'a self, pos: usize) -> Step<'a, Statement<'a>> {
        let fn_t = |p: usize| token!(self.t(p) => Token::Function);
        let end_t = |p: usize| token!(self.t(p) => Token::End);
        let block = |p: usize| self.block(p);
        let local = |p: usize| token!(self.t(p) => Token::Local);
        let id = |p: usize| self.id(p);
        let do_t = |p: usize| token!(self.t(p) => Token::Do);
        let expr = |p: usize| self.expr(p);
        let then_t = |p: usize| token!(self.t(p) => Token::Then);
        let assign = |p: usize| token!(self.t(p) => Token::Assign);

        let empty = |p: usize| token!(self.t(p) => Token::Semi => Statement::Empty);
        let assignment = |p: usize| {
            self.var_list(p)
                .then_skip(assign)
                .then_zip(|p| self.expr_list(p))
                .map(|(vs, es)| Statement::Assignment(vs, es))
        };
        let fn_call = |p: usize| self.fn_call(p).map(Statement::FnCall);
        let label = |p: usize| {
            let del = |p: usize| token!(self.t(p) => Token::DColon);
            wrap!(p => del;id;del).map(Statement::Label)
        };
        let break_s = |p: usize| token!(self.t(p) => Token::Break => Statement::Break);
        let goto = |p: usize| {
            token!(self.t(p) => Token::Goto).then(|p| self.id(p)).map(Statement::Goto)
        };

        let do_s = |p: usize| {
            wrap!(p => do_t;block;end_t).map(Statement::Do)
        };

        let while_s = |p: usize| {
            let while_t = |p: usize| token!(self.t(p) => Token::While);
            while_t(p)
                .then(expr)
                .then_zip(|p| wrap!(p => do_t;block;end_t))
                .map(|(cond, body)|
                    Statement::While(While { cond, body }))
        };

        let repeat_s = |p: usize| {
            let repeat_t = |p: usize| token!(self.t(p) => Token::Repeat);
            let until_t = |p: usize| token!(self.t(p) => Token::Until);

            repeat_t(p)
                .then(block)
                .then_skip(until_t)
                .then_zip(expr)
                .map(|(body, until)| Statement::Repeat(Repeat { until, body }))
        };

        let if_s = |p: usize| {
            let if_t = |p: usize| token!(self.t(p) => Token::If);
            let else_if_t = |p: usize| token!(self.t(p) => Token::Elseif);
            let else_t = |p: usize| token!(self.t(p) => Token::Else);

            let if_main = |p: usize| {
                wrap!(p => if_t;expr;then_t)
                    .then_zip(block)
                    .map(|(cond, body)| IfBranch { cond, body })
            };
            let else_if = |p: usize| {
                wrap!(p => else_if_t;expr;then_t)
                    .then_zip(block)
                    .map(|(cond, body)| IfBranch { cond, body })
            };
            let else_b = |p: usize| {
                else_t(p).then(block)
            };

            if_main(p)
                .then_multi_zip(else_if)
                .then_or_none_zip(|p| else_b(p).or_none())
                .then_skip(end_t)
                .map(|((main, elseifs), else_opt)| {
                    if let Some(opt) = else_opt {
                        Statement::If(If::IfElse(main, elseifs, opt))
                    } else {
                        Statement::If(If::If(main, elseifs))
                    }
                })
        };

        let for_s = |p: usize| {
            let comma = |p: usize| token!(self.t(p) => Token::Comma);
            let for_t = |p: usize| token!(self.t(p) => Token::For);
            let in_t = |p: usize| token!(self.t(p) => Token::In);
            let exprs = |p: usize| self.expr_list(p);

            let names = |p: usize| self.names(p);

            let plain = |p: usize| {
                for_t(p)
                    .then(id)
                    .then_skip(assign)
                    .then_zip(expr)
                    .then_skip(comma)
                    .then_zip(expr)
                    .then_or_none_zip(|p| comma(p).then(expr).or_none())
                    .then_skip(do_t)
                    .then_zip(block)
                    .then_skip(end_t)
                    .map(|(((init, border), step), body)|
                        Statement::For(For::Plain(PlainFor {
                            init,
                            border,
                            step,
                            body,
                        })))
            };
            let col = |p: usize| {
                for_t(p)
                    .then(names)
                    .then_skip(in_t)
                    .then_zip(exprs)
                    .then_skip(do_t)
                    .then_zip(block)
                    .then_skip(end_t)
                    .map(|((names, expressions), body)|
                        Statement::For(For::ForCol(ExprFor {
                            names,
                            expressions,
                            body,
                        })))
            };
            let res: Step<'a, Statement> = plain(p).or_from(p).or(col).into();
            res
        };

        let function = |p: usize| {
            let fn_name = |p: usize| self.fn_name(p);
            let fn_params = |p: usize| self.fn_params(p);

            fn_t(p)
                .then(fn_name)
                .then_zip(fn_params)
                .then_zip(block)
                .then_skip(end_t)
                .map(|((name, params), body)| Statement::FnDef(FnDef {
                    name,
                    params,
                    body,
                }))
        };
        let local_function = |p: usize| {
            let name = |p: usize| self.id(p);
            let fn_params = |p: usize| self.fn_params(p);

            local(p)
                .then(fn_t)
                .then(name)
                .then_zip(fn_params)
                .then_zip(block)
                .then_skip(end_t)
                .map(|((name, params), body)| Statement::LocalFnDef(FnDef {
                    name: FnName { names: vec![name], with_self: false },
                    params,
                    body,
                }))
        };
        let local_attrs = |p: usize| {
            let attr_names = |p: usize| self.attr_name_list(p);
            let assign = |p: usize| token!(self.t(p) => Token::Assign);
            let exprs = |p: usize| self.expr_list(p);

            local(p).then(attr_names)
                .then_or_default_zip(|p| assign(p).then(exprs))
                .map(|(attrs, exprs)| Statement::LocalAttrNames(attrs, exprs))
        };

        empty(pos).or_from(pos)
            .or(assignment)
            .or(fn_call)
            .or(label)
            .or(break_s)
            .or(goto)
            .or(do_s)
            .or(while_s)
            .or(repeat_s)
            .or(if_s)
            .or(for_s)
            .or(function)
            .or(local_function)
            .or(local_attrs)
            .into()
    }

    fn atom_expression(&'a self, pos: usize) -> Step<'a, Expression<'a>> {
        let primitive = |p: usize|
            token!(self.t(p) =>
                        Token::True => Expression::True,
                        Token::False => Expression::False,
                        Token::Nil => Expression::Nil,
                        Token::EllipsisOut => Expression::VarArgs)
                .or(|p| self.text(p).map(Expression::Text))
                .or(|p| self.number(p).map(Expression::Number));

        let fn_def = |p: usize|
            token!(self.t(p) => Token::Function)
                .then(|p| self.fn_params(p))
                .then_zip(|p| self.block(p))
                .then_skip(|p| token!(self.t(p) => Token::End))
                .map(|(params, body)| Expression::FnDef(params, body));

        let prefix_expr = |p: usize| {
            self.var_or_expr(p)
                .then_zip(|p| self.p.one_or_more(p, |p| self.name_args(p)))
                .map(|(head, args)| Expression::PrefixExpr(Box::new(FnCall { head, args })))
        };

        let unary = |p: usize| {
            token!(self.t(p) =>
                    Token::Not => UnaryType::Not,
                    Token::Hash => UnaryType::Hash,
                    Token::Tilde => UnaryType::Tilde,
                    Token::Minus => UnaryType::Minus)
                .then_zip(|p| self.expr(p))
                .map(|(t, e)| Expression::Unary(t, Box::new(e)))
        };

        primitive(pos)
            .or_from(pos)
            .or(fn_def)
            .or(prefix_expr)
            .or(unary)
            .or(|p| self.table_const(p).map(Expression::TableConstructor))
            .into()
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
    use crate::parser::ast::{Expression, Field, FieldKey, FnParams, Id, Number, TableConst, Text};
    use crate::parser::ast::Field::{Pair, Value};
    use crate::parser::LuaParser;
    use crate::parser::tokens::Token;

    fn p(src: &str) -> LuaParser {
        LuaParser::new(src).unwrap()
    }

    #[test]
    fn atom_expr_test() {
        expect_pos(p("true").atom_expression(0), 1);
        expect_pos(p("1").atom_expression(0), 1);
        expect_pos(p("false").atom_expression(0), 1);
        expect_pos(p("nil").atom_expression(0), 1);
        expect_pos(p("function() return  0 end").atom_expression(0), 6);
    }

    #[test]
    fn expr_test() {
        expect_pos(p("1").expr(0), 1);
        // expect_pos(p("1 * 2").expr(0), 3);
    }

    #[test]
    fn block_test() {
        expect_pos(p("; return ;").block(0), 3);
        expect_pos(p("; return 1;").block(0), 4);
        expect_pos(p("; return true, 2 ;").block(0), 6);
        expect_pos(p("goto a return 1, 0 ;").block(0), 7);
    }

    #[test]
    fn statement_test() {
        expect_pos(p(";").statement(0), 1);
        expect_pos(p("a = >=").statement(0), 3);
        expect_pos(p("a,b = >=,>=").statement(0), 7);
        expect_pos(p("a:a(>=)").statement(0), 6);
        expect_pos(p("::q::").statement(0), 3);
        expect_pos(p("break").statement(0), 1);
        expect_pos(p("goto to").statement(0), 2);
        expect_pos(p("do a = >= ; :: q :: end").statement(0), 9);
        expect_pos(p("while >= do a = >= ; :: q :: end").statement(0), 11);
        expect_pos(p("repeat a = >= ; :: q :: until >= ").statement(0), 10);
        expect_pos(p("if >= then ::q:: ; end  ").statement(0), 8);
        expect_pos(p("if >= then ::q:: ; else ; end  ").statement(0), 10);
        expect_pos(p("if >= then ::q:: ; elseif >= then ; else ; end  ").statement(0), 14);
        expect_pos(p("for x = >= , >= do ; end").statement(0), 9);
        expect_pos(p("for x = >= , >= , >= do ; end").statement(0), 11);
        expect_pos(p("for x in >= do ; end").statement(0), 7);
        expect_pos(p("for x,y in >=, >=  do ; end").statement(0), 11);
        expect_pos(p("function x.y:z(a) ; end").statement(0), 11);
        expect_pos(p("local function x(a) ; end").statement(0), 8);
        expect_pos(p("local x<y>").statement(0), 5);
        expect_pos(p("local x<y> = >=").statement(0), 7);
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
        expect_pos(p("{}").table_const(0), 2);
        expect_pos(p("{>=}").table_const(0), 3);
        expect_pos(p("{some_id = >=}").table_const(0), 5);
        expect_pos(p("{[>=] = >=}").table_const(0), 7);
        expect_pos(p("{>= ; [>=] = >= ; [>=] = >=, [>=] = >=,some_id = >= }").table_const(0), 25);
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
    fn att_name_list_test() {
        expect_pos(p("id").attr_name_list(0), 1);
        expect_pos(p("id <id>").attr_name_list(0), 4);
        expect_pos(p("id <id>,id <id>").attr_name_list(0), 9);
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
    fn fn_name_test() {
        expect_pos(p("a.b.c").fn_name(0), 5);
        expect_pos(p("a").fn_name(0), 1);
        expect_pos(p("a:b").fn_name(0), 3);
        expect_pos(p("a.b:c").fn_name(0), 5);
    }
}