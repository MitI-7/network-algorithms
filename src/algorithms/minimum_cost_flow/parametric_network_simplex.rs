use crate::algorithms::minimum_cost_flow::spanning_tree_structure::{EdgeState, SpanningTreeStructure};
use crate::algorithms::minimum_cost_flow::status::Status;
use crate::algorithms::minimum_cost_flow::{MinimumCostFlowNum, MinimumCostFlowSolver};
use std::collections::VecDeque;
use crate::core::direction::Directed;
use crate::core::graph::Graph;
use crate::core::ids::EdgeId;
use crate::edge::capacity_cost::CapCostEdge;
use crate::algorithms::minimum_cost_flow::csr::construct_extend_network_one_supply_one_demand;
use crate::algorithms::minimum_cost_flow::translater::translater;
use crate::node::excess::ExcessNode;

#[derive(Default)]
pub struct ParametricNetworkSimplex<Flow> {
    st: SpanningTreeStructure<Flow>,
    sink: usize,
}

impl<Flow> MinimumCostFlowSolver<Flow> for ParametricNetworkSimplex<Flow>
where
    Flow: MinimumCostFlowNum,
{
    fn solve(&mut self, graph: &mut Graph<Directed, ExcessNode<Flow>, CapCostEdge<Flow>>) -> Result<Flow, Status> {
        if (0..graph.num_nodes()).into_iter().fold(Flow::zero(), |sum, u| sum + graph.nodes[u].b) != Flow::zero() {
            return Err(Status::Unbalanced);
        }

        let mut new_graph = translater(graph);
        let (source, sink, _artificial_edges) = construct_extend_network_one_supply_one_demand(&mut new_graph);
        self.st.build(&mut new_graph);
        (self.st.root, self.sink) = (source.index(), sink.index());

        if !self.make_initial_spanning_tree_structure() {
            // there is no s-t path
            let status = if self.st.satisfy_constraints() { Status::Optimal } else { Status::Infeasible };
            // graph.remove_artificial_sub_graph(&[source, sink], &artificial_edges);

            return if status == Status::Optimal {
                for edge_id in 0..graph.num_edges() {
                    let edge = &graph.edges[edge_id];
                    assert!(self.st.flow[edge_id] <= self.st.upper[edge_id]);

                    graph.edges[edge_id].data.flow = if edge.data.cost >= Flow::zero() {
                        self.st.flow[edge_id] + edge.data.lower
                    }
                    else {
                        edge.data.upper - self.st.flow[edge_id]
                    };
                    assert!(graph.edges[edge_id].data.flow <= graph.edges[edge_id].data.upper);
                    assert!(graph.edges[edge_id].data.flow >= graph.edges[edge_id].data.lower);
                }

                Ok((0..graph.num_edges()).fold(Flow::zero(), |cost, edge_id| {
                    let edge = graph.get_edge(EdgeId(edge_id));
                    cost + edge.data.cost * edge.data.flow
                }))
            } else {
                Err(status)
            };
        }
        debug_assert!(self.st.satisfy_optimality_conditions());

        self.run();

        let status = if self.st.satisfy_constraints() { Status::Optimal } else { Status::Infeasible };
        // copy
        // graph.excesses = self.st.excesses.clone().to_vec();
        // for edge_id in 0..graph.num_edges() {
        //     graph.edges[edge_id].flow = self.st.flow[edge_id];
        // }
        // graph.remove_artificial_sub_graph(&artificial_nodes, &artificial_edges);

        for edge_id in 0..graph.num_edges() {
            let edge = &graph.edges[edge_id];
            assert!(self.st.flow[edge_id] <= self.st.upper[edge_id]);

            graph.edges[edge_id].data.flow = if edge.data.cost >= Flow::zero() {
                self.st.flow[edge_id] + edge.data.lower
            }
            else {
                edge.data.upper - self.st.flow[edge_id]
            };
            assert!(graph.edges[edge_id].data.flow <= graph.edges[edge_id].data.upper);
            assert!(graph.edges[edge_id].data.flow >= graph.edges[edge_id].data.lower);
        }

        if status == Status::Optimal {
            Ok((0..graph.num_edges()).fold(Flow::zero(), |cost, edge_id| {
                let edge = graph.get_edge(EdgeId(edge_id));
                cost + edge.data.cost * edge.data.flow
            }))
        } else {
            Err(status)
        }
    }
}

