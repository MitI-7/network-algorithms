use crate::ids::NodeId;
use std::{error::Error as StdError, fmt};

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaximumFlowError {
    InvalidTerminal { source: NodeId, sink: NodeId, num_nodes: usize },
    NotSolved,
}

impl fmt::Display for MaximumFlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTerminal { source, sink, num_nodes } =>
                write!(f, "invalid source/sink (source={source:?}, sink={sink:?}, num_nodes={num_nodes})"),
            Self::NotSolved =>
                write!(f, "solver has not been run yet"),
        }
    }
}
impl StdError for MaximumFlowError {}
