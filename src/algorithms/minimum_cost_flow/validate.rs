use crate::{
    algorithms::minimum_cost_flow::{
        residual_network::ResidualNetwork, spanning_tree_structure::SpanningTreeStructure, status::Status,
    },
    core::numeric::CostNum,
};

pub(crate) fn validate_balance<F: CostNum>(rn: &ResidualNetwork<F>) -> Result<(), Status> {
    if rn.b.iter().fold(F::zero(), |sum, &e| sum + e) != F::zero() {
        return Err(Status::Unbalanced);
    }
    Ok(())
}

pub(crate) fn validate_balance_spanning_tree<F: CostNum>(st: &SpanningTreeStructure<F>) -> Result<(), Status> {
    if st.b.iter().fold(F::zero(), |sum, &e| sum + e) != F::zero() {
        return Err(Status::Unbalanced);
    }
    Ok(())
}

pub(crate) fn validate_infeasible<F: CostNum>(rn: &ResidualNetwork<F>) -> Result<(), Status> {
    if rn.num_edges == 0 && rn.b.iter().any(|&e| e != F::zero()) {
        return Err(Status::Infeasible);
    }
    Ok(())
}

pub(crate) fn validate_infeasible_spanning_tree<F: CostNum>(st: &SpanningTreeStructure<F>) -> Result<(), Status> {
    if st.num_edges == 0 && st.b.iter().any(|&e| e != F::zero()) {
        return Err(Status::Infeasible);
    }
    Ok(())
}

pub(crate) fn trivial_solution_if_any<F: CostNum>(rn: &ResidualNetwork<F>) -> Option<Result<F, Status>> {
    if rn.num_nodes == 0 || rn.num_edges == 0 {
        return Some(Ok(F::zero()));
    }

    None
}
