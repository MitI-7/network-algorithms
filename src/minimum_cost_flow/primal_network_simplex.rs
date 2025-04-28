use crate::minimum_cost_flow::graph::Graph;
use crate::minimum_cost_flow::network_simplex_pivot_rules::{BlockSearchPivotRule, PivotRule};
use crate::minimum_cost_flow::spanning_tree_structure::{EdgeState, SpanningTreeStructure};
use crate::minimum_cost_flow::status::Status;
use crate::minimum_cost_flow::MinimumCostFlowSolver;
use num_traits::NumAssign;
use std::ops::Neg;

pub struct PrimalNetworkSimplex<Flow, Pivot = BlockSearchPivotRule<Flow>> {
    st: SpanningTreeStructure<Flow>,
    pivot: Pivot,
}

impl<Flow, Pivot> MinimumCostFlowSolver<Flow> for PrimalNetworkSimplex<Flow, Pivot>
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy + Default,
    Pivot: PivotRule<Flow>,
{
    fn solve(&mut self, graph: &mut Graph<Flow>) -> Result<Flow, Status> {
        if graph.is_unbalance() {
            return Err(Status::Unbalanced);
        }

        let inf_cost = graph.edges.iter().map(|e| e.cost).fold(Flow::one(), |acc, cost| acc + cost); // all edge costs are non-negative
        let (root, artificial_nodes, artificial_edges) = graph.construct_extend_network_feasible_solution();
        self.st.build(graph);
        (self.st.root, self.st.parent[root], self.st.parent_edge_id[root]) = (root, usize::MAX, usize::MAX);

        self.make_initial_spanning_tree_structure(graph, &artificial_edges, inf_cost);
        debug_assert!(self.st.validate_num_successors(self.st.root));
        debug_assert!(self.st.satisfy_constraints());

        self.run(&artificial_edges);

        // copy
        graph.excesses = self.st.excesses.to_vec();
        for edge_id in 0..graph.num_edges() {
            graph.edges[edge_id].flow = self.st.flow[edge_id];
        }
        graph.remove_artificial_sub_graph(&artificial_nodes, &artificial_edges);

        if self.st.satisfy_constraints() {
            Ok(graph.minimum_cost())
        } else {
            Err(Status::Infeasible)
        }
    }
}

impl<Flow> PrimalNetworkSimplex<Flow, BlockSearchPivotRule<Flow>>
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy + Default,
{
    pub fn new(num_edges: usize) -> Self {
        Self { st: SpanningTreeStructure::default(), pivot: BlockSearchPivotRule::new(num_edges) }
    }
}

impl<Flow, Pivot> PrimalNetworkSimplex<Flow, Pivot>
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy + Default,
    Pivot: PivotRule<Flow>,
{
    pub fn set_pivot<P: PivotRule<Flow>>(self, new_pivot: P) -> PrimalNetworkSimplex<Flow, P> {
        PrimalNetworkSimplex { st: self.st, pivot: new_pivot }
    }

    pub fn solve(&mut self, graph: &mut Graph<Flow>) -> Result<Flow, Status> {
        <Self as MinimumCostFlowSolver<Flow>>::solve(self, graph)
    }

    pub(crate) fn run(&mut self, artificial_edges: &[usize]) {
        while let Some(entering_edge_id) = self.pivot.find_entering_edge(&self.st, Self::calculate_violation) {
            let (leaving_edge_id, apex, delta, t2_now_root, t2_new_root) = self.select_leaving_edge(entering_edge_id);
            self.st.update_flow_in_cycle(entering_edge_id, delta, apex);
            self.pivot(leaving_edge_id, entering_edge_id, t2_now_root, t2_new_root);

            debug_assert!(self.st.validate_num_successors(self.st.root));
            debug_assert!(self.st.satisfy_constraints());
        }

        // if there is remaining flow on the artificial edge, revert it
        for &edge_id in artificial_edges.iter() {
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

    fn make_initial_spanning_tree_structure(&mut self, graph: &mut Graph<Flow>, artificial_edges: &[usize], inf_cost: Flow) {
        let mut prev_node = self.st.root;
        for &edge_id in artificial_edges.iter() {
            let edge = &graph.edges[edge_id];
            let u = if edge.from == self.st.root { edge.to } else { edge.from };

            if edge.from == u {
                (self.st.potential[u], self.st.state[edge_id]) = (inf_cost, EdgeState::Tree);
            } else {
                (self.st.potential[u], self.st.state[edge_id]) = (-inf_cost, EdgeState::Tree);
            }

            (self.st.parent[u], self.st.parent_edge_id[u]) = (self.st.root, edge_id);
            self.st.next_node_dft[prev_node] = u;
            self.st.prev_node_dft[u] = prev_node;
            self.st.last_descendent_dft[u] = u;
            self.st.num_successors[u] = 1;
            graph.excesses[u] = Flow::zero();
            prev_node = u;
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

        let (mut leaving_edge_id, mut mini_delta, mut t2_now_root, mut t2_new_root) =
            (entering_edge_id, self.st.upper[entering_edge_id], usize::MAX, usize::MAX);

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
