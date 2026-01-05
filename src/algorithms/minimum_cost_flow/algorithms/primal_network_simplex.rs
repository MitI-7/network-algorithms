use crate::core::direction::Directed;
use crate::core::graph::Graph;
use crate::core::ids::EdgeId;
use crate::edge::capacity_cost::CapCostEdge;
use crate::algorithms::minimum_cost_flow::network_simplex_pivot_rules::{BlockSearchPivotRule, PivotRule};
use crate::algorithms::minimum_cost_flow::spanning_tree_structure::{EdgeState, SpanningTreeStructure};
use crate::algorithms::minimum_cost_flow::status::Status;
use crate::algorithms::minimum_cost_flow::{MinimumCostFlowNum, MinimumCostFlowSolver};
use crate::algorithms::minimum_cost_flow::csr::construct_extend_network_feasible_solution;
use crate::algorithms::minimum_cost_flow::translater::translater;
use crate::node::excess::ExcessNode;

#[derive(Default)]
pub struct PrimalNetworkSimplex<Flow, Pivot = BlockSearchPivotRule<Flow>> {
    st: SpanningTreeStructure<Flow>,
    pivot: Pivot,
}

impl<Flow, Pivot> MinimumCostFlowSolver<Flow> for PrimalNetworkSimplex<Flow, Pivot>
where
    Flow: MinimumCostFlowNum,
    Pivot: PivotRule<Flow>,
{
    fn solve(&mut self, graph: &mut Graph<Directed, ExcessNode<Flow>, CapCostEdge<Flow>>) -> Result<Flow, Status> {
        if (0..graph.num_nodes()).into_iter().fold(Flow::zero(), |sum, u| sum + graph.nodes[u].b) != Flow::zero() {
            return Err(Status::Unbalanced);
        }
        
        let mut new_graph = translater(graph);

        let inf_cost = new_graph.edges.iter().map(|e| e.data.cost).fold(Flow::one(), |acc, cost| acc + cost); // all edge costs are non-negative
        let (root, _artificial_nodes, artificial_edges) = construct_extend_network_feasible_solution(&mut new_graph);
        self.st.build(&mut new_graph);
        (self.st.root, self.st.parent[root.index()], self.st.parent_edge_id[root.index()]) = (root.index(), usize::MAX, usize::MAX);

        self.make_initial_spanning_tree_structure(&mut new_graph, &artificial_edges, inf_cost);
        debug_assert!(self.st.validate_num_successors(self.st.root));
        debug_assert!(self.st.satisfy_constraints());

        self.pivot.initialize(self.st.num_edges);
        self.run(&artificial_edges);

        // copy
        // graph.excesses = self.st.excesses.to_vec();
        // for edge_id in 0..graph.num_edges() {
        //     graph.edges[edge_id].flow = self.st.flow[edge_id];
        // }
        for edge_id in 0..graph.num_edges() {
            let edge = &graph.edges[edge_id];
            graph.edges[edge_id].data.flow = if edge.data.cost >= Flow::zero() {
                self.st.flow[edge_id] + edge.data.lower
            }
            else {
                edge.data.upper - self.st.flow[edge_id]
            };
            assert!(graph.edges[edge_id].data.flow <= graph.edges[edge_id].data.upper);
            assert!(graph.edges[edge_id].data.flow >= graph.edges[edge_id].data.lower);
        }
        // graph.remove_artificial_sub_graph(&artificial_nodes, &artificial_edges);

        self.pivot.clear();
        if self.st.satisfy_constraints() {
            Ok((0..graph.num_edges()).fold(Flow::zero(), |cost, edge_id| {
                let edge = graph.get_edge(EdgeId(edge_id));
                cost + edge.data.cost * edge.data.flow
            }))
        } else {
            Err(Status::Infeasible)
        }
    }
}

