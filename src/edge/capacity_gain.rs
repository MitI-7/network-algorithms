#[derive(Clone, Copy, Debug, Default)]
pub struct CapGainEdge<F> {
    pub flow: F,
    pub lower: F,
    pub upper: F,
    pub gain: F,
}