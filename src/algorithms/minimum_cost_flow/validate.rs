use crate::minimum_cost_flow::residual_network::ResidualNetwork;
use crate::{
    algorithms::minimum_cost_flow::status::Status,
    core::numeric::CostNum,
};

pub fn validate_balance<F: CostNum>(rn: &ResidualNetwork<F>) -> Result<(), Status> {
    if rn.excesses.iter().fold(F::zero(), |sum, &e| sum + e) != F::zero() {
        return Err(Status::Unbalanced);
    }
    Ok(())
}

pub fn validate_infeasible<F: CostNum>(rn: &ResidualNetwork<F>) -> Result<(), Status> {
    if rn.num_edges == 0 {
        if rn.excesses.iter().any(|&e| e != F::zero()) {
            return Err(Status::Infeasible);
        }
    }
    Ok(())
}

pub fn trivial_solution_if_any<F: CostNum>(rn: &ResidualNetwork<F>) -> Option<Result<F, Status>> {
    if rn.num_nodes == 0 ||rn.num_edges == 0 {
        return Some(Ok(F::zero()));
    }

    None
}
