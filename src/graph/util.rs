use super::disjoint_set::DisjointSets;
use super::graph::{AdjListIterator, DirectedGraph, UndirectedGraph};
use std::cmp::Reverse;
use std::f32::consts::E;

impl DirectedGraph {
    // Helper function used by euler_path. Note that we can't use a for-loop
    // that would consume the adjacency list as recursive calls may need it.
    fn euler_recurse(u: usize, adj: &mut [AdjListIterator], edges: &mut Vec<usize>) {
        while let Some((e, v)) = adj[u].next() {
            Self::euler_recurse(*v, adj, edges);
            edges.push(*e);
        }
    }
    /// Finds the sequence of edges in an Euler path starting from u, assuming
    /// it exists and that the graph is directed. Undefined behavior if this
    /// precondition is violated. To extend this to undirected graphs, maintain
    /// a visited array to skip the reverse edge.
    pub fn euler_path(&self, u: usize) -> Vec<usize> {
        let mut adj_iters = (0..self.num_v())
            .map(|u| self.adj_list(u))
            .collect::<Vec<_>>();
        let mut edges = Vec::with_capacity(self.num_e());
        Self::euler_recurse(u, &mut adj_iters, &mut edges);
        edges.reverse();
        edges
    }

    // Single-source shortest paths on a directed graph with non-negative weights
    pub fn dijkstra(&self, u: usize) -> Vec<u64> {
        let mut dist = vec![u64::max_value(); self.edge_weights.len()];
        let mut heap = std::collections::BinaryHeap::new();

        dist[u] = 0;
        heap.push((Reverse(0), 0));
        while let Some((Reverse(dist_u), u)) = heap.pop() {
            if dist[u] == dist_u {
                for (e, v) in self.adj_list(u) {
                    let dist_v = dist_u + self.edge_weights[*e] as u64;
                    if dist[*v] > dist_v {
                        dist[*v] = dist_v;
                        heap.push((Reverse(dist_v), *v));
                    }
                }
            }
        }
        dist
    }

    pub fn dfs(&self, root: usize) -> DfsIterator {
        let mut visited = vec![false; self.num_v()];
        visited[root] = true;
        let adj_iters = (0..self.num_v())
            .map(|u| self.adj_list(u))
            .collect::<Vec<_>>();

        DfsIterator {
            visited,
            stack: vec![root],
            adj_iters,
        }
    }
    // this does not check for overflow
    // you can have negative edge weights, but you also need to have no negative cycles
    pub fn floyd_warshall(&self) -> Vec<Vec<i64>> {
        let numv = self.num_v();
        let mut dist = vec![vec![i64::MAX; numv]; numv];

        for v_idx in 0..numv {
            dist[v_idx][v_idx] = 0;
        }

        for (idx, edge) in self.edges.iter().enumerate() {
            dist[edge.0][edge.1] = self.edge_weights[idx];
        }

        for k in 0..numv {
            for i in 0..numv {
                for j in 0..numv {
                    if dist[i][k] < i64::MAX && dist[k][j] < i64::MAX {
                        if dist[i][j] > dist[i][k] + dist[k][j] {
                            dist[i][j] = dist[i][k] + dist[k][j];
                        }
                    }
                }
            }
        }
        dist
    }
}

impl UndirectedGraph {
    /// Kruskal's minimum spanning tree algorithm on an undirected weighted graph.
    pub fn min_spanning_tree(&self) -> Vec<usize> {
        let mut edges = (0..self.edge_weights.len()).collect::<Vec<_>>();
        edges.sort_unstable_by_key(|&e| self.edge_weights[e]);

        let mut components = DisjointSets::new(self.num_v());
        edges
            .into_iter()
            .filter(|&e| {
                let edge_vec = Vec::from_iter(&self.edges[e]);
                components.merge(*edge_vec[0], *edge_vec[1])
            })
            .collect()
    }
}
pub struct DfsIterator<'a> {
    visited: Vec<bool>,
    stack: Vec<usize>,
    adj_iters: Vec<AdjListIterator<'a>>,
}

impl<'a> Iterator for DfsIterator<'a> {
    type Item = (usize, usize);

