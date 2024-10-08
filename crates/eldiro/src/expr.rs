mod binding_usage;
mod block;
mod func_call;

use crate::env::Env;
use crate::utils;
use crate::val::Val;
pub(crate) use binding_usage::BindingUsage;
pub(crate) use block::Block;
pub(crate) use func_call::FuncCall;
#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct Number(pub(crate) i32);

impl Number {
    fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, number) = utils::extract_digits(s)?;
        Ok((s, Self(number.parse().unwrap())))
    }
}

#[derive(PartialEq, Debug, Eq, Clone)]
pub(crate) enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    fn new(s: &str) -> Result<(&str, Self), String> {
        utils::tag("+", s)
            .map(|s| (s, Self::Add))
            .or_else(|_| utils::tag("-", s).map(|s| (s, Self::Sub)))
            .or_else(|_| utils::tag("*", s).map(|s| (s, Self::Mul)))
            .or_else(|_| utils::tag("/", s).map(|s| (s, Self::Div)))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Expr {
    Number(Number),
    Operation {
        lhs: Box<Self>,
        rhs: Box<Self>,
        op: Op,
    },
    BindingUsage(BindingUsage),
    Block(Block),
    FuncCall(FuncCall),
}

impl Expr {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), String> {
        Self::new_operation(s).or_else(|_| Self::new_non_operation(s))
    }

    fn new_non_operation(s: &str) -> Result<(&str, Self), String> {
        Self::new_number(s)
            .or_else(|_| FuncCall::new(s).map(|(s, func_call)| (s, Self::FuncCall(func_call))))
            .or_else(|_| {
                BindingUsage::new(s)
                    .map(|(s, binding_usage)| (s, Self::BindingUsage(binding_usage)))
            })
            .or_else(|_| Block::new(s).map(|(s, block)| (s, Self::Block(block))))
    }

    fn new_operation(s: &str) -> Result<(&str, Self), String> {
        let (s, lhs) = Self::new_non_operation(s)?;
        let (s, _) = utils::extract_whitespace(s);

        let (s, op) = Op::new(s)?;
        let (s, _) = utils::extract_whitespace(s);

        let (s, rhs) = Self::new_non_operation(s)?;

        Ok((
            s,
            Self::Operation {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                op,
            },
        ))
    }
    fn new_number(s: &str) -> Result<(&str, Self), String> {
        Number::new(s).map(|(s, number)| (s, Self::Number(number)))
    }

    pub(crate) fn eval(&self, env: &Env) -> Result<Val, String> {
        match self {
            Self::Number(Number(n)) => Ok(Val::Number(*n)),
            Self::Operation { lhs, rhs, op } => {
                let lhs = lhs.eval(env)?;
                let rhs = rhs.eval(env)?;

                let (lhs, rhs) = match (lhs,rhs) {
                    (Val::Number(lhs), Val::Number(rhs)) => (lhs, rhs),
                    _ => return Err("cannot evaluate operation whose left-hand side and right-hand side are not both numbers".to_string()),
                };

                let result = match op {
                    Op::Add => lhs + rhs,
                    Op::Sub => lhs - rhs,
                    Op::Mul => lhs * rhs,
                    Op::Div => lhs / rhs,
                };

                Ok(Val::Number(result))
            }
            Self::BindingUsage(binding_usage) => binding_usage.eval(env),
            Self::Block(block) => block.eval(env),
            Self::FuncCall(func_call) => func_call.eval(env),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::env::Env;
    use crate::func_def::FuncDef;
    use crate::stmt::Stmt;
    use crate::val::Val;

    #[test]
    fn eval_func_call() {
        let mut env = Env::default();

        env.store_func(
            "add".to_string(),
            vec!["x".to_string(), "y".to_string()],
            Stmt::Expr(Expr::Operation {
                lhs: Box::new(Expr::BindingUsage(BindingUsage {
                    name: "x".to_string(),
                })),
                rhs: Box::new(Expr::BindingUsage(BindingUsage {
                    name: "y".to_string(),
                })),
                op: Op::Add,
            }),
        );

        assert_eq!(
            Expr::FuncCall(FuncCall {
                callee: "add".to_string(),
                params: vec![Expr::Number(Number(2)), Expr::Number(Number(2))],
            })
            .eval(&env),
            Ok(Val::Number(4)),
        );
    }

    #[test]
    fn eval_func_def() {
        assert_eq!(
            Stmt::FuncDef(FuncDef {
                name: "always_return_one".to_string(),
                params: Vec::new(),
                body: Box::new(Stmt::Expr(Expr::Number(Number(1)))),
            })
            .eval(&mut Env::default()),
            Ok(Val::Unit),
        );
    }

    #[test]
    fn eval_non_number_operation() {
        assert_eq!(
            Expr::Operation {
                lhs: Box::new(Expr::Number(Number(10))),
                rhs: Box::new(Expr::Block(Block { stmts: Vec::new() })),
                op: Op::Add,
            }
            .eval(&Env::default()),
            Err("cannot evaluate operation whose left-hand side and right-hand side are not both numbers".to_string()),
        );
    }
    #[test]
    fn parse_one_plus_two() {
        assert_eq!(
            Expr::new("1+2"),
            Ok((
                "",
                Expr::Operation {
                    lhs: Box::new(Expr::Number(Number(1))),
                    rhs: Box::new(Expr::Number(Number(2))),
                    op: Op::Add,
                },
            )),
        );
    }

    #[test]
    fn parse_expr_with_whitespace() {
        assert_eq!(
            Expr::new("2 * 2"),
            Ok((
                "",
                Expr::Operation {
                    lhs: Box::new(Expr::Number(Number(2))),
                    rhs: Box::new(Expr::Number(Number(2))),
                    op: Op::Mul,
                },
            )),
        );
    }

    // snip

    #[test]
    fn eval_add() {
        assert_eq!(
            Expr::Operation {
                lhs: Box::new(Expr::Number(Number(10))),
                rhs: Box::new(Expr::Number(Number(10))),
                op: Op::Add,
            }
            .eval(&Env::default()),
            Ok(Val::Number(20)),
        );
    }

    #[test]
    fn eval_sub() {
        assert_eq!(
            Expr::Operation {
                lhs: Box::new(Expr::Number(Number(1))),
                rhs: Box::new(Expr::Number(Number(5))),
                op: Op::Sub,
            }
            .eval(&Env::default()),
            Ok(Val::Number(-4)),
        );
    }

    #[test]
    fn eval_mul() {
        assert_eq!(
            Expr::Operation {
                lhs: Box::new(Expr::Number(Number(5))),
                rhs: Box::new(Expr::Number(Number(6))),
                op: Op::Mul,
            }
            .eval(&Env::default()),
            Ok(Val::Number(30)),
        );
    }

    #[test]
    fn eval_div() {
        assert_eq!(
            Expr::Operation {
                lhs: Box::new(Expr::Number(Number(200))),
                rhs: Box::new(Expr::Number(Number(20))),
                op: Op::Div,
            }
            .eval(&Env::default()),
            Ok(Val::Number(10)),
        );
    }

    // snip
}