impl<Flow, Pivot> PrimalNetworkSimplex<Flow, Pivot>
where
    Flow: MinimumCostFlowNum,
    Pivot: PivotRule<Flow>,
{
    pub fn set_pivot<P: PivotRule<Flow>>(self, new_pivot: P) -> PrimalNetworkSimplex<Flow, P> {
        PrimalNetworkSimplex { st: self.st, pivot: new_pivot }
    }

    pub fn solve(&mut self, graph: &mut Graph<Directed, ExcessNode<Flow>, CapCostEdge<Flow>>) -> Result<Flow, Status> {
        <Self as MinimumCostFlowSolver<Flow>>::solve(self, graph)
    }

    pub(crate) fn run(&mut self, artificial_edges: &[EdgeId]) {
        while let Some(entering_edge_id) = self.pivot.find_entering_edge(&self.st, Self::calculate_violation) {
            let (leaving_edge_id, apex, delta, t2_now_root, t2_new_root) = self.select_leaving_edge(entering_edge_id);
            self.st.update_flow_in_cycle(entering_edge_id, delta, apex);
            self.pivot(leaving_edge_id, entering_edge_id, t2_now_root, t2_new_root);

            debug_assert!(self.st.validate_num_successors(self.st.root));
            debug_assert!(self.st.satisfy_constraints());
        }

        // if there is remaining flow on the artificial edge, revert it
        for &edge_id in artificial_edges.iter() {
            let edge_id = edge_id.index();
            if self.st.flow[edge_id] > Flow::zero() {
                self.st.excesses[self.st.from[edge_id]] += self.st.flow[edge_id];
                self.st.excesses[self.st.to[edge_id]] -= self.st.flow[edge_id];
                self.st.flow[edge_id] = Flow::zero();
            }
        }
    }

    fn calculate_violation(edge_id: usize, st: &SpanningTreeStructure<Flow>) -> Flow {
        match st.state[edge_id] {
            EdgeState::Upper => st.reduced_cost(edge_id),
            _ => -st.reduced_cost(edge_id),
        }
    }

    fn make_initial_spanning_tree_structure(&mut self, graph: &mut Graph<Directed, ExcessNode<Flow>, CapCostEdge<Flow>>, artificial_edges: &[EdgeId], inf_cost: Flow) {
        let mut prev_node = self.st.root;
        for &edge_id in artificial_edges.iter() {
            let edge = &graph.edges[edge_id.index()];
            let u = if edge.u.index() == self.st.root { edge.v } else { edge.u };

            if edge.u == u {
                (self.st.potential[u.index()], self.st.state[edge_id.index()]) = (inf_cost, EdgeState::Tree);
            } else {
                (self.st.potential[u.index()], self.st.state[edge_id.index()]) = (-inf_cost, EdgeState::Tree);
            }

            (self.st.parent[u.index()], self.st.parent_edge_id[u.index()]) = (self.st.root, edge_id.index());
            self.st.next_node_dft[prev_node] = u.index();
            self.st.prev_node_dft[u.index()] = prev_node;
            self.st.last_descendent_dft[u.index()] = u.index();
            self.st.num_successors[u.index()] = 1;
            // graph.excesses[u] = Flow::zero();
            prev_node = u.index();
        }
        self.st.next_node_dft[prev_node] = self.st.root;
        self.st.prev_node_dft[self.st.root] = prev_node;
        self.st.last_descendent_dft[self.st.root] = prev_node;

        self.st.num_successors[self.st.root] = graph.num_nodes();
    }

    // keep strongly feasible solution
    fn select_leaving_edge(&self, entering_edge_id: usize) -> (usize, usize, Flow, usize, usize) {
        let (from, to) = match self.st.state[entering_edge_id] {
            EdgeState::Tree => panic!("state of entering edge {entering_edge_id} is invalid."),
            EdgeState::Lower => (self.st.from[entering_edge_id], self.st.to[entering_edge_id]),
            EdgeState::Upper => (self.st.to[entering_edge_id], self.st.from[entering_edge_id]),
        };

        let (mut leaving_edge_id, mut mini_delta, mut t2_now_root, mut t2_new_root) = (entering_edge_id, self.st.upper[entering_edge_id], usize::MAX, usize::MAX);

        let apex = {
            let (mut u, mut v) = (from, to);
            while u != v {
                let (u_num, v_num) = (self.st.num_successors[u], self.st.num_successors[v]);

                if u_num <= v_num {
                    let edge_id = self.st.parent_edge_id[u];
                    let delta = if u == self.st.to[edge_id] {
                        self.st.residual_capacity(edge_id)
                    } else {
                        self.st.flow[edge_id]
                    };

                    // search first blocking arc
                    if delta < mini_delta {
                        (leaving_edge_id, mini_delta, t2_now_root, t2_new_root) = (edge_id, delta, u, from);
                    }
                    u = self.st.parent[u];
                }

                if v_num <= u_num {
                    let edge_id = self.st.parent_edge_id[v];
                    let delta = if v == self.st.from[edge_id] {
                        self.st.residual_capacity(edge_id)
                    } else {
                        self.st.flow[edge_id]
                    };

                    // search last blocking arc
                    if delta <= mini_delta {
                        (leaving_edge_id, mini_delta, t2_now_root, t2_new_root) = (edge_id, delta, v, to);
                    }
                    v = self.st.parent[v];
                }
            }
            u
        };

        (leaving_edge_id, apex, mini_delta, t2_now_root, t2_new_root)
    }

    fn pivot(&mut self, leaving_edge_id: usize, entering_edge_id: usize, t2_now_root: usize, t2_new_root: usize) {
        if leaving_edge_id == entering_edge_id {
            self.st.state[entering_edge_id] = match self.st.state[entering_edge_id] {
                EdgeState::Upper => EdgeState::Lower,
                EdgeState::Lower => EdgeState::Upper,
                _ => panic!("state of entering edge {entering_edge_id} is invalid."),
            };
            return;
        }

        // drop leaving edge and detach tree
        self.st.detach_tree(self.st.root, t2_now_root, leaving_edge_id);

        // if the size of subtree t2 is larger than that of subtree t1, swap t1 and t2.
        let (t1_new_root, t2_new_root, t2_now_root, new_attach_node) = if self.st.num_successors[t2_now_root] * 2 >= self.st.num_nodes {
            (t2_now_root, self.st.opposite_side(t2_new_root, entering_edge_id), self.st.root, t2_new_root)
        } else {
            (self.st.root, t2_new_root, t2_now_root, self.st.opposite_side(t2_new_root, entering_edge_id))
        };

        // enter entering edge and attach tree
        self.st.re_rooting(t2_now_root, t2_new_root, entering_edge_id);
        self.st.attach_tree(t1_new_root, new_attach_node, t2_new_root, entering_edge_id);
        self.st.root = t1_new_root;
        assert_eq!(self.st.parent[self.st.root], usize::MAX);
    }
}
