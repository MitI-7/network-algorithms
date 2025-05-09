#[derive(Clone, Copy, Debug, Default)]
pub struct CapCostEdge<F> {
    pub flow: F,
    pub lower: F,
    pub upper: F,
    pub cost: F,
}