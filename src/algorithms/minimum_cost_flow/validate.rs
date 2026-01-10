use crate::{
    algorithms::minimum_cost_flow::{
        error::MinimumCostFlowError, residual_network::ResidualNetwork, spanning_tree_structure::SpanningTreeStructure,
    },
    core::numeric::CostNum,
};

pub(crate) fn validate_balance<F: CostNum>(rn: &ResidualNetwork<F>) -> Result<(), MinimumCostFlowError> {
    if rn.b.iter().fold(F::zero(), |sum, &e| sum + e) != F::zero() {
        return Err(MinimumCostFlowError::Unbalanced);
    }
    Ok(())
}

pub(crate) fn validate_balance_spanning_tree<F: CostNum>(
    st: &SpanningTreeStructure<F>,
) -> Result<(), MinimumCostFlowError> {
    if st.b.iter().fold(F::zero(), |sum, &e| sum + e) != F::zero() {
        return Err(MinimumCostFlowError::Unbalanced);
    }
    Ok(())
}

pub(crate) fn validate_infeasible<F: CostNum>(rn: &ResidualNetwork<F>) -> Result<(), MinimumCostFlowError> {
    if rn.num_edges == 0 && rn.b.iter().any(|&e| e != F::zero()) {
        return Err(MinimumCostFlowError::Infeasible);
    }
    Ok(())
}

pub(crate) fn validate_infeasible_spanning_tree<F: CostNum>(
    st: &SpanningTreeStructure<F>,
) -> Result<(), MinimumCostFlowError> {
    if st.num_edges == 0 && st.b.iter().any(|&e| e != F::zero()) {
        return Err(MinimumCostFlowError::Infeasible);
    }
    Ok(())
}

pub(crate) fn trivial_solution_if_any<F: CostNum>(rn: &ResidualNetwork<F>) -> Option<Result<F, MinimumCostFlowError>> {
    if rn.num_nodes == 0 || rn.num_edges == 0 {
        return Some(Ok(F::zero()));
    }

    None
}
