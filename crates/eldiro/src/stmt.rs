use std::clone;

use crate::{binding_def::BindingDef, env::Env, expr::Expr, func_def::FuncDef, val::Val};

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Stmt {
    BindingDef(BindingDef),
    Expr(Expr),
    FuncDef(FuncDef),
}

impl Stmt {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), String> {
        BindingDef::new(s)
            .map(|(s, binding_def)| (s, Self::BindingDef(binding_def)))
            .or_else(|_| FuncDef::new(s).map(|(s, func_def)| (s, Self::FuncDef(func_def))))
            .or_else(|_| Expr::new(s).map(|(s, expr)| (s, Self::Expr(expr))))
    }

    pub(crate) fn eval(&self, env: &mut Env) -> Result<Val, String> {
        match self {
            Self::BindingDef(binding_def) => {
                binding_def.eval(env)?;
                Ok(Val::Unit)
            }
            Self::FuncDef(func_def) => {
                func_def.eval(env)?;
                Ok(Val::Unit)
            }
            Self::Expr(expr) => expr.eval(env),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Env, Expr, FuncDef, Stmt, Val};
    use crate::expr::BindingUsage;
    use crate::expr::{Number, Op};
    use crate::stmt::BindingDef;
    #[test]
    fn parse_expr() {
        assert_eq!(
            Stmt::new("1+1"),
            Ok((
                "",
                Stmt::Expr(Expr::Operation {
                    lhs: Box::new(Expr::Number(Number(1))),
                    rhs: Box::new(Expr::Number(Number(1))),
                    op: Op::Add,
                }),
            )),
        );
    }
    #[test]
    fn parse_func_def() {
        fn parse_func_def() {
            assert_eq!(
                Stmt::new("fn identity x => x"),
                Ok((
                    "",
                    Stmt::FuncDef(FuncDef {
                        name: "identity".to_string(),
                        params: vec!["x".to_string()],
                        body: Box::new(Stmt::Expr(Expr::BindingUsage(BindingUsage {
                            name: "x".to_string(),
                        }))),
                    }),
                )),
            );
        }
    }
}
