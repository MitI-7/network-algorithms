#[derive(Clone, Copy, Debug)]
pub struct CapEdge<F> {
    pub flow: F,
    pub upper: F,
}