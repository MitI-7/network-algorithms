#[derive(PartialEq, Clone, Debug)]
pub struct Edge {
    pub u: usize,
    pub v: usize,
}

#[derive(Default)]
pub struct Graph {
    num_nodes: usize,
    num_edges: usize,
    pub(crate) edges: Vec<Edge>,
}

impl Graph {
    #[inline]
    pub fn num_nodes(&self) -> usize {
        self.num_nodes
    }

    #[inline]
    pub fn num_edges(&self) -> usize {
        self.num_edges
    }

    pub fn add_node(&mut self) -> usize {
        self.num_nodes += 1;
        self.num_nodes - 1
    }

    pub fn add_nodes(&mut self, num_nodes: usize) -> Vec<usize> {
        self.num_nodes += num_nodes;
        ((self.num_nodes - num_nodes)..self.num_nodes).collect()
    }

    // return edge index
    pub fn add_undirected_edge(&mut self, u: usize, v: usize) -> Option<usize> {
        if u >= self.num_nodes || v >= self.num_nodes {
            return None;
        }
        self.edges.push(Edge { u, v });
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
}
