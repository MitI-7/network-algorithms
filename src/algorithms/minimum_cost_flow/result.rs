pub struct MinimumCostFlowResult<F> {
    pub objective_value: F,
    pub flows: Vec<F>,
    // bも必要?
}