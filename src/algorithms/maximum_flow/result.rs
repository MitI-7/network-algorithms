pub struct MaximumFlowResult<F> {
    pub objective_value: F,
    pub flows: Vec<F>,
}

pub struct MinimumCutResult<F> {
    pub objective_value: F,
    pub minimum_cut: Vec<bool>,
}