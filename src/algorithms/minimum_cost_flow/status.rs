#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Status {
    Optimal,
    #[default]
    NotSolved,
    BadInput,
    Unbalanced,
    Infeasible,
}
