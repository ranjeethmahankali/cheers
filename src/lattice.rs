use std::{collections::HashMap, num::NonZeroU32, ops::Index};

// Slot where a vertex maybe stored. The nonzerou32 stuff is to optimize the storage
// for the two states when the vertex does and does not exist in the slot.
#[repr(transparent)]
#[derive(Copy, Clone)]
struct Neighbor(Option<NonZeroU32>);

impl Default for Neighbor {
    fn default() -> Self {
        Self(None)
    }
}

impl Neighbor {
    fn put(&mut self, id: u32) {
        self.0 = NonZeroU32::new(id ^ u32::MAX);
    }

    fn get(&self) -> Option<u32> {
        self.0.map(|v| v.get() ^ u32::MAX)
    }

    fn exists(&self) -> bool {
        self.0.is_some()
    }
}

struct Lattice {
    neighbors: Box<[[Neighbor; 6]]>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct Direction(u8);

impl Direction {
    const RIGHT: Self = Self(0);
    const TOP_RIGHT: Self = Self(1);
    const TOP_LEFT: Self = Self(2);
    const LEFT: Self = Self(3);
    const BOTTOM_LEFT: Self = Self(4);
    const BOTTOM_RIGHT: Self = Self(5);

    const fn opposite(self) -> Self {
        Self((self.0 + 3) % 6)
    }

    const fn rotate_cw(self) -> Self {
        Self((self.0 + 1) % 6)
    }

    const fn rotate_ccw(self) -> Self {
        Self((self.0 + 5) % 6)
    }
}

impl Index<Direction> for [Neighbor; 6] {
    type Output = Neighbor;

    fn index(&self, dir: Direction) -> &Self::Output {
        &self[dir.0 as usize]
    }
}

struct Slot(u32, Direction);

impl Lattice {
    fn new(num_nodes: usize) -> Self {
        Self {
            neighbors: vec![Default::default(); num_nodes].into_boxed_slice(),
        }
    }

    fn step_loop_ccw(&self, node_id: u32, direction: Direction) -> Option<(u32, Direction, u8)> {
        let nb = self.neighbors[node_id as usize][direction].get()?;
        let stop = direction.opposite();
        let mut dir = direction.opposite().rotate_ccw();
        let mut rotations = 1;
        while dir != stop {
            if self.neighbors[nb as usize][dir].exists() {
                return Some((nb, dir, rotations));
            }
            dir = dir.rotate_ccw();
            rotations += 1;
        }
        None
    }

    fn step_loop_cw(&self, node_id: u32, direction: Direction) -> Option<(u32, Direction, u8)> {
        let nb = self.neighbors[node_id as usize][direction].get()?;
        let stop = direction.opposite();
        let mut dir = direction.opposite().rotate_cw();
        let mut rotations = 1;
        while dir != stop {
            if self.neighbors[nb as usize][dir].exists() {
                return Some((nb, dir, rotations));
            }
            dir = dir.rotate_cw();
            rotations += 1;
        }
        None
    }

    fn insert(id: u32, slot: Slot) {
        todo!("Not Implemented")
    }

    /// Return the empty slot with the highest valence and it's neighbors.
    fn best_empty_slot() -> (Slot, [Neighbor; 6]) {
        todo!("Not Implemented")
    }
}
