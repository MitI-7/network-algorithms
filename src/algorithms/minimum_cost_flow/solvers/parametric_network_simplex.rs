use crate::{
    algorithms::minimum_cost_flow::{
        edge::MinimumCostFlowEdge,
        extend_network::construct_extend_network_one_supply_one_demand,
        node::MinimumCostFlowNode,
        normalized_network::NormalizedNetwork,
        solvers::{macros::impl_minimum_cost_flow_solver, solver::MinimumCostFlowSolver},
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

pub struct ParametricNetworkSimplex<F> {
    st: SpanningTreeStructure<F>,
    sink: NodeId,
}

impl<F> ParametricNetworkSimplex<F>
where
    F: CostNum,
{
    fn new(graph: &Graph<Directed, MinimumCostFlowNode<F>, MinimumCostFlowEdge<F>>) -> Self {
        let nn = NormalizedNetwork::new(&graph);

        let (source, sink, artificial_edges, fix_excesses) = construct_extend_network_one_supply_one_demand(&nn);
        let mut st =
            SpanningTreeStructure::new(&nn, Some(&[source, sink]), Some(&artificial_edges), None, Some(&fix_excesses));
        st.root = source;

        Self { st, sink }
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
            // graph.remove_artificial_sub_graph(&[source, sink], &artificial_edges);

            return if status == Status::Optimal {
                Ok(self.st.calculate_objective_value_in_original_graph())
            } else {
                Err(status)
            };
        }
        debug_assert!(self.st.satisfy_optimality_conditions());

        self.run2();

        if !self.st.satisfy_constraints() {
            return Err(Status::Infeasible);
        }

        Ok(self.st.calculate_objective_value_in_original_graph())
    }

    pub(crate) fn run2(&mut self) {
        while let Some((leaving_edge_id, delta)) = self.select_leaving_edge() {
            let t2_now_root = if self.st.parent[self.st.from[leaving_edge_id.index()].index()]
                == self.st.to[leaving_edge_id.index()]
            {
                self.st.from[leaving_edge_id.index()]
            } else {
                self.st.to[leaving_edge_id.index()]
            };

            self.st.update_flow_in_path(self.st.root, self.sink, delta);
            if self.st.excesses[self.st.root.index()] == F::zero() {
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
        if prev_edge_id[self.sink.index()].is_none() {
            return false;
        }

        // make tree structure
        let mut children = vec![Vec::new(); self.st.num_nodes];
        for edge_id in prev_edge_id.iter().filter_map(|&edge_id| edge_id) {
            self.st.state[edge_id.index()] = EdgeState::Tree;
            let (from, to) = (self.st.from[edge_id.index()], self.st.to[edge_id.index()]);
            (self.st.parent[to.index()], self.st.parent_edge_id[to.index()]) = (from, edge_id);
            children[from.index()].push(to);
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

        true
    }

    fn select_leaving_edge(&self) -> Option<(EdgeId, F)> {
        let mut leaving_edge_id = None;
        let mut mini_delta = F::zero();
        let mut now = self.sink;
        while now != self.st.root {
            let (parent, edge_id) = (self.st.parent[now.index()], self.st.parent_edge_id[now.index()]);
            assert_eq!(self.st.state[edge_id.index()], EdgeState::Tree);

            let delta = if self.st.from[edge_id.index()] == parent {
                self.st.residual_capacity(edge_id)
            } else {
                self.st.flow[edge_id.index()]
            };
            // select the edge closest to the source as the leaving edge
            if leaving_edge_id.is_none() || delta <= mini_delta {
                mini_delta = delta;
                leaving_edge_id = Some(edge_id);
            }

            now = parent;
        }
        Some((leaving_edge_id?, mini_delta.min(self.st.excesses[self.st.root.index()])))
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

        let mut entering_edge_id = None;
        let mut t2_new_root = None;
        let mut mini_delta = F::zero();
        for edge_id in (0..self.st.num_edges).map(EdgeId) {
            if edge_id == leaving_edge_id {
                continue;
            }

            // t1 -> t2 and lower
            let (from, to) = (self.st.from[edge_id.index()], self.st.to[edge_id.index()]);
            if is_t1_node[from.index()]
                && !is_t1_node[to.index()]
                && self.st.state[edge_id.index()] == EdgeState::Lower
                && self.st.upper[edge_id.index()] != F::zero()
            {
                let reduced_cost = self.st.reduced_cost(edge_id);
                if reduced_cost < mini_delta || entering_edge_id.is_none() {
                    mini_delta = reduced_cost;
                    entering_edge_id = Some(edge_id);
                    t2_new_root = Some(to);
                }
            }

            // t2 -> t1 and upper
            if !is_t1_node[from.index()] && is_t1_node[to.index()] && self.st.state[edge_id.index()] == EdgeState::Upper
            {
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

        // drop leaving edge
        self.st.detach_tree(self.st.root, t2_now_root, leaving_edge_id);

        // enter entering edge
        let attach_node = self.st.opposite_side(t2_new_root, entering_edge_id);
        self.st.re_rooting(t2_now_root, t2_new_root, entering_edge_id);
        self.st
            .attach_tree(self.st.root, attach_node, t2_new_root, entering_edge_id);
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

impl_minimum_cost_flow_solver!(ParametricNetworkSimplex, run);
