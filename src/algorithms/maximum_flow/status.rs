#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum Status {
    BadInput,
    #[default]
    NotSolved,
    Optimal,
}
