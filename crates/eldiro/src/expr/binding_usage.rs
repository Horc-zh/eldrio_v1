use crate::expr::func_call::FuncCall;
use crate::{env::Env, utils, val::Val};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct BindingUsage {
    pub(crate) name: String,
}

impl BindingUsage {
    pub(super) fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, name) = utils::extract_ident(s)?;

        Ok((
            s,
            Self {
                name: name.to_string(),
            },
        ))
    }
    pub(super) fn eval(&self, env: &Env) -> Result<Val, String> {
        env.get_binding(&self.name).or_else(|error_msg| {
            if env.get_func(&self.name).is_ok() {
                FuncCall {
                    callee: self.name.clone(),
                    params: Vec::new(),
                }
                .eval(env)
            } else {
                Err(error_msg)
            }
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_binding_usage() {
        assert_eq!(
            BindingUsage::new("abc"),
            Ok((
                "",
                BindingUsage {
                    name: "abc".to_string(),
                },
            )),
        );
    }
    #[test]
    fn eval_existing_binding_usage() {
        let mut env = Env::default();
        env.store_binding("foo".to_string(), Val::Number(10));

        assert_eq!(
            BindingUsage {
                name: "foo".to_string(),
            }
            .eval(&env),
            Ok(Val::Number(10)),
        );
    }
}
