#[derive(Clone, Debug)]
pub struct MinimumCostFlowEdge<F> {
    pub lower: F,
    pub upper: F,
    pub cost: F,
}