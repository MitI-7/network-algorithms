#[derive(Clone, Copy, Debug)]
pub struct CapCostEdge<F> {
    pub flow: F,
    pub lower: F,
    pub upper: F,
    pub cost: F,
}