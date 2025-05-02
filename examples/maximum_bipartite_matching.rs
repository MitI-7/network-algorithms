use network_algorithms::maximum_bipartite_matching::hopcroft_karp::HopcroftKarp;
use std::collections::VecDeque;

use std::io::*;
use std::str::FromStr;
use std::vec;

fn read<T: FromStr>() -> T {
    let stdin = stdin();
    let stdin = stdin.lock();
    let token: String = stdin
        .bytes()
        .map(|c| c.expect("failed to read char") as char)
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| !c.is_whitespace())
        .collect();
    token.parse().ok().expect("failed to parse token")
}

fn main() {
    // let (num_left_nodes, num_right_nodes, num_edges) = (read(), read(), read());
    // let mut hk = HopcroftKarp::new(num_left_nodes, num_right_nodes);
    //
    // for _ in 0..num_edges {
    //     let (a, b) = (read(), read());
    //     hk.add_edge(a, b);
    // }
    //
    // println!("{}", hk.solve().len());
}
