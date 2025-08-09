use fixedbitset::FixedBitSet;
use std::fmt::Display;

pub trait TGraph: Clone {
    fn new_complete(n: usize) -> Self;
    fn has_edge(&self, i: usize, j: usize) -> bool;
    fn remove_edge(&mut self, i: usize, j: usize);
    fn edge_count(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn valence(&self, node: usize) -> usize;
    fn find_candidates(&self, required: &[usize], candidates: &mut FixedBitSet);
    fn node_count(&self) -> usize;
}

#[derive(Clone)]
pub struct Graph {
    n: usize,
    neighbors: Vec<FixedBitSet>,
}

impl TGraph for Graph {
    fn new_complete(n: usize) -> Self {
        let mut neighbors = Vec::with_capacity(n);

        for i in 0..n {
            let mut node_neighbors = FixedBitSet::with_capacity(n);
            // Insert all nodes except self
            node_neighbors.insert_range(..i);
            node_neighbors.insert_range((i + 1)..n);
            neighbors.push(node_neighbors);
        }

        Self { n, neighbors }
    }

    fn has_edge(&self, i: usize, j: usize) -> bool {
        self.neighbors[i].contains(j)
    }

    fn remove_edge(&mut self, i: usize, j: usize) {
        self.neighbors[i].remove(j);
        self.neighbors[j].remove(i);
    }

    fn edge_count(&self) -> usize {
        self.neighbors
            .iter()
            .map(|n| n.count_ones(..))
            .sum::<usize>()
            / 2
    }

    fn is_empty(&self) -> bool {
        self.neighbors.iter().all(|n| n.is_clear())
    }

    fn valence(&self, node: usize) -> usize {
        self.neighbors[node].count_ones(..)
    }

    fn find_candidates(&self, required: &[usize], candidates: &mut FixedBitSet) {
        candidates.clear();

        if required.is_empty() {
            candidates.insert_range(..self.n);
            return;
        }

        // Start with neighbors of first required node
        candidates.clone_from(&self.neighbors[required[0]]);

        // Intersect with neighbors of each subsequent required node
        for &node in &required[1..] {
            candidates.intersect_with(&self.neighbors[node]);
        }
    }

    fn node_count(&self) -> usize {
        self.n
    }
}

impl Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Graph K_{} ({} edges remaining):",
            self.n,
            self.edge_count()
        )?;

        for i in 0..self.n {
            write!(f, "Node {}: ", i)?;
            let mut first = true;
            for j in self.neighbors[i].ones() {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "{}", j)?;
                first = false;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_graph_creation() {
        let graph = Graph::new_complete(4);
        assert_eq!(graph.edge_count(), 6); // K4 has 6 edges

        // Check all pairs are connected
        for i in 0..4 {
            for j in 0..4 {
                if i != j {
                    assert!(graph.has_edge(i, j));
                } else {
                    assert!(!graph.has_edge(i, j));
                }
            }
        }
    }

    #[test]
    fn test_edge_removal() {
        let mut graph = Graph::new_complete(3);
        assert_eq!(graph.edge_count(), 3); // K3 has 3 edges

        graph.remove_edge(0, 1);
        assert_eq!(graph.edge_count(), 2);
        assert!(!graph.has_edge(0, 1));
        assert!(!graph.has_edge(1, 0));
        assert!(graph.has_edge(0, 2));
        assert!(graph.has_edge(1, 2));
    }

    #[test]
    fn test_valence_calculation() {
        let graph = Graph::new_complete(4);

        // In K4, every node has valence 3
        for i in 0..4 {
            assert_eq!(graph.valence(i), 3);
        }
    }

    #[test]
    fn test_find_candidates() {
        let graph = Graph::new_complete(4);
        let mut candidates = FixedBitSet::with_capacity(4);

        // All nodes should be candidates when no requirements
        graph.find_candidates(&[], &mut candidates);
        assert_eq!(candidates.count_ones(..), 4);

        // In complete graph, all other nodes connect to any given node
        graph.find_candidates(&[0], &mut candidates);
        let result: Vec<usize> = candidates.ones().collect();
        assert_eq!(result, vec![1, 2, 3]);

        // In complete graph, all nodes connect to any pair
        graph.find_candidates(&[0, 1], &mut candidates);
        let result: Vec<usize> = candidates.ones().collect();
        assert_eq!(result, vec![2, 3]);
    }

    #[test]
    fn test_is_empty() {
        let mut graph = Graph::new_complete(2);
        assert!(!graph.is_empty());

        graph.remove_edge(0, 1);
        assert!(graph.is_empty());
    }
}
