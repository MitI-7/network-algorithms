use network_algorithms::maximum_matching::{Blossom, Graph};

use std::io::*;
use std::str::FromStr;

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
    let (n, m) = (read(), read());
    let mut g = Graph::default();
    g.add_nodes(n);
    for _ in 0..m {
        let (u, v) = (read(), read());
        g.add_undirected_edge(u, v);
    }

    let matching = Blossom::default().solve(&g);
    println!("{}", matching.len());
    for edge_id in matching {
        let edge = g.get_edge(edge_id).unwrap();
        println!("{} {}", edge.u, edge.v);
    }
}
