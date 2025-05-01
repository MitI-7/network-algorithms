use std::fmt::Debug;

#[derive(PartialEq, Clone, Debug)]
pub struct Edge {
    pub u: usize,
    pub v: usize,
}

#[derive(Default)]
pub struct BipartiteGraph {
    num_left_nodes: usize,
    num_right_nodes: usize,
    num_edges: usize,
    pub(crate) edges: Vec<Edge>,
}

impl BipartiteGraph {
    #[inline]
    pub fn num_left_nodes(&self) -> usize {
        self.num_left_nodes
    }

    pub fn num_right_nodes(&self) -> usize {
        self.num_right_nodes
    }

    #[inline]
    pub fn num_edges(&self) -> usize {
        self.num_edges
    }

    pub fn add_left_node(&mut self) -> usize {
        self.num_left_nodes += 1;
        self.num_left_nodes + self.num_right_nodes - 1
    }

    pub fn add_right_node(&mut self) -> usize {
        self.num_right_nodes += 1;
        self.num_right_nodes + self.num_left_nodes - 1
    }

    pub fn add_left_nodes(&mut self, num_nodes: usize) -> Vec<usize> {
        self.num_left_nodes += num_nodes;
        ((self.num_left_nodes + self.num_right_nodes - num_nodes)..self.num_left_nodes + self.num_right_nodes).collect()
    }

    pub fn add_right_nodes(&mut self, num_nodes: usize) -> Vec<usize> {
        self.num_right_nodes += num_nodes;
        ((self.num_left_nodes + self.num_right_nodes - num_nodes)..self.num_left_nodes + self.num_right_nodes).collect()
    }

    // return edge index
    pub fn add_undirected_edge(&mut self, left_node: usize, right_node: usize) -> Option<usize> {
        // if left_node >= self.num_left_nodes || right_node >= self.num_right_nodes {
        //     return None;
        // }

        self.edges.push(Edge { u: left_node, v: right_node });

        self.num_edges += 1;
        Some(self.num_edges - 1)
    }

    pub fn get_edge(&self, edge_id: usize) -> Option<Edge> {
        if edge_id >= self.edges.len() {
            return None;
        }
        let edge = &self.edges[edge_id];
        Some(Edge { u: edge.u, v: edge.v })
    }

    pub fn pop_edge(&mut self) {
        self.edges.pop();
        self.num_edges -= 1;
    }
}