    /// Returns next edge and vertex in the depth-first traversal
    // Refs: https://www.geeksforgeeks.org/iterative-depth-first-traversal/
    //       https://en.wikipedia.org/wiki/Depth-first_search
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let &u = self.stack.last()?;
            for (e, v) in self.adj_iters[u].by_ref() {
                if !self.visited[*v] {
                    self.visited[*v] = true;
                    self.stack.push(*v);
                    return Some((*e, *v));
                }
            }
            self.stack.pop();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_euler() {
        let mut graph = DirectedGraph::new(3, 4);
        graph.add_edge(0, 1);
        graph.add_edge(1, 0);
        graph.add_edge(1, 2);
        graph.add_edge(2, 1);

        assert_eq!(graph.euler_path(0), vec![0, 2, 3, 1]);
    }

    #[test]
    fn test_min_spanning_tree() {
        let mut graph = UndirectedGraph::new(3, 6);
        graph.add_weighted_edge(0, 1, 7);
        graph.add_weighted_edge(1, 2, 3);
        graph.add_weighted_edge(2, 0, 5);

        let mst = graph.min_spanning_tree();
        let mst_cost = mst.iter().map(|&e| graph.edge_weights[e]).sum::<i64>();
        assert_eq!(mst, vec![1, 2]);
        assert_eq!(mst_cost, 8);
    }

    #[test]
    fn test_dijkstra() {
        let mut graph = DirectedGraph::new(3, 3);
        graph.add_weighted_edge(0, 1, 7);
        graph.add_weighted_edge(1, 2, 3);
        graph.add_weighted_edge(2, 0, 5);

        let dist = graph.dijkstra(0);
        assert_eq!(dist, vec![0, 7, 10]);
    }

    #[test]
    fn test_dfs() {
        let mut graph = DirectedGraph::new(4, 6);
        graph.add_edge(0, 2);
        graph.add_edge(2, 0);
        graph.add_edge(1, 2);
        graph.add_edge(0, 1);
        graph.add_edge(3, 3);
        graph.add_edge(2, 3);

        let dfs_root = 2;
        let dfs_traversal = std::iter::once(dfs_root)
            .chain(graph.dfs(dfs_root).map(|(_, v)| v))
            .collect::<Vec<_>>();

        assert_eq!(dfs_traversal, vec![2, 0, 1, 3]);
    }

    #[test]
    fn test_dfs2() {
        let mut graph = DirectedGraph::new(5, 6);
        graph.add_edge(0, 2);
        graph.add_edge(2, 1);
        graph.add_edge(1, 0);
        graph.add_edge(0, 3);
        graph.add_edge(3, 4);
        graph.add_edge(4, 0);

        let dfs_root = 0;
        let dfs_traversal = std::iter::once(dfs_root)
            .chain(graph.dfs(dfs_root).map(|(_, v)| v))
            .collect::<Vec<_>>();

        assert_eq!(dfs_traversal, vec![0, 2, 1, 3, 4]);
    }

    #[test]
    fn test_dfs_space_complexity() {
        let num_v = 20;
        let mut graph = DirectedGraph::new(num_v, 0);
        for i in 0..num_v {
            for j in 0..num_v {
                graph.add_edge(i, j);
                graph.add_edge(j, i);
            }
        }

        let dfs_root = 7;
        let mut dfs_search = graph.dfs(dfs_root);
        let mut dfs_check = vec![dfs_root];
        for _ in 1..num_v {
            dfs_check.push(dfs_search.next().unwrap().1);
            assert!(dfs_search.stack.len() <= num_v + 1);
        }

        dfs_check.sort();
        dfs_check.dedup();
        assert_eq!(0, dfs_check[0]);
        assert_eq!(num_v, dfs_check.len());
        assert_eq!(num_v - 1, dfs_check[num_v - 1]);
    }

     #[test]
    fn test_floyd_warshall() {
        let num_v = 8;
        let mut graph = DirectedGraph::new(num_v, 10);
        graph.add_weighted_edge(0, 1, 1);
        graph.add_weighted_edge(1, 2, 2);
        graph.add_weighted_edge(1, 4, 4);
        graph.add_weighted_edge(2, 5, 3);
        graph.add_weighted_edge(4, 3, 6);
        graph.add_weighted_edge(5, 4, 10);
        graph.add_weighted_edge(3, 6, 2);
        graph.add_weighted_edge(4, 6, 7);
        graph.add_weighted_edge(6, 7, 2);
        graph.add_weighted_edge(5, 7, 9);

        let dist = graph.floyd_warshall();

        assert_eq!(dist[0][7], 14i64);
    }
}
