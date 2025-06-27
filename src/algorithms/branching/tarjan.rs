use crate::data_structures::skew_heap::SkewHeap;
use crate::data_structures::UnionFind;
use crate::edge::weight::WeightEdge;
use crate::prelude::{Directed, EdgeId, Graph};
use crate::traits::{Bounded, IntNum, Zero};
use std::marker::PhantomData;
use std::mem;

#[derive(Default)]
pub struct Tarjan<W> {
    phantom_data: PhantomData<W>,
}

impl<W> Tarjan<W>
where
    W: IntNum + Zero + Bounded + std::ops::Neg<Output = W> + Default,
{
    pub fn solve(&mut self, graph: &Graph<Directed, (), WeightEdge<W>>) -> (W, Vec<EdgeId>) {
        #[derive(Clone, Debug, Default)]
        struct Edge {
            id: EdgeId,
            from: usize,
        }

        let num_nodes = graph.num_nodes();

        let mut uf_wcc = UnionFind::new(num_nodes);
        let mut uf_scc = UnionFind::new(num_nodes);
        let mut enter = vec![(usize::MAX, W::max_value(), EdgeId(usize::MAX)); num_nodes];
        let mut enter_edges = vec![SkewHeap::<W, Edge>::default(); num_nodes]; // in_edges[v] = all incoming edges of v
        let mut rset = Vec::new();
        let mut min: Vec<usize> = (0..num_nodes).collect();
        let mut cycles: Vec<Vec<EdgeId>> = vec![Vec::new(); num_nodes];

        // forest F
        let mut lambda = vec![usize::MAX; num_nodes];
        let mut parent = vec![usize::MAX; graph.num_edges()];
        let mut children = vec![Vec::new(); graph.num_edges()];
        let mut is_root_in_f = vec![false; graph.num_edges()];

        for (idx, edge) in graph.edges.iter().enumerate() {
            enter_edges[edge.v.index()].push(edge.data.weight, Edge { id: EdgeId(idx), from: edge.u.index() });
        }

        let mut roots: Vec<usize> = (0..num_nodes).collect(); // array of root components
        while let Some(k) = roots.pop() {
            let v = uf_scc.find(k);
            let (maximum_weight, edge) = enter_edges[v].pop().unwrap_or((W::zero(), Edge::default()));

            // no positive weight incoming edge of v
            if maximum_weight <= W::zero() {
                rset.push(v);
                continue;
            }

            let u = uf_scc.find(edge.from);

            // u and v are in the same scc
            if uf_scc.same(u, v) {
                roots.push(k);
                continue;
            }

            enter[v] = (u, maximum_weight, edge.id);
            is_root_in_f[edge.id.index()] = true;

            if cycles[v].is_empty() {
                lambda[v] = edge.id.index();
            }

            for cycle_edge_id in cycles[v].drain(..) {
                parent[cycle_edge_id.index()] = edge.id.index();
                children[edge.id.index()].push(cycle_edge_id.index());
                is_root_in_f[cycle_edge_id.index()] = false;
            }

            // u and v are not in the same wcc
            if uf_wcc.union(u, v) {
                continue;
            }

            // contract cycle
            let mut cycle_edges = vec![edge.id];
            let mut cycle_nodes = vec![(v, enter[v].1)];
            let mut minimum_weight_in_cycle = maximum_weight;
            let mut vertex = uf_scc.find(v);
            let mut now = uf_scc.find(u);
            while now != v {
                let (prev, weight, edge_id) = enter[now];
                cycle_nodes.push((now, weight));
                cycle_edges.push(edge_id);

                if weight < minimum_weight_in_cycle {
                    minimum_weight_in_cycle = weight;
                    vertex = now;
                }
                now = uf_scc.find(prev);
            }

            let mut scc = v;
            let mut stock = SkewHeap::<W, Edge>::default();
            for (now, weight) in cycle_nodes {
                // adjust weight
                enter_edges[now].add_all(minimum_weight_in_cycle - weight);
                debug_assert!(weight != W::max_value());

                // contraction
                uf_scc.union(scc, now);
                uf_wcc.union(scc, now);

                stock.merge_with(mem::take(&mut enter_edges[now]));

                scc = uf_scc.find(scc);
            }
            enter_edges[scc].merge_with(stock);
            cycles[scc] = cycle_edges;
            min[scc] = min[vertex];
            roots.push(scc);
        }

        // construct branching
        for root in rset {
            self.delete_path(lambda[min[root]], &mut parent, &children, &mut is_root_in_f);
        }

        let mut stack = Vec::new();
        for edge_id in 0..graph.num_edges() {
            if is_root_in_f[edge_id] {
                stack.push(edge_id);
            }
        }

        let mut total_cost = W::zero();
        let mut branchings = Vec::with_capacity(num_nodes - 1);

        while let Some(edge_id) = stack.pop() {
            branchings.push(EdgeId(edge_id));
            total_cost += graph.edges[edge_id].data.weight;

            let v = graph.edges[edge_id].v.index();
            let ve = self.delete_path(lambda[v], &mut parent, &children, &mut is_root_in_f);
            stack.extend(ve);
        }

        (total_cost, branchings)
    }

    fn delete_path(&self, mut edge_id: usize, parent: &mut Vec<usize>, children: &Vec<Vec<usize>>,  is_root_in_f: &mut Vec<bool>) -> Vec<usize> {
        let mut new_root = Vec::new();
        let mut pre_edge_id = usize::MAX;
        while edge_id != usize::MAX {
            is_root_in_f[edge_id] = false;
            for &child_edge_id in children[edge_id].iter() {
                if child_edge_id != pre_edge_id {
                    parent[child_edge_id] = usize::MAX;
                    is_root_in_f[child_edge_id] = true;
                    new_root.push(child_edge_id);
                }
            }
            pre_edge_id = edge_id;
            edge_id = parent[edge_id];
        }

        new_root
    }
}
