use crate::ids::NodeId;
use std::{error::Error as StdError, fmt};

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaximumFlowError {
    InvalidInput,
    NegativeCycle,
    NotSolved,
}

impl fmt::Display for MaximumFlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput =>
                write!(f, "invalid "),
            Self::NegativeCycle =>
                write!(f, "graph has negative cycle"),
            Self::NotSolved =>
                write!(f, "solver has not been run yet"),
        }
    }
}
impl StdError for MaximumFlowError {}
