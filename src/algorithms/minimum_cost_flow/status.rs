#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    NotSolved,
    BadInput,
    Unbalanced,
    Infeasible,
    // Optimal,
}
