#[derive(Clone, Copy, Debug, Default)]
pub struct CapEdge<F> {
    pub flow: F,
    pub upper: F,
}