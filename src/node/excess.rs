#[derive(Clone, Copy, Debug, Default)]
pub struct ExcessNode<F> {
    pub b: F,
    pub excess: F,
}