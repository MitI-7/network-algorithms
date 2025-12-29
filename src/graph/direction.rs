pub trait Direction {
    const IS_DIRECTED: bool;
}

#[derive(Clone, Copy)]
pub struct Directed;

#[derive(Clone, Copy)]
pub struct Undirected;

impl Direction for Directed {
    const IS_DIRECTED: bool = true;
}

impl Direction for Undirected {
    const IS_DIRECTED: bool = false;
}
