use network_algorithms::minimum_cost_flow::network_simplex_pivot_rules::{BlockSearchPivotRule, PivotRule};
use network_algorithms::minimum_cost_flow::primal_network_simplex::PrimalNetworkSimplex;
use network_algorithms::minimum_cost_flow::spanning_tree_structure::SpanningTreeStructure;
use network_algorithms::minimum_cost_flow::status::Status;

fn main() {
    let mut st = SpanningTreeStructure::<i32>::default();
    st.add_nodes(4);

    let edges = vec![
        st.add_directed_edge(0, 1, 0, 2, 1).unwrap(),
        st.add_directed_edge(0, 2, 0, 1, 2).unwrap(),
        st.add_directed_edge(1, 2, 0, 1, 1).unwrap(),
        st.add_directed_edge(1, 3, 0, 1, 3).unwrap(),
        st.add_directed_edge(2, 3, 0, 2, 1).unwrap(),
    ];

    st.add_supply(0, 2);
    st.add_supply(3, -2);

    let mut pivot_rule = BlockSearchPivotRule::new(edges.len());
    let mut solver = PrimalNetworkSimplex::new(&mut st);

    match solver.solve(&mut pivot_rule) {
        Status::Optimal => {
            println!("minimum cost:{}", st.minimum_cost());
            for edge_id in edges {
                println!("{:?}", st.get_edge(edge_id).unwrap());
            }
        }
        _ => unreachable!(),
    }
}
