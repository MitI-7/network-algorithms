use crate::{
    algorithms::minimum_cost_flow::{
        edge::MinimumCostFlowEdge,
        extend_network::construct_extend_network_feasible_solution,
        node::MinimumCostFlowNode,
        normalized_network::{NormalizedEdge, NormalizedNetwork},
        solvers::{
            macros::impl_minimum_cost_flow_solver, network_simplex_pivot_rules::BlockSearchPivotRule,
            network_simplex_pivot_rules::PivotRule, solver::MinimumCostFlowSolver,
        },
        spanning_tree_structure::{EdgeState, SpanningTreeStructure},
        status::Status,
        validate::{validate_balance_spanning_tree, validate_infeasible_spanning_tree},
    },
    core::numeric::CostNum,
    graph::{
        direction::Directed,
        graph::Graph,
        ids::{EdgeId, INVALID_EDGE_ID, INVALID_NODE_ID, NodeId},
    },
};

pub struct PrimalNetworkSimplex<F, P = BlockSearchPivotRule<F>> {
    st: SpanningTreeStructure<F>,
    pivot: P,

    root: NodeId,
    artificial_edges: Vec<NormalizedEdge<F>>,
    inf_cost: F,
}

impl<F, P> PrimalNetworkSimplex<F, P>
where
    F: CostNum,
    P: PivotRule<F> + Default,
{
    pub fn set_pivot<P2: PivotRule<F>>(self, new_pivot: P2) -> PrimalNetworkSimplex<F, P2> {
        PrimalNetworkSimplex {
            st: self.st,
            pivot: new_pivot,
            root: self.root,
            artificial_edges: self.artificial_edges,
            inf_cost: self.inf_cost,
        }
    }

    fn new(graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Self {
        let nn = NormalizedNetwork::new(graph);

        let inf_cost = nn.iter_edges().map(|e| e.cost).fold(F::one(), |acc, cost| acc + cost); // all edge costs are non-negative

        let (root, artificial_edges, initial_flows, fix_excesses) = construct_extend_network_feasible_solution(&nn);
        let st = SpanningTreeStructure::new(
            &nn,
            Some(&[root]),
            Some(&artificial_edges),
            Some(&initial_flows),
            Some(&fix_excesses),
        );

        let mut pivot = P::default();
        pivot.initialize(st.num_edges);

        Self { st, pivot, root, artificial_edges, inf_cost }
    }

    fn run(&mut self) -> Result<F, Status> {
        validate_balance_spanning_tree(&self.st)?;
        validate_infeasible_spanning_tree(&self.st)?;

        self.st.root = self.root;
        self.st.parent[self.root.index()] = INVALID_NODE_ID;
        self.st.parent_edge_id[self.root.index()] = INVALID_EDGE_ID;

        self.make_initial_spanning_tree_structure(self.inf_cost);
        debug_assert!(self.st.validate_num_successors(self.st.root));
        debug_assert!(self.st.satisfy_constraints());

        self.pivot.initialize(self.st.num_edges);
        self.run2();

        self.pivot.clear();
        if !self.st.satisfy_constraints() {
            return Err(Status::Infeasible);
        }

        Ok(self.st.calculate_objective_value_in_original_graph())
    }

    pub(crate) fn run2(&mut self) {
        while let Some(entering_edge_id) = self.pivot.find_entering_edge(&self.st, Self::calculate_violation) {
            let (leaving_edge_id, apex, delta, t2_now_root, t2_new_root) = self.select_leaving_edge(entering_edge_id);
            self.st.update_flow_in_cycle(entering_edge_id, delta, apex);
            self.pivot(leaving_edge_id, entering_edge_id, t2_now_root, t2_new_root);

            debug_assert!(self.st.validate_num_successors(self.st.root));
            debug_assert!(self.st.satisfy_constraints());
        }

        // if there is remaining flow on the artificial edge, revert it
        for edge_id in self.st.num_edges_original_graph..self.st.num_edges {
            if self.st.flow[edge_id] > F::zero() {
                self.st.excesses[self.st.from[edge_id].index()] += self.st.flow[edge_id];
                self.st.excesses[self.st.to[edge_id].index()] -= self.st.flow[edge_id];
                self.st.flow[edge_id] = F::zero();
            }
        }
    }

    fn calculate_violation(edge_id: EdgeId, st: &SpanningTreeStructure<F>) -> F {
        match st.state[edge_id.index()] {
            EdgeState::Upper => st.reduced_cost(edge_id),
            _ => -st.reduced_cost(edge_id),
        }
    }

    fn make_initial_spanning_tree_structure(&mut self, inf_cost: F) {
        let mut prev_node = self.st.root;
        for edge_id in (self.st.num_edges_original_graph..self.st.num_edges).map(EdgeId) {
            let u = if self.st.from[edge_id.index()] == self.st.root {
                self.st.to[edge_id.index()]
            } else {
                self.st.from[edge_id.index()]
            };

            if self.st.from[edge_id.index()] == u {
                (self.st.potentials[u.index()], self.st.state[edge_id.index()]) = (inf_cost, EdgeState::Tree);
            } else {
                (self.st.potentials[u.index()], self.st.state[edge_id.index()]) = (-inf_cost, EdgeState::Tree);
            }

            (self.st.parent[u.index()], self.st.parent_edge_id[u.index()]) = (self.st.root, edge_id);
            self.st.next_node_dft[prev_node.index()] = u;
            self.st.prev_node_dft[u.index()] = prev_node;
            self.st.last_descendent_dft[u.index()] = u;
            self.st.num_successors[u.index()] = 1;
            // graph.excesses[u] = Flow::zero();
            prev_node = u;
        }
        self.st.next_node_dft[prev_node.index()] = self.st.root;
        self.st.prev_node_dft[self.st.root.index()] = prev_node;
        self.st.last_descendent_dft[self.st.root.index()] = prev_node;

        self.st.num_successors[self.st.root.index()] = self.st.num_nodes;
    }

    // keep strongly feasible solution
    fn select_leaving_edge(&self, entering_edge_id: EdgeId) -> (EdgeId, NodeId, F, NodeId, NodeId) {
        let (from, to) = match self.st.state[entering_edge_id.index()] {
            EdgeState::Tree => panic!("state of entering edge {} is invalid.", entering_edge_id.index()),
            EdgeState::Lower => (self.st.from[entering_edge_id.index()], self.st.to[entering_edge_id.index()]),
            EdgeState::Upper => (self.st.to[entering_edge_id.index()], self.st.from[entering_edge_id.index()]),
        };

        let (mut leaving_edge_id, mut mini_delta, mut t2_now_root, mut t2_new_root) =
            (entering_edge_id, self.st.upper[entering_edge_id.index()], INVALID_NODE_ID, INVALID_NODE_ID);

        let apex = {
            let (mut u, mut v) = (from, to);
            while u != v {
                let (u_num, v_num) = (self.st.num_successors[u.index()], self.st.num_successors[v.index()]);

                if u_num <= v_num {
                    let edge_id = self.st.parent_edge_id[u.index()];
                    let delta = if u == self.st.to[edge_id.index()] {
                        self.st.residual_capacity(edge_id)
                    } else {
                        self.st.flow[edge_id.index()]
                    };

                    // search first blocking arc
                    if delta < mini_delta {
                        (leaving_edge_id, mini_delta, t2_now_root, t2_new_root) = (edge_id, delta, u, from);
                    }
                    u = self.st.parent[u.index()];
                }

                if v_num <= u_num {
                    let edge_id = self.st.parent_edge_id[v.index()];
                    let delta = if v == self.st.from[edge_id.index()] {
                        self.st.residual_capacity(edge_id)
                    } else {
                        self.st.flow[edge_id.index()]
                    };

                    // search last blocking arc
                    if delta <= mini_delta {
                        (leaving_edge_id, mini_delta, t2_now_root, t2_new_root) = (edge_id, delta, v, to);
                    }
                    v = self.st.parent[v.index()];
                }
            }
            u
        };

        (leaving_edge_id, apex, mini_delta, t2_now_root, t2_new_root)
    }

    fn pivot(&mut self, leaving_edge_id: EdgeId, entering_edge_id: EdgeId, t2_now_root: NodeId, t2_new_root: NodeId) {
        if leaving_edge_id == entering_edge_id {
            self.st.state[entering_edge_id.index()] = match self.st.state[entering_edge_id.index()] {
                EdgeState::Upper => EdgeState::Lower,
                EdgeState::Lower => EdgeState::Upper,
                _ => panic!("state of entering edge {} is invalid.", entering_edge_id.index()),
            };
            return;
        }

        // drop leaving edge and detach tree
        self.st.detach_tree(self.st.root, t2_now_root, leaving_edge_id);

        // if the size of subtree t2 is larger than that of subtree t1, swap t1 and t2.
        let (t1_new_root, t2_new_root, t2_now_root, new_attach_node) =
            if self.st.num_successors[t2_now_root.index()] * 2 >= self.st.num_nodes {
                (t2_now_root, self.st.opposite_side(t2_new_root, entering_edge_id), self.st.root, t2_new_root)
            } else {
                (self.st.root, t2_new_root, t2_now_root, self.st.opposite_side(t2_new_root, entering_edge_id))
            };

        // enter entering edge and attach tree
        self.st.re_rooting(t2_now_root, t2_new_root, entering_edge_id);
        self.st
            .attach_tree(t1_new_root, new_attach_node, t2_new_root, entering_edge_id);
        self.st.root = t1_new_root;
        assert_eq!(self.st.parent[self.st.root.index()], INVALID_NODE_ID);
    }

    fn flow(&self, edge_id: EdgeId) -> Option<F> {
        self.st.flow_original_graph(edge_id)
    }

    fn flows(&self) -> Vec<F> {
        self.st.flows_original_graph()
    }

    fn potential(&self, node_id: NodeId) -> Option<F> {
        self.st.potential_original_graph(node_id)
    }

    fn potentials(&self) -> Vec<F> {
        self.st.potentials_original_graph()
    }
}

impl_minimum_cost_flow_solver!(PrimalNetworkSimplex, run);
