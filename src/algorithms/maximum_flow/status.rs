#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum Status {
    #[default]
    NotSolved,
    Optimal,
}
