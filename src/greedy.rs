use crate::{graph::TGraph, lattice::Lattice};

pub fn solve_greedy<G>(num_nodes: usize) -> Vec<Lattice>
where
    G: TGraph,
{
    let graph = G::new_complete(num_nodes);
    todo!("Not Implemented")
}
