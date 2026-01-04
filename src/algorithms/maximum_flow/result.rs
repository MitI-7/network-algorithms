pub struct MaximumFlowResult<F> {
    pub objective_value: F,
    pub flows: Vec<F>,
}

pub struct MinimumCutResult<F> {
    pub objective_value: F,
    pub source_side: Vec<bool>,
}