mod graph;
mod greedy;
mod lattice;

use graph::{Graph, TGraph};
use std::fmt::Display;

#[derive(Debug)]
enum Error {
    AlreadyOccupied(isize, isize),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AlreadyOccupied(x, y) => {
                write!(f, "Position ({}, {}) is already occupied", x, y)
            }
        }
    }
}

fn main() -> Result<(), Error> {
    use lattice::{Direction, *};

    let mut lattice = Lattice::new(507);
    lattice.insert(0, Direction::RIGHT, 1);
    let mut visited = Vec::new();
    let mut nb_buf = Vec::new();
    let mut nbs_out = Vec::new();
    println!("{}\n===========================\n", lattice);
    for id in 2u32..(lattice.len() as u32) {
        let (best, dir) = lattice
            .best_empty_slot(&mut visited, &mut nb_buf, &mut nbs_out)
            .expect("Cannot find the best empty slot");
        lattice.insert(best, dir, id);
        println!("{}\n===========================\n", lattice);
    }

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

    Ok(())
}
