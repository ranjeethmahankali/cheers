use fixedbitset::FixedBitSet;

use crate::{
    graph::TGraph,
    lattice::{Direction, Lattice},
};

pub fn solve_greedy<G>(num_nodes: usize) -> Vec<Lattice>
where
    G: TGraph,
{
    let mut graph = G::new_complete(num_nodes);
    let mut candidates = FixedBitSet::new();
    let mut lattice = Lattice::new(num_nodes);
    lattice.insert(0, Direction::RIGHT, 1);
    let mut latnbs = Vec::new();
    let mut visitedbuf = Vec::new();
    let mut nb_temp = Vec::new();
    while let Some((id, dir)) = lattice.best_empty_slot(&mut visitedbuf, &mut nb_temp, &mut latnbs)
        && !graph.is_empty()
    {
        graph.find_candidates(&latnbs, &mut candidates);
        let best = match candidates
            .ones()
            .map(|c| (graph.valence(c), c as u32))
            .max()
        {
            Some((_, newid)) => newid,
            None => continue,
        };
    }
    todo!("Not Implemented")
}
