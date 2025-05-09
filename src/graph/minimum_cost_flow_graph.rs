use crate::minimum_cost_flow::MinimumCostFlowNum;
use std::fmt::Debug;

#[derive(PartialEq, Debug, Clone)]
pub struct Edge<Flow> {
    pub from: usize,
    pub to: usize,
    pub flow: Flow,
    pub lower: Flow,
    pub upper: Flow,
    pub cost: Flow,
}

#[derive(Default, Clone)]
pub struct Graph<Flow> {
    num_nodes: usize,
    num_edges: usize,
    pub(crate) edges: Vec<Edge<Flow>>,
    pub(crate) b: Vec<Flow>,
    // pub(crate) excesses: Vec<Flow>,
}

impl<Flow> Graph<Flow>
where
    Flow: MinimumCostFlowNum,
{
    #[inline]
    pub fn num_nodes(&self) -> usize {
        self.num_nodes
    }

    #[inline]
    pub fn num_edges(&self) -> usize {
        self.num_edges
    }

    pub fn add_node(&mut self) -> usize {
        self.b.push(Flow::zero());
        // self.excesses.push(Flow::zero());
        self.num_nodes += 1;
        self.num_nodes - 1
    }

    pub fn add_nodes(&mut self, num_nodes: usize) -> Vec<usize> {
        self.b.extend(vec![Flow::zero(); num_nodes]);
        // self.excesses.extend(vec![Flow::zero(); num_nodes]);
        self.num_nodes += num_nodes;
        ((self.num_nodes - num_nodes)..self.num_nodes).collect()
    }

    pub fn add_supply(&mut self, u: usize, supply: Flow) {
        self.b[u] += supply;
        // self.excesses[u] += supply;
    }

    pub fn add_demand(&mut self, u: usize, demand: Flow) {
        self.b[u] -= demand;
        // self.excesses[u] -= demand;
    }

    // return edge index
    pub fn add_directed_edge(&mut self, from: usize, to: usize, lower: Flow, upper: Flow, cost: Flow) -> Option<usize> {
        if lower > upper || from >= self.num_nodes || to >= self.num_nodes {
            return None;
        }

        self.edges.push(Edge { from, to, flow: Flow::zero(), lower, upper, cost });

        self.num_edges += 1;
        Some(self.num_edges - 1)
    }

    pub fn get_edge(&self, edge_id: usize) -> Option<&Edge<Flow>> {
        if edge_id >= self.edges.len() {
            return None;
        }
        Some(&self.edges[edge_id])
        
        // let lower = self.lowers[edge_id];
        // if self.is_reversed[edge_id] {
        //     Some(Edge { from: edge.to, to: edge.from, flow: edge.upper - edge.flow + lower, lower, upper: edge.upper + lower, cost: -edge.cost })
        // } else {
        //     Some(Edge { from: edge.from, to: edge.to, flow: edge.flow + lower, lower, upper: edge.upper + lower, cost: edge.cost })
        // }
    }

    pub fn reset(&mut self) {
        // for u in 0..self.num_nodes {
        //     self.excesses[u] = self.b[u];
        // }
        self.edges.iter_mut().enumerate().for_each(|(_edge_id, edge)| {
            edge.flow = Flow::zero();
            // if self.is_reversed[edge_id] {
            //     let u = self.lowers[edge_id] + edge.upper;
            //     self.excesses[edge.from] += u;
            //     self.excesses[edge.to] -= u;
            // }
        });
    }

    pub fn minimum_cost(&self) -> Flow {
        (0..self.num_edges).fold(Flow::zero(), |cost, edge_id| {
            let edge = self.get_edge(edge_id).unwrap();
            cost + edge.cost * edge.flow
        })
    }

    pub fn is_unbalance(&self) -> bool {
        self.b.iter().fold(Flow::zero(), |sum, &excess| sum + excess) != Flow::zero()
    }

    // pub(crate) fn remove_artificial_sub_graph(&mut self, artificial_nodes: &[usize], artificial_edges: &[usize]) {
    //     self.edges.truncate(self.num_edges - artificial_edges.len());
    //     self.b.truncate(self.num_nodes - artificial_nodes.len());
    //     self.lowers.truncate(self.num_edges - artificial_edges.len());
    //     self.excesses.truncate(self.num_nodes - artificial_nodes.len());
    //     self.is_reversed.truncate(self.num_edges - artificial_edges.len());
    // 
    //     self.num_nodes -= artificial_nodes.len();
    //     self.num_edges -= artificial_edges.len();
    // }
}

pub(crate) fn construct_extend_network_one_supply_one_demand<Flow>(graph: &Graph<Flow>) -> (usize, usize, Vec<Edge<Flow>>, Flow)
where
    Flow: MinimumCostFlowNum,
{
    let mut artificial_edges = Vec::new();
    let source = graph.num_nodes;
    let sink = source + 1;
    let mut total_excess = Flow::zero();

    for u in 0..graph.num_nodes() {
        if u == source || u == sink {
            continue;
        }
        if graph.b[u] > Flow::zero() {
            artificial_edges.push(Edge{from: source, to: u, flow: Flow::zero(), lower:Flow::zero(), upper:graph.b[u], cost:Flow::zero()});
            total_excess += graph.b[u];
        }
        if graph.b[u] < Flow::zero() {
            artificial_edges.push(Edge{from: u, to: sink, flow: Flow::zero(), lower: Flow::zero(), upper: -graph.b[u], cost: Flow::zero()});
        }
    }

    (source, sink, artificial_edges, total_excess)
}

pub(crate) fn construct_extend_network_feasible_solution<Flow>(graph: &mut Graph<Flow>) -> (usize, Vec<usize>, Vec<usize>)
where
    Flow: MinimumCostFlowNum,
{
    let inf_cost = graph.edges.iter().map(|e| e.cost).fold(Flow::one(), |acc, cost| acc + cost); // all edge costs are non-negative

    // add artificial nodes
    let root = graph.add_node();

    // add artificial edges
    let mut artificial_edges = Vec::new();
    for u in 0..graph.num_nodes {
        if u == root {
            continue;
        }

        let excess = graph.b[u];
        if excess >= Flow::zero() {
            // u -> root
            let edge_id = graph.add_directed_edge(u, root, Flow::zero(), excess, inf_cost).unwrap();
            graph.edges[edge_id].flow = excess;
            artificial_edges.push(edge_id);
        } else {
            // root -> u
            let edge_id = graph.add_directed_edge(root, u, Flow::zero(), -excess, inf_cost).unwrap();
            graph.edges[edge_id].flow = -excess;
            artificial_edges.push(edge_id);
        }
        graph.b[u] = Flow::zero();
    }

    (root, vec![root], artificial_edges)
}
