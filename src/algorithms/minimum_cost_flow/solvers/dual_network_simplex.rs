use crate::{
    algorithms::minimum_cost_flow::{
        edge::MinimumCostFlowEdge,
        node::MinimumCostFlowNode,
        normalized_network::NormalizedNetwork,
        residual_network::construct_extend_network_one_supply_one_demand,
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
use std::collections::VecDeque;

pub struct DualNetworkSimplex<F, P = BlockSearchPivotRule<F>> {
    st: SpanningTreeStructure<F>,
    sink: NodeId,
    pivot: P,
}

impl<F, P> DualNetworkSimplex<F, P>
where
    F: CostNum,
    P: PivotRule<F> + Default,
{
    pub fn set_pivot<Q>(self, new_pivot: Q) -> DualNetworkSimplex<F, Q>
    where
        Q: PivotRule<F>,
    {
        DualNetworkSimplex { st: self.st, sink: self.sink, pivot: new_pivot }
    }

    fn new(graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Self {
        let nn = NormalizedNetwork::new(graph);

        let (source, sink, artificial_edges, fix_excesses) = construct_extend_network_one_supply_one_demand(&nn);
        let mut st =
            SpanningTreeStructure::new(&nn, Some(&[source, sink]), Some(&artificial_edges), None, Some(&fix_excesses));
        st.root = source;

        let mut pivot = P::default();
        pivot.initialize(st.num_edges);

        Self { st, sink, pivot }
    }

    fn run(&mut self) -> Result<F, Status> {
        validate_balance_spanning_tree(&self.st)?;
        validate_infeasible_spanning_tree(&self.st)?;

        if !self.make_initial_spanning_tree_structure() {
            // there is no s-t path
            let status = if self.st.satisfy_constraints() {
                Status::Optimal
            } else {
                Status::Infeasible
            };

            return if status == Status::Optimal {
                Ok(self.st.calculate_objective_value_in_original_graph())
            } else {
                Err(status)
            };
        }

        // debug_assert!(self.st.satisfy_optimality_conditions());

        self.pivot.initialize(self.st.num_edges);
        self.run2();

        self.pivot.clear();

        if !self.st.satisfy_constraints() {
            return Err(Status::Infeasible);
        }
        Ok(self.st.calculate_objective_value_in_original_graph())
    }

    fn run2(&mut self) {
        while let Some(leaving_edge_id) = self.pivot.find_entering_edge(&self.st, Self::calculate_violation) {
            let t2_now_root = if self.st.parent[self.st.from[leaving_edge_id.index()].index()]
                == self.st.to[leaving_edge_id.index()]
            {
                self.st.from[leaving_edge_id.index()]
            } else {
                self.st.to[leaving_edge_id.index()]
            };

            if let Some((entering_edge_id, t2_new_root)) = self.select_entering_edge_id(leaving_edge_id, t2_now_root) {
                let delta = Self::calculate_violation(leaving_edge_id, &self.st);
                let apex = self.find_apex(entering_edge_id);

                // update flow
                self.st.update_flow_in_cycle(entering_edge_id, delta, apex);
                assert!(self.st.is_lower(leaving_edge_id) || self.st.is_upper(leaving_edge_id));

                self.dual_pivot(leaving_edge_id, entering_edge_id, t2_now_root, t2_new_root);
                debug_assert!(self.st.validate_num_successors(self.st.root));
                debug_assert!(self.st.satisfy_optimality_conditions());
            } else {
                break;
            }
        }
    }

    fn calculate_violation(edge_id: EdgeId, st: &SpanningTreeStructure<F>) -> F {
        if st.flow[edge_id.index()] < F::zero() {
            -st.flow[edge_id.index()]
        } else if st.flow[edge_id.index()] > st.upper[edge_id.index()] {
            st.flow[edge_id.index()] - st.upper[edge_id.index()]
        } else {
            F::zero()
        }
    }

    // T: shortest path
    // L: A \ T
    // U: empty
    fn make_initial_spanning_tree_structure(&mut self) -> bool {
        let (distances, prev_edge_id) = self.st.shortest_path(self.st.root);

        // there is no s-t path
        if prev_edge_id[self.sink.index()].is_none() {
            return false;
        }

        // make tree structure
        let mut children = vec![Vec::new(); self.st.num_nodes];
        for edge_id in prev_edge_id.iter().filter_map(|&edge_id| edge_id) {
            self.st.state[edge_id.index()] = EdgeState::Tree;
            (
                self.st.parent[self.st.to[edge_id.index()].index()],
                self.st.parent_edge_id[self.st.to[edge_id.index()].index()],
            ) = (self.st.from[edge_id.index()], edge_id);
            children[self.st.from[edge_id.index()].index()].push(self.st.to[edge_id.index()]);
        }
        (self.st.parent[self.st.root.index()], self.st.parent_edge_id[self.st.root.index()]) =
            (INVALID_NODE_ID, INVALID_EDGE_ID);
        self.st.last_descendent_dft = (0..self.st.num_nodes).map(NodeId).collect();

        let mut prev_node = INVALID_NODE_ID;
        let mut stack = VecDeque::from([(self.st.root, INVALID_NODE_ID)]);
        let mut seen = vec![false; self.st.num_nodes];
        while let Some((u, parent)) = stack.pop_back() {
            if seen[u.index()] {
                self.st.num_successors[u.index()] += 1;
                if parent != INVALID_NODE_ID {
                    self.st.last_descendent_dft[parent.index()] = self.st.last_descendent_dft[u.index()];
                    self.st.num_successors[self.st.parent[u.index()].index()] += self.st.num_successors[u.index()];
                }
                continue;
            }

            seen[u.index()] = true;
            self.st.prev_node_dft[u.index()] = prev_node;
            if prev_node != INVALID_NODE_ID {
                self.st.next_node_dft[prev_node.index()] = u;
            }
            prev_node = u;
            stack.push_back((u, parent));
            for &child in children[u.index()].iter().rev() {
                stack.push_back((child, u));
            }
        }
        self.st.next_node_dft[prev_node.index()] = self.st.root;

        // determine potentials
        for u in 0..self.st.num_nodes {
            self.st.potentials[u] = -distances[u];
        }

        // send flow from source to sink
        self.st
            .update_flow_in_path(self.st.root, self.sink, self.st.excesses[self.st.root.index()]);
        assert!(
            self.st.excesses[self.st.root.index()] == F::zero() && self.st.excesses[self.sink.index()] == F::zero()
        );

        true
    }

    fn select_entering_edge_id(&self, leaving_edge_id: EdgeId, t2_now_root: NodeId) -> Option<(EdgeId, NodeId)> {
        let mut is_t1_node = vec![false; self.st.num_nodes];
        let mut now = self.st.root;
        loop {
            is_t1_node[now.index()] = true;
            now = self.st.next_node_dft[now.index()];
            if now == t2_now_root {
                now = self.st.next_node_dft[self.st.last_descendent_dft[now.index()].index()];
            }
            if now == self.st.root {
                break;
            }
        }

        let flow_direction_t1_t2 = |edge_id: EdgeId| {
            // (t1 -> t2 and lower) or (t2 -> t1 and upper)
            (is_t1_node[self.st.from[edge_id.index()].index()]
                && !is_t1_node[self.st.to[edge_id.index()].index()]
                && self.st.flow[edge_id.index()] <= F::zero())
                || (!is_t1_node[self.st.from[edge_id.index()].index()]
                    && is_t1_node[self.st.to[edge_id.index()].index()]
                    && self.st.flow[edge_id.index()] >= self.st.upper[edge_id.index()])
        };

        let leaving_edge_flow_direction = flow_direction_t1_t2(leaving_edge_id);

        let mut entering_edge_id = None;
        let mut t2_new_root = None;
        let mut mini_delta = F::zero();
        for edge_id in (0..self.st.num_edges).map(EdgeId) {
            if self.st.state[edge_id.index()] == EdgeState::Tree || self.st.upper[edge_id.index()] == F::zero() {
                continue;
            }

            let entering_edge_flow_direction = flow_direction_t1_t2(edge_id);
            if leaving_edge_flow_direction == entering_edge_flow_direction
                || is_t1_node[self.st.from[edge_id.index()].index()] == is_t1_node[self.st.to[edge_id.index()].index()]
            {
                continue;
            }

            let reduced_cost = if self.st.state[edge_id.index()] == EdgeState::Lower {
                self.st.reduced_cost(edge_id)
            } else {
                -self.st.reduced_cost(edge_id)
            };
            assert!(reduced_cost >= F::zero());

            if reduced_cost < mini_delta || entering_edge_id.is_none() {
                mini_delta = reduced_cost;
                entering_edge_id = Some(edge_id);
                t2_new_root = if (entering_edge_flow_direction && self.st.state[edge_id.index()] == EdgeState::Lower)
                    || (!entering_edge_flow_direction && self.st.state[edge_id.index()] == EdgeState::Upper)
                {
                    Some(self.st.to[edge_id.index()])
                } else {
                    Some(self.st.from[edge_id.index()])
                };
            }
        }

        Some((entering_edge_id?, t2_new_root?))
    }

    fn dual_pivot(
        &mut self,
        leaving_edge_id: EdgeId,
        entering_edge_id: EdgeId,
        t2_now_root: NodeId,
        t2_new_root: NodeId,
    ) {
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
        self.st.parent[self.st.root.index()] = INVALID_NODE_ID;
        assert_eq!(self.st.parent_edge_id[self.st.root.index()], INVALID_EDGE_ID);
    }

    fn find_apex(&self, entering_edge_id: EdgeId) -> NodeId {
        let (mut u, mut v) = (self.st.from[entering_edge_id.index()], self.st.to[entering_edge_id.index()]);
        while u != v {
            let (u_num, v_num) = (self.st.num_successors[u.index()], self.st.num_successors[v.index()]);
            if u_num <= v_num {
                u = self.st.parent[u.index()];
            }
            if v_num <= u_num {
                v = self.st.parent[v.index()];
            }
        }
        u
    }

    fn make_minimum_cost_flow_in_original_graph(&self) -> Vec<F> {
        self.st.make_minimum_cost_flow_in_original_graph()
    }

    fn flow(&self, edge_id: EdgeId) -> Option<F> {
        self.st.flow(edge_id)
    }

    fn flows(&self) -> Vec<F> {
        self.st.flows()
    }

    fn potential(&self, node_id: NodeId) -> Option<F> {
        self.st.potential(node_id)
    }

    fn potentials(&self) -> Vec<F> {
        self.st.potentials()
    }
}

impl_minimum_cost_flow_solver!(DualNetworkSimplex, run);
