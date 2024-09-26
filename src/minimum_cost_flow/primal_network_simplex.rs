use crate::minimum_cost_flow::network_simplex_pivot_rules::PivotRule;
use crate::minimum_cost_flow::spanning_tree_structure::{EdgeState, InternalEdge, SpanningTreeStructure};
use crate::minimum_cost_flow::status::Status;
use num_traits::NumAssign;
use std::ops::Neg;

pub struct PrimalNetworkSimplex<'a, Flow> {
    st: &'a mut SpanningTreeStructure<Flow>,
}

impl<'a, Flow> PrimalNetworkSimplex<'a, Flow>
where
    Flow: NumAssign + Neg<Output = Flow> + Ord + Copy,
{
    pub fn new(st: &'a mut SpanningTreeStructure<Flow>) -> Self {
        Self { st }
    }

    pub fn solve<Pivot: PivotRule<Flow>>(&mut self, pivot: &mut Pivot) -> Status {
        if self.st.is_unbalance() {
            return Status::Unbalanced;
        }

        let (root, artificial_nodes, artificial_edges) = self.st.construct_extend_network_feasible_solution();
        self.st.root = root;
        debug_assert!(self.st.validate_num_successors(self.st.root));
        debug_assert!(self.st.satisfy_constraints());

        self.run(pivot, &artificial_edges);

        let status = if self.st.satisfy_constraints() { Status::Optimal } else { Status::Infeasible };
        self.st.remove_artificial_sub_graph(&artificial_nodes, &artificial_edges);

        status
    }

    pub fn solve_max_flow_with_min_cost<Pivot: PivotRule<Flow>>(&mut self, source: usize, sink: usize, pivot: &mut Pivot) -> (Status, Flow) {
        if source >= self.st.num_nodes || sink >= self.st.num_nodes {
            return (Status::BadInput, Flow::zero());
        }

        let status = self.solve(pivot);
        if status == Status::Infeasible {
            return (status, Flow::zero());
        }

        let infinite_flow = self.st.edges.iter().filter(|e| e.from == source).map(|e| e.upper).fold(Flow::zero(), |acc, upper| acc + upper);

        self.st.add_supply(source, infinite_flow);
        self.st.add_supply(sink, -infinite_flow);
        self.solve(pivot);
        self.st.excesses[source] = Flow::zero();
        self.st.excesses[sink] = Flow::zero();

        let maximum_flow = self.st.edges.iter().filter(|e| e.from == source).map(|e| e.flow).fold(Flow::zero(), |acc, upper| acc + upper);
        (Status::Optimal, maximum_flow)
    }

    pub(crate) fn run<Pivot: PivotRule<Flow>>(&mut self, pivot: &mut Pivot, artificial_edges: &[usize]) {
        while let Some(entering_edge_id) = pivot.find_entering_edge(self.st, Self::calculate_violation) {
            let (leaving_edge_id, apex, delta, t2_now_root, t2_new_root) = self.select_leaving_edge(entering_edge_id);
            self.st.update_flow_in_cycle(entering_edge_id, delta, apex);
            self.pivot(leaving_edge_id, entering_edge_id, t2_now_root, t2_new_root);

            debug_assert!(self.st.validate_num_successors(self.st.root));
            debug_assert!(self.st.satisfy_constraints());
        }

        // if there is remaining flow on the artificial edge, revert it
        for &edge_id in artificial_edges.iter() {
            let edge = &mut self.st.edges[edge_id];
            if edge.flow > Flow::zero() {
                self.st.excesses[edge.from] += edge.flow;
                self.st.excesses[edge.to] -= edge.flow;
                edge.flow = Flow::zero();
            }
        }
    }

    fn calculate_violation(edge: &InternalEdge<Flow>, st: &SpanningTreeStructure<Flow>) -> Flow {
        match edge.state {
            EdgeState::Upper => st.reduced_cost(edge),
            _ => -st.reduced_cost(edge),
        }
    }

    // keep strongly feasible solution
    pub fn select_leaving_edge(&self, entering_edge_id: usize) -> (usize, usize, Flow, usize, usize) {
        let entering_edge = &self.st.edges[entering_edge_id];

        let (from, to) = match entering_edge.state {
            EdgeState::Tree => panic!("state of entering edge {entering_edge_id} is invalid."),
            EdgeState::Lower => (entering_edge.from, entering_edge.to),
            EdgeState::Upper => (entering_edge.to, entering_edge.from),
        };

        let (mut leaving_edge_id, mut mini_delta, mut t2_now_root, mut t2_new_root) = (entering_edge_id, entering_edge.upper, usize::MAX, usize::MAX);

        let apex = {
            let (mut u, mut v) = (from, to);
            while u != v {
                let (u_num, v_num) = (self.st.num_successors[u], self.st.num_successors[v]);

                if u_num <= v_num {
                    let edge_id = self.st.nodes[u].parent_edge_id;
                    let edge = &self.st.edges[edge_id];
                    let delta = if u == edge.to { edge.residual_capacity() } else { edge.flow };

                    // search first blocking arc
                    if delta < mini_delta {
                        (leaving_edge_id, mini_delta, t2_now_root, t2_new_root) = (edge_id, delta, u, from);
                    }
                    u = self.st.nodes[u].parent;
                }

                if v_num <= u_num {
                    let edge_id = self.st.nodes[v].parent_edge_id;
                    let edge = &self.st.edges[edge_id];
                    let delta = if v == edge.from { edge.residual_capacity() } else { edge.flow };

                    // search last blocking arc
                    if delta <= mini_delta {
                        (leaving_edge_id, mini_delta, t2_now_root, t2_new_root) = (edge_id, delta, v, to);
                    }
                    v = self.st.nodes[v].parent;
                }
            }
            u
        };

        (leaving_edge_id, apex, mini_delta, t2_now_root, t2_new_root)
    }

    pub fn pivot(&mut self, leaving_edge_id: usize, entering_edge_id: usize, t2_now_root: usize, t2_new_root: usize) {
        if leaving_edge_id == entering_edge_id {
            self.st.edges[entering_edge_id].state = match self.st.edges[entering_edge_id].state {
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
            (t2_now_root, self.st.edges[entering_edge_id].opposite_side(t2_new_root), self.st.root, t2_new_root)
        } else {
            (self.st.root, t2_new_root, t2_now_root, self.st.edges[entering_edge_id].opposite_side(t2_new_root))
        };

        // enter entering edge and attach tree
        self.st.re_rooting(t2_now_root, t2_new_root, entering_edge_id);
        self.st.attach_tree(t1_new_root, new_attach_node, t2_new_root, entering_edge_id);
        self.st.root = t1_new_root;
        assert_eq!(self.st.nodes[self.st.root].parent, usize::MAX);
    }
}
