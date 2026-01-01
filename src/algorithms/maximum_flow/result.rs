pub struct MaxFlowResult<F> {
    pub objective_value: F,
    pub flows: Vec<F>,
}