// use crate::data_structures::rollback_union_find::RollbackUnionFind;
// use crate::data_structures::UnionFind;
// use crate::data_structures::skew_heap::SkewHeap;
// use crate::edge::weight::WeightEdge;
// use crate::prelude::{Directed, EdgeId, Graph};
// use crate::traits::{Bounded, IntNum, Zero};
// use std::marker::PhantomData;
// 
// #[derive(Default)]
// pub struct Tarjan<W> {
//     phantom_data: PhantomData<W>,
// }
// 
// impl<W> Tarjan<W>
// where
//     W: IntNum + Zero + Bounded + std::ops::Neg<Output = W> + Default,
// {
//     pub fn solve(&mut self, graph: &Graph<Directed, (), WeightEdge<W>>, root: usize) -> Option<(W, Vec<EdgeId>)> {
//         struct Edge {
//             id: EdgeId,
//             from: usize,
//             to: usize,
//         }
// 
//         let num_nodes = graph.num_nodes();
//         let mut skew_heap = SkewHeap::<W, Edge>::with_capacity(graph.num_edges());
//         let mut heap_node_id = vec![None; num_nodes];
// 
//         for (idx, edge) in graph.edges.iter().enumerate() {
//             let id = skew_heap.add_node(edge.data.weight, Edge {id: EdgeId(idx), from: edge.u.index(), to: edge.v.index()});
//             heap_node_id[edge.v.index()] = skew_heap.merge(heap_node_id[edge.v.index()], Some(id));
//         }
// 
//         let mut total_cost = W::zero();
//         let mut edge = vec![None::<usize>; num_nodes];
//         let mut cycles = Vec::<(usize, usize)>::new();
//         let mut uf_wcc = UnionFind::new(num_nodes);
//         let mut uf_scc = RollbackUnionFind::new(num_nodes);
// 
//         for u in 0..num_nodes {
//             if u == root {
//                 continue;
//             }
//             let mut now = u;
//             loop {
//                 let mini_edge = heap_node_id[now]?; // 到達不能
//                 edge[now] = Some(mini_edge);
//                 let mini_cost = skew_heap.get_node(mini_edge).key;
//                 total_cost += mini_cost;
//                 skew_heap.add(mini_edge, mini_cost);
// 
//                 let from = uf_scc.find(skew_heap.get_node(mini_edge).val.from);
//                 if uf_wcc.unite(now, from) {
//                     break;
//                 }
// 
//                 // contract cycle
//                 let time_stamp = uf_scc.time();
//                 let mut nxt = from;
//                 while uf_scc.join(now, nxt) {
//                     let rep = uf_scc.find(now);
//                     heap_node_id[rep] = skew_heap.merge(heap_node_id[now], heap_node_id[nxt]);
//                     now = rep;
//                     nxt = uf_scc.find(skew_heap.get_node(edge[nxt].unwrap()).val.from);
//                 }
//                 
//                 cycles.push((edge[now].unwrap(), time_stamp));
// 
//                 // remove self-loops
//                 loop {
//                     let idx = match heap_node_id[now] {
//                         Some(x) => x,
//                         None => break,
//                     };
//                     if uf_scc.same(skew_heap.get_node(idx).val.from, now) {
//                         skew_heap.pop(&mut heap_node_id[now]);
//                     } else {
//                         break;
//                     }
//                 }
//             }
//         }
// 
//         // expand cycles
//         for &(last_edge, t) in cycles.iter().rev() {
//             //
//             let a = skew_heap.get_node(last_edge).val.to;   // サイクル最後の辺が入る頂点
//             let vr = uf_scc.find(a);                        // サイクル最後の辺が入る超頂点
// 
//             let e = edge[vr].unwrap(); //サイクル外部から入る辺
//             let v = skew_heap.get_node(e).val.to;
// 
//             uf_scc.rollback(t);
//             let vin = uf_scc.find(v);   // 現在の超頂点
// 
//             // let old = std::mem::replace(&mut edge[vr], Some(last_edge));
//             // edge[vin] = old;    // edge[vr]がはいる
//             let back = edge[vr];
//             edge[vr] = Some(last_edge);
//             edge[vin] = back;
//         }
// 
//         let mut arborescence = Vec::with_capacity(num_nodes - 1);
//         for u in 0..num_nodes {
//             if u != root {
//                 arborescence.push(skew_heap.get_node(edge[u].unwrap()).val.id);
//             }
//         }
//         Some((total_cost, arborescence))
//     }
// }
