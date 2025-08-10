use fixedbitset::FixedBitSet;
use std::fmt::Display;

pub trait TGraph: Clone + Display {
    fn new_complete(n: usize) -> Self;
    fn has_edge(&self, i: u32, j: u32) -> bool;
    fn remove_edge(&mut self, i: u32, j: u32);
    fn num_edges(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn valence(&self, node: u32) -> usize;
    fn find_candidates(&self, required: &[u32], candidates: &mut FixedBitSet);
    fn num_nodes(&self) -> usize;
}

#[derive(Clone)]
pub struct Graph {
    n_nodes: usize,
    conn: Vec<FixedBitSet>,
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

        Self {
            n_nodes: n,
            conn: neighbors,
        }
    }

    fn has_edge(&self, i: u32, j: u32) -> bool {
        self.conn[i as usize].contains(j as usize)
    }

    fn remove_edge(&mut self, i: u32, j: u32) {
        self.conn[i as usize].remove(j as usize);
        self.conn[j as usize].remove(i as usize);
    }

    fn num_edges(&self) -> usize {
        self.conn.iter().map(|n| n.count_ones(..)).sum::<usize>() / 2
    }

    fn is_empty(&self) -> bool {
        self.conn.iter().all(|n| n.is_clear())
    }

    fn valence(&self, node: u32) -> usize {
        self.conn[node as usize].count_ones(..)
    }

    fn find_candidates(&self, required: &[u32], candidates: &mut FixedBitSet) {
        candidates.clear();
        candidates.grow_and_insert(self.n_nodes);
        if required.is_empty() {
            return;
        }
        // Start with neighbors of first required node
        candidates.clone_from(&self.conn[required[0] as usize]);
        // Intersect with neighbors of each subsequent required node
        for &node in &required[1..] {
            candidates.intersect_with(&self.conn[node as usize]);
        }
    }

    fn num_nodes(&self) -> usize {
        self.n_nodes
    }
}

impl Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Graph K_{} ({} edges remaining):",
            self.n_nodes,
            self.num_edges()
        )?;
        // Calculate how many digits we need for the largest number
        let max_digits = if self.n_nodes == 0 {
            1
        } else {
            (self.n_nodes - 1).to_string().len()
        };
        // Top border - close the row label area and connect to main area
        write!(f, "┌{:─<width$}┬", "", width = max_digits)?;
        for _j in 0..self.n_nodes {
            write!(f, "─")?;
        }
        writeln!(f, "┐")?;
        // Print each row with row labels and borders (only bottom-right triangle)
        for i in 0..(self.n_nodes as u32) {
            write!(f, "│{:width$}│", i, width = max_digits)?;
            for j in 0..=i {
                if i == j {
                    write!(f, " ")?; // Diagonal (self-loops don't exist)
                } else if self.has_edge(i, j) {
                    write!(f, "x")?; // Edge exists
                } else {
                    write!(f, " ")?; // No edge
                }
            }
            // Fill remaining space to align with full width
            for _j in (i + 1)..(self.n_nodes as u32) {
                write!(f, " ")?;
            }
            writeln!(f, "│")?;
        }
        // Bottom border of row label area and separator to column labels
        write!(f, "└{:─<width$}┼", "", width = max_digits)?;
        for _j in 0..self.n_nodes {
            write!(f, "─")?;
        }
        writeln!(f, "┤")?;
        // Column headers - print digits vertically, bottom-aligned (least significant digit closest to table)
        for digit_pos in 0..max_digits {
            write!(f, "{:width$}│", "", width = max_digits + 1)?;
            for j in 0..self.n_nodes {
                let j_str = format!("{:0width$}", j, width = max_digits);
                write!(f, "{}", j_str.chars().nth(digit_pos).unwrap())?;
            }
            writeln!(f, "│")?;
        }
        // Bottom border
        write!(f, "{:width$}└", "", width = max_digits + 1)?;
        for _j in 0..self.n_nodes {
            write!(f, "─")?;
        }
        writeln!(f, "┘")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_graph_creation() {
        let graph = Graph::new_complete(4);
        assert_eq!(graph.num_edges(), 6); // K4 has 6 edges
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
        assert_eq!(graph.num_edges(), 3); // K3 has 3 edges
        graph.remove_edge(0, 1);
        assert_eq!(graph.num_edges(), 2);
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
