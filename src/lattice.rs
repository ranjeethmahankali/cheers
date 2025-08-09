use std::{
    collections::HashMap,
    num::NonZeroU32,
    ops::{Index, IndexMut},
};

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
    conn: Box<[[Neighbor; 6]]>,
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

impl IndexMut<Direction> for [Neighbor; 6] {
    fn index_mut(&mut self, dir: Direction) -> &mut Self::Output {
        &mut self[dir.0 as usize]
    }
}

struct Slot(u32, Direction);

impl Lattice {
    fn new(num_nodes: usize) -> Self {
        Self {
            conn: vec![Default::default(); num_nodes].into_boxed_slice(),
        }
    }

    fn step_loop_ccw(&self, node_id: u32, direction: Direction) -> Option<(u32, Direction, u8)> {
        let nb = self.neighbor(node_id, direction)?;
        let stop = direction.opposite();
        let mut dir = direction.opposite().rotate_ccw();
        let mut rotations = 1;
        while dir != stop {
            if self.conn[nb as usize][dir].exists() {
                return Some((nb, dir, rotations));
            }
            dir = dir.rotate_ccw();
            rotations += 1;
        }
        None
    }

    fn step_loop_cw(&self, node_id: u32, direction: Direction) -> Option<(u32, Direction, u8)> {
        let nb = self.neighbor(node_id, direction)?;
        let stop = direction.opposite();
        let mut dir = direction.opposite().rotate_cw();
        let mut rotations = 1;
        while dir != stop {
            if self.conn[nb as usize][dir].exists() {
                return Some((nb, dir, rotations));
            }
            dir = dir.rotate_cw();
            rotations += 1;
        }
        None
    }

    fn link(&mut self, from: u32, dir: Direction, to: u32) {
        self.conn[from as usize][dir].put(to);
        self.conn[to as usize][dir.opposite()].put(from);
    }

    fn neighbor(&self, from: u32, dir: Direction) -> Option<u32> {
        self.conn[from as usize][dir].get()
    }

    fn insert(&mut self, id: u32, Slot(newid, dir): Slot) {
        self.link(id, dir, newid);
        {
            // Orbit the loop clockwise and link nodes.
            let mut id = id;
            let mut dir = dir.rotate_ccw();
            while let Some(next) = self.neighbor(id, dir) {
                dir = dir.opposite().rotate_ccw();
                self.link(next, dir, newid);
                dir = dir.rotate_ccw();
                id = next;
            }
        }
        {
            // Orbig the loop counter clock wise direction.
            // This may not be required depending on how far the other loop went, but leaving this in for now.
            // Will think about it if it becomes a bottlneck.
            let mut id = id;
            let mut dir = dir.rotate_cw();
            while let Some(next) = self.neighbor(id, dir) {
                dir = dir.opposite().rotate_cw();
                self.link(next, dir, newid);
                dir = dir.rotate_cw();
                id = next;
            }
        }
    }

    /// Return the empty slot with the highest valence and it's neighbors.
    fn best_empty_slot() -> (Slot, [Neighbor; 6]) {
        todo!("Not Implemented")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_neighbor_put_get() {
        let test_values = [0, 1, 42, 1000, u32::MAX / 2, u32::MAX - 1];

        for &id in &test_values {
            let mut neighbor = Neighbor::default();
            assert!(!neighbor.exists());
            assert_eq!(neighbor.get(), None);

            neighbor.put(id);
            assert!(neighbor.exists());
            assert_eq!(neighbor.get(), Some(id));
        }
    }

    #[test]
    fn test_neighbor_max_value_edge_case() {
        let mut neighbor = Neighbor::default();
        neighbor.put(u32::MAX);
        assert!(!neighbor.exists());
        assert_eq!(neighbor.get(), None);
    }

    #[test]
    fn test_neighbor_overwrite() {
        let mut neighbor = Neighbor::default();

        neighbor.put(42);
        assert_eq!(neighbor.get(), Some(42));

        neighbor.put(0);
        assert_eq!(neighbor.get(), Some(0));
    }
}
