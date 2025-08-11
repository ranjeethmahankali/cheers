mod graph;
mod greedy;
mod lattice;

use graph::{Graph, TGraph};
use greedy::solve_greedy;
use lattice::Lattice;

fn verify(num_nodes: usize, solutions: &[Lattice]) {
    let mut graph = Graph::new_complete(num_nodes);
    for (a, b) in solutions.iter().flat_map(|lat| lat.edges()) {
        graph.remove_edge(a, b);
    }
    assert!(
        graph.is_empty(),
        "The set of solutions didn't cover all the edges of the complete graph"
    );
}

fn main() {
    let num_nodes = 19;
    let solns = solve_greedy::<Graph>(num_nodes);
    verify(num_nodes, &solns);
    println!("Found {}", solns.len());
    for soln in solns {
        println!("=============\n{}", soln);
    }

    // let mut lattice = Lattice::new(507);
    // lattice.insert(0, Direction::RIGHT, 1);
    // let mut visited = Vec::new();
    // let mut slots = Vec::new();
    // println!("{}\n===========================\n", lattice);
    // for id in 2u32..(lattice.len() as u32) {
    //     lattice.empty_slots(&mut visited, &mut slots);
    //     let (from, dir, _) = slots.last().unwrap();
    //     lattice.insert(*from, *dir, id);
    //     println!("{}\n===========================\n", lattice);
    // }

    // let mut our_lattice = Lattice::new(9);
    // our_lattice.insert(0, Direction::TOP_RIGHT, 1);
    // our_lattice.insert(0, Direction::RIGHT, 2);
    // our_lattice.insert(1, Direction::RIGHT, 3);
    // our_lattice.insert(1, Direction::TOP_RIGHT, 4);
    // our_lattice.insert(1, Direction::TOP_LEFT, 5);
    // our_lattice.insert(3, Direction::RIGHT, 6);
    // our_lattice.insert(3, Direction::TOP_RIGHT, 8);
    // our_lattice.insert(6, Direction::TOP_RIGHT, 7);

    // println!("Our Lattice:\n{}\n", our_lattice);

    // // Test the graph
    // let mut graph = Graph::new_complete(12);
    // println!("{}", graph);

    // graph.remove_edge(0, 1);
    // graph.remove_edge(5, 7);
    // graph.remove_edge(9, 11);
    // println!("After removing some edges:");
    // println!("{}", graph);
}
