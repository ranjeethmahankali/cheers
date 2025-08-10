use fixedbitset::FixedBitSet;

use crate::{
    graph::TGraph,
    lattice::{Direction, Lattice, Neighbor},
};

fn count_nbs(nbs: &[Neighbor; 6]) -> usize {
    nbs.iter().filter_map(|n| n.get()).count()
}

pub fn solve_greedy<G>(num_nodes: usize) -> Vec<Lattice>
where
    G: TGraph,
{
    let mut out = Vec::new();
    let mut graph = G::new_complete(num_nodes);
    let mut candidates = FixedBitSet::new();
    let mut lattice = Lattice::new(num_nodes);
    lattice.insert(0, Direction::RIGHT, 1);
    let mut latnbs = Vec::new();
    let mut visitedbuf = Vec::new();
    let mut slots = Vec::new();
    while !graph.is_empty() {
        lattice.empty_slots(&mut visitedbuf, &mut slots);
        slots.sort_by(|(_, _, anbs), (_, _, bnbs)| count_nbs(anbs).cmp(&count_nbs(bnbs)));
        let mut found = false;
        while let Some((id, dir, nbs)) = slots.pop() {
            latnbs.clear();
            latnbs.extend(nbs.iter().filter_map(|n| n.get()));
            if latnbs.is_empty() {
                continue;
            }
            graph.find_candidates(&latnbs, &mut candidates);
            let best = match candidates.ones().find(|i| !lattice.contains(*i as u32)) {
                Some(i) => i as u32,
                None => continue,
            };
            lattice.insert(id, dir, best);
            for nb in lattice.neighbors(best) {
                graph.remove_edge(best, nb);
            }
            found = true;
            break;
        }
        if !found {
            out.push(lattice.clone());
        }
    }
    todo!("Not Implemented")
}
