use crate::algorithms::maximum_flow::{Dinic, FlowNum, MaximumFlowSolver, Status};
use crate::core::direction::Directed;
use crate::core::graph::Graph;
use crate::core::ids::NodeId;
use crate::edge::capacity::CapEdge;
use std::collections::VecDeque;

pub fn minimum_cut<Flow>(
    solver: &mut impl MaximumFlowSolver<Flow>,
    graph: &mut Graph<Directed, (), CapEdge<Flow>>,
    source: NodeId,
    sink: NodeId,
) -> Result<(Flow, Vec<NodeId>, Vec<bool>), Status>
where
    Flow: FlowNum,
{
    // 最大流を求める
    let max_flow = solver.solve(graph, source, sink, None)?;

    let mut g = vec![Vec::new(); graph.num_nodes()];
    for e in graph.edges.iter() {
        if e.data.upper - e.data.flow > Flow::zero() {
            g[e.u.index()].push(e.v.index())
        }
    }

    let mut visited = vec![false; graph.num_nodes()];
    let mut queue = VecDeque::new();
    visited[source.index()] = true;
    queue.push_back(source.index());

    while let Some(u) = queue.pop_front() {
        for &v in g[u].iter() {
            if !visited[v] {
                visited[v] = true;
                queue.push_back(v);
            }
        }
    }

    // S = visited が true なノード
    let cut_set = visited.iter().enumerate().filter(|&(_, &v)| v).map(|(i, _)| NodeId(i)).collect();

    Ok((max_flow, cut_set, visited))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maximum_flow::MaximumFlowGraph;

    #[test]
    fn test() {
        let mut graph = MaximumFlowGraph::default();
        let nodes = graph.add_nodes(4);
        graph.add_directed_edge(nodes[0], nodes[1], 2);
        graph.add_directed_edge(nodes[1], nodes[0], 2);
        
        
        graph.add_directed_edge(nodes[0], nodes[2], 1);
        graph.add_directed_edge(nodes[2], nodes[0], 1);
        
        graph.add_directed_edge(nodes[0], nodes[3], 1);
        graph.add_directed_edge(nodes[3], nodes[0], 1);
        
        graph.add_directed_edge(nodes[1], nodes[2], 1);
        graph.add_directed_edge(nodes[2], nodes[1], 1);
        
        graph.add_directed_edge(nodes[2], nodes[3], 3);
        graph.add_directed_edge(nodes[3], nodes[2], 3);
        
        let mut dinic = Dinic::<usize>::default();
        let (flow, cut, _) = minimum_cut(&mut dinic, &mut graph, nodes[2], nodes[3]).unwrap();

        println!("max flow = {}", flow);
        println!("min cut S = {:?}", cut.iter().map(|n| n.index()).collect::<Vec<_>>());
    }
}
