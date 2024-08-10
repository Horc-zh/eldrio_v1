use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Val {
    Number(i32),
    Unit,
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Val::Number(n) => write!(f, "{}", n),
            Val::Unit => write!(f, "Unit"),
        }
    }
}
