use std::{error::Error as StdError, fmt};

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MinimumCostFlowError {
    NotSolved,
    Unbalanced,
    Infeasible,
}

impl fmt::Display for MinimumCostFlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotSolved => write!(f, "solver has not been run yet"),
            Self::Unbalanced => write!(f, "unbalanced"),
            Self::Infeasible => write!(f, "infeasible"),
        }
    }
}
impl StdError for MinimumCostFlowError {}
