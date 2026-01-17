use crate::ids::{EdgeId, NodeId};
use std::{error::Error as StdError, fmt};

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaximumFlowError {
    InvalidTerminal {
        source: NodeId,
        sink: NodeId,
        num_nodes: usize,
    },
    InvalidEdgeId {
        edge_id: EdgeId,
    },
    NotSolved,
}

impl fmt::Display for MaximumFlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTerminal { source, sink, num_nodes } => {
                write!(f, "invalid source/sink (source={source:?}, sink={sink:?}, num_nodes={num_nodes})")
            }
            Self::InvalidEdgeId { edge_id } => write!(f, "invalid edge id (edge id={edge_id:?})"),
            Self::NotSolved => write!(f, "solver has not been run yet"),
        }
    }
}
impl StdError for MaximumFlowError {}
