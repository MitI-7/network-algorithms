//! Algorithms for finding optimum branchings
//! Based on the following works:
//! [1] R. E. Tarjan, “Finding optimum branchings,” Networks, vol. 7, no. 1, pp. 25–35, Mar. 1977, doi: 10.1002/net.3230070103.
//! [2] P. M. Camerini, L. Fratta, and F. Maffioli, “A note on finding optimum branchings,” Networks, vol. 9, no. 4, pp. 309–312, Dec. 1979, doi: 10.1002/net.3230090403.
//! [3] A. Tofigh, “Optimum Branchings and Spanning Aborescences”, PDF: https://cw.fel.cvut.cz/old/_media/courses/a4m33pal/cviceni/algorithm-description.pdf

use std::fmt::Debug;
use crate::data_structures::bit_vector::BitVector;
use crate::data_structures::skew_heap::SkewHeap;
use crate::data_structures::UnionFind;
use crate::algorithms::branching::edge::WeightEdge;
use crate::graph::direction::{Directed};
use crate::graph::graph::Graph;
use std::marker::PhantomData;
use std::mem;
use crate::core::numeric::FlowNum;
use crate::graph::ids::EdgeId;

struct Forest {
    parent: Box<[Option<usize>]>,
    lambda: Box<[Option<usize>]>, // leaf edge in Forest headed by v
    order: Vec<usize>,
    deleted: BitVector,
}

impl Forest {
    pub fn new(num_nodes: usize, num_edges: usize) -> Self {
        Self {
            parent: vec![None; num_edges].into_boxed_slice(),
            lambda: vec![None; num_nodes].into_boxed_slice(),
            order: Vec::with_capacity(num_edges),
            deleted: BitVector::new(num_edges),
        }
    }

    #[inline]
    fn add_child(&mut self, parent: usize, child: usize) {
        self.parent[child] = Some(parent);
    }

    #[inline]
    fn delete_path(&mut self, u: usize) {
        let mut edge_id_opt = self.lambda[u];
        while let Some(edge_id) = edge_id_opt {
            if self.deleted.get(edge_id) {
                break;
            }
            self.deleted.set(edge_id, true);
            edge_id_opt = self.parent[edge_id];
        }
    }
}

#[derive(Default)]
pub struct Tarjan<W> {
    _marker: PhantomData<W>,
}

impl<W> Tarjan<W>
where
    W: FlowNum + Default,
{
    pub fn solve(&mut self, graph: &Graph<Directed, (), WeightEdge<W>>) -> (W, Vec<EdgeId>) {
        let (branching_roots, forest) = self.construct_forest(graph);
        self.construct_branching(branching_roots, forest, graph)
    }

    fn construct_forest(&self, graph: &Graph<Directed, (), WeightEdge<W>>) -> (Vec<usize>, Forest) {
        #[derive(Clone, Default)]
        struct Edge {
            id: EdgeId,
            from: usize,
        }

        let num_nodes = graph.num_nodes();
        let num_edges = graph.num_edges();

        let mut uf_wcc = UnionFind::new(num_nodes);
        let mut uf_scc = UnionFind::new(num_nodes);
        let mut enter = vec![(usize::MAX, W::max_value(), EdgeId(usize::MAX)); num_nodes];
        let mut enter_edges = vec![SkewHeap::<W, Edge>::default(); num_nodes]; // in_edges[v] = all incoming edges of v
        let mut rset = Vec::new();
        let mut min: Vec<usize> = (0..num_nodes).collect();
        let mut cycles: Vec<Vec<EdgeId>> = vec![Vec::new(); num_nodes];

        let mut forest = Forest::new(num_nodes, num_edges);

        for (idx, edge) in graph.edges().enumerate() {
            enter_edges[edge.v.index()].push(edge.data.weight, Edge { id: EdgeId(idx), from: edge.u.index() });
        }

        let mut roots: Vec<usize> = (0..num_nodes).collect(); // array of root components
        while let Some(r) = roots.pop() {
            let v = uf_scc.find(r);

            let (maximum_weight, edge) = match enter_edges[v].pop() {
                Some((maximum_weight, edge)) => (maximum_weight, edge),
                None => {
                    rset.push(v);
                    continue;
                }
            };

            // no positive weight incoming edge of v
            if maximum_weight <= W::zero() {
                rset.push(v);
                continue;
            }

            let u = uf_scc.find(edge.from);

            // u and v are in the same scc
            if uf_scc.same(u, v) {
                roots.push(r);
                continue;
            }

            enter[v] = (u, maximum_weight, edge.id);
            forest.order.push(edge.id.index());

            if cycles[v].is_empty() {
                forest.lambda[v] = Some(edge.id.index());
            }

            for child in cycles[v].drain(..) {
                forest.add_child(edge.id.index(), child.index());
            }

            // u and v are not in the same wcc
            if uf_wcc.union(u, v) {
                continue;
            }

            // contract cycle
            let mut cycle_edges = vec![edge.id];
            let mut cycle_nodes = vec![(v, enter[v].1)];
            let mut minimum_weight_in_cycle = maximum_weight;
            let mut minimum_weight_incoming_node = uf_scc.find(v);
            let mut now = uf_scc.find(u);
            while now != v {
                let (prev, weight, edge_id) = enter[now];
                cycle_nodes.push((now, weight));
                cycle_edges.push(edge_id);

                if weight < minimum_weight_in_cycle {
                    minimum_weight_in_cycle = weight;
                    minimum_weight_incoming_node = now;
                }
                now = uf_scc.find(prev);
            }

            let mut scc = v;
            let mut stock = SkewHeap::<W, Edge>::new();
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
            min[scc] = min[minimum_weight_incoming_node];
            roots.push(scc);
        }

        let branching_roots: Vec<usize> = rset.iter().map(|r| min[*r]).collect();
        (branching_roots, forest)
    }

    fn construct_branching(&self, branching_roots: Vec<usize>, mut forest: Forest, graph: &Graph<Directed, (), WeightEdge<W>>) -> (W, Vec<EdgeId>) {
        let mut total_weight = W::zero();
        let mut branchings = Vec::with_capacity(graph.num_nodes() - 1);

        for r in branching_roots {
            forest.delete_path(r);
        }

        while let Some(edge_id) = forest.order.pop() {
            let edge_id = EdgeId(edge_id);
            if forest.deleted.get(edge_id.index()) {
                continue;
            }
            branchings.push(edge_id);
            total_weight += graph.get_edge(edge_id).unwrap().data.weight;
            let v = graph.get_edge(edge_id).unwrap().v.index();
            forest.delete_path(v);
        }

        (total_weight, branchings)
    }
}
