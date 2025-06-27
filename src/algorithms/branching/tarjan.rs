//! Tarjan’s optimum branching algorithm
//!
//! # References
//! [1] R. E. Tarjan, “Finding optimum branchings,” Networks, vol. 7, no. 1, pp. 25–35, Mar. 1977, doi: 10.1002/net.3230070103.
//! [2] P. M. Camerini, L. Fratta, and F. Maffioli, “A note on finding optimum branchings,” Networks, vol. 9, no. 4, pp. 309–312, Dec. 1979, doi: 10.1002/net.3230090403.
//! [3] A. Tofigh, “Optimum Branchings and Spanning Aborescences”, PDF: https://cw.fel.cvut.cz/old/_media/courses/a4m33pal/cviceni/algorithm-description.pdf

use crate::data_structures::bit_vector::BitVector;
use crate::data_structures::skew_heap::SkewHeap;
use crate::data_structures::UnionFind;
use crate::edge::weight::WeightEdge;
use crate::prelude::{Directed, EdgeId, Graph};
use crate::traits::{Bounded, IntNum, Zero};
use std::marker::PhantomData;
use std::mem;

#[derive(Clone, Default)]
struct Edge {
    id: EdgeId,
    from: usize,
}

struct Forest {
    parent: Box<[usize]>,
    children: Box<[Vec<usize>]>,
    is_root: BitVector,
    lambda: Box<[usize]>, // Leaf edge in Forest headed by v; usize::MAX if none.
}

impl Forest {
    #[inline]
    fn add_child(&mut self, parent: usize, child: usize) {
        self.parent[child] = parent;
        self.children[parent].push(child);
        self.is_root.set(child, false);
    }

    fn delete_path(&mut self, u: usize) -> Vec<usize> {
        let mut new_root = Vec::new();

        let mut edge_id = self.lambda[u];
        let mut pre_edge_id = usize::MAX;
        while edge_id != usize::MAX {
            self.is_root.set(edge_id, false);
            for &child_edge_id in self.children[edge_id].iter() {
                if child_edge_id != pre_edge_id {
                    self.parent[child_edge_id] = usize::MAX;
                    self.is_root.set(child_edge_id, true);
                    new_root.push(child_edge_id);
                }
            }
            pre_edge_id = edge_id;
            edge_id = self.parent[edge_id];
        }

        new_root
    }
}

#[derive(Default)]
pub struct Tarjan<W> {
    _marker: PhantomData<W>,
}

impl<W> Tarjan<W>
where
    W: IntNum + Zero + Bounded + std::ops::Neg<Output = W> + Default,
{
    pub fn solve(&mut self, graph: &Graph<Directed, (), WeightEdge<W>>) -> (W, Vec<EdgeId>) {
        let (branching_roots, forest) = self.construct_forest(graph);
        self.construct_branching(branching_roots, forest, graph)
    }

    fn construct_forest(&self, graph: &Graph<Directed, (), WeightEdge<W>>) -> (Vec<usize>, Forest) {
        let num_nodes = graph.num_nodes();

        let mut uf_wcc = UnionFind::new(num_nodes);
        let mut uf_scc = UnionFind::new(num_nodes);
        let mut enter = vec![(usize::MAX, W::max_value(), EdgeId(usize::MAX)); num_nodes];
        let mut enter_edges = vec![SkewHeap::<W, Edge>::default(); num_nodes]; // in_edges[v] = all incoming edges of v
        let mut rset = Vec::new();
        let mut min: Vec<usize> = (0..num_nodes).collect();
        let mut cycles: Vec<Vec<EdgeId>> = vec![Vec::new(); num_nodes];

        let mut forest = Forest {
            parent: vec![usize::MAX; graph.num_edges()].into_boxed_slice(),
            children: vec![Vec::new(); graph.num_edges()].into_boxed_slice(),
            is_root: BitVector::new(graph.num_edges()),
            lambda: vec![usize::MAX; num_nodes].into_boxed_slice(),
        };

        for (idx, edge) in graph.edges.iter().enumerate() {
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
            forest.is_root.set(edge.id.index(), true);

            if cycles[v].is_empty() {
                forest.lambda[v] = edge.id.index();
            }

            for cycle_edge_id in cycles[v].drain(..) {
                forest.add_child(edge.id.index(), cycle_edge_id.index());
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
            min[scc] = min[minimum_weight_incoming_node];
            roots.push(scc);
        }

        let branching_roots: Vec<usize> = rset.iter().map(|r| min[*r]).collect();
        (branching_roots, forest)
    }

    fn construct_branching(&self, branching_roots: Vec<usize>, mut forest: Forest, graph: &Graph<Directed, (), WeightEdge<W>>) -> (W, Vec<EdgeId>) {
        for r in branching_roots {
            forest.delete_path(r);
        }

        let mut total_cost = W::zero();
        let mut branchings = Vec::with_capacity(graph.num_nodes() - 1);

        let mut forest_roots: Vec<usize> = (0..graph.num_edges()).filter(|&e| forest.is_root.get(e)).collect();
        while let Some(edge_id) = forest_roots.pop() {
            branchings.push(EdgeId(edge_id));
            total_cost += graph.edges[edge_id].data.weight;

            let v = graph.edges[edge_id].v.index();
            let new_roots = forest.delete_path(v);
            forest_roots.extend(new_roots);
        }

        (total_cost, branchings)
    }
}