impl<Flow> ParametricNetworkSimplex<Flow>
where
    Flow: MinimumCostFlowNum,
{
    pub fn solve(&mut self, graph: &mut Graph<Directed, ExcessNode<Flow>, CapCostEdge<Flow>>) -> Result<Flow, Status> {
        <Self as MinimumCostFlowSolver<Flow>>::solve(self, graph)
    }

    pub(crate) fn run(&mut self) {
        while let Some((leaving_edge_id, delta)) = self.select_leaving_edge() {
            let t2_now_root = if self.st.parent[self.st.from[leaving_edge_id]] == self.st.to[leaving_edge_id] {
                self.st.from[leaving_edge_id]
            } else {
                self.st.to[leaving_edge_id]
            };

            self.st.update_flow_in_path(self.st.root, self.sink, delta);
            if self.st.excesses[self.st.root] == Flow::zero() {
                break;
            }

            if let Some((entering_edge_id, t2_new_root)) = self.select_entering_edge_id(leaving_edge_id, t2_now_root) {
                self.dual_pivot(leaving_edge_id, entering_edge_id, t2_now_root, t2_new_root);
                debug_assert!(self.st.satisfy_optimality_conditions());
            } else {
                break;
            }
        }
    }

    // T: shortest path
    // L: A \ T
    // U: empty
    fn make_initial_spanning_tree_structure(&mut self) -> bool {
        let (distances, prev_edge_id) = self.st.shortest_path(self.st.root);

        // there is no s-t path
        if prev_edge_id[self.sink].is_none() {
            return false;
        }

        // make tree structure
        let mut children = vec![Vec::new(); self.st.num_nodes];
        for edge_id in prev_edge_id.iter().filter_map(|&edge_id| edge_id) {
            self.st.state[edge_id] = EdgeState::Tree;
            let (from, to) = (self.st.from[edge_id], self.st.to[edge_id]);
            (self.st.parent[to], self.st.parent_edge_id[to]) = (from, edge_id);
            children[from].push(to);
        }
        (self.st.parent[self.st.root], self.st.parent_edge_id[self.st.root]) = (usize::MAX, usize::MAX);
        self.st.last_descendent_dft = (0..self.st.num_nodes).collect();

        let mut prev_node = usize::MAX;
        let mut stack = VecDeque::from([(self.st.root, usize::MAX)]);
        let mut seen = vec![false; self.st.num_nodes];
        while let Some((u, parent)) = stack.pop_back() {
            if seen[u] {
                self.st.num_successors[u] += 1;
                if parent != usize::MAX {
                    self.st.last_descendent_dft[parent] = self.st.last_descendent_dft[u];
                    self.st.num_successors[self.st.parent[u]] += self.st.num_successors[u];
                }
                continue;
            }

            seen[u] = true;
            self.st.prev_node_dft[u] = prev_node;
            if prev_node != usize::MAX {
                self.st.next_node_dft[prev_node] = u;
            }
            prev_node = u;
            stack.push_back((u, parent));
            for &child in children[u].iter().rev() {
                stack.push_back((child, u));
            }
        }
        self.st.next_node_dft[prev_node] = self.st.root;

        // determine potentials
        for u in 0..self.st.num_nodes {
            self.st.potential[u] = -distances[u];
        }

        true
    }

    fn select_leaving_edge(&self) -> Option<(usize, Flow)> {
        let mut leaving_edge_id = None;
        let mut mini_delta = Flow::zero();
        let mut now = self.sink;
        while now != self.st.root {
            let (parent, edge_id) = (self.st.parent[now], self.st.parent_edge_id[now]);
            assert_eq!(self.st.state[edge_id], EdgeState::Tree);

            let delta = if self.st.from[edge_id] == parent {
                self.st.residual_capacity(edge_id)
            } else {
                self.st.flow[edge_id]
            };
            // select the edge closest to the source as the leaving edge
            if leaving_edge_id.is_none() || delta <= mini_delta {
                mini_delta = delta;
                leaving_edge_id = Some(edge_id);
            }

            now = parent;
        }
        Some((leaving_edge_id?, mini_delta.min(self.st.excesses[self.st.root])))
    }

    fn select_entering_edge_id(&self, leaving_edge_id: usize, t2_now_root: usize) -> Option<(usize, usize)> {
        let mut is_t1_node = vec![false; self.st.num_nodes];
        let mut now = self.st.root;
        loop {
            is_t1_node[now] = true;
            now = self.st.next_node_dft[now];
            if now == t2_now_root {
                now = self.st.next_node_dft[self.st.last_descendent_dft[now]];
            }
            if now == self.st.root {
                break;
            }
        }

        let mut entering_edge_id = None;
        let mut t2_new_root = None;
        let mut mini_delta = Flow::zero();
        for edge_id in 0..self.st.num_edges {
            if edge_id == leaving_edge_id {
                continue;
            }

            // t1 -> t2 and lower
            let (from, to) = (self.st.from[edge_id], self.st.to[edge_id]);
            if is_t1_node[from] && !is_t1_node[to] && self.st.state[edge_id] == EdgeState::Lower && self.st.upper[edge_id] != Flow::zero() {
                let reduced_cost = self.st.reduced_cost(edge_id);
                if reduced_cost < mini_delta || entering_edge_id.is_none() {
                    mini_delta = reduced_cost;
                    entering_edge_id = Some(edge_id);
                    t2_new_root = Some(to);
                }
            }

            // t2 -> t1 and upper
            if !is_t1_node[from] && is_t1_node[to] && self.st.state[edge_id] == EdgeState::Upper {
                let reduced_cost = -self.st.reduced_cost(edge_id);
                if reduced_cost < mini_delta || entering_edge_id.is_none() {
                    mini_delta = reduced_cost;
                    entering_edge_id = Some(edge_id);
                    t2_new_root = Some(from);
                }
            }
        }
        Some((entering_edge_id?, t2_new_root?))
    }

    fn dual_pivot(&mut self, leaving_edge_id: usize, entering_edge_id: usize, t2_now_root: usize, t2_new_root: usize) {
        if leaving_edge_id == entering_edge_id {
            self.st.state[entering_edge_id] = match self.st.state[entering_edge_id] {
                EdgeState::Upper => EdgeState::Lower,
                EdgeState::Lower => EdgeState::Upper,
                _ => panic!("state of entering edge {entering_edge_id} is invalid."),
            };
            return;
        }

        // drop leaving edge
        self.st.detach_tree(self.st.root, t2_now_root, leaving_edge_id);

        // enter entering edge
        let attach_node = self.st.opposite_side(t2_new_root, entering_edge_id);
        self.st.re_rooting(t2_now_root, t2_new_root, entering_edge_id);
        self.st.attach_tree(self.st.root, attach_node, t2_new_root, entering_edge_id);
    }
}
