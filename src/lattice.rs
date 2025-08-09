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

pub struct Lattice {
    conn: Box<[[Neighbor; 6]]>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Direction(u8);

impl Direction {
    pub const RIGHT: Self = Self(0);
    pub const TOP_RIGHT: Self = Self(1);
    pub const TOP_LEFT: Self = Self(2);
    pub const LEFT: Self = Self(3);
    pub const BOTTOM_LEFT: Self = Self(4);
    pub const BOTTOM_RIGHT: Self = Self(5);

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

impl Lattice {
    pub fn new(num_nodes: usize) -> Self {
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

    fn neighbors(&self, id: u32) -> impl Iterator<Item = u32> {
        self.conn[id as usize].iter().filter_map(|n| n.get())
    }

    fn contains(&self, id: u32) -> bool {
        self.neighbors(id).next().is_some()
    }

    pub fn insert(&mut self, id: u32, dir: Direction, newid: u32) {
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
            // Orbit the loop counter clock wise direction.
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
    fn best_empty_slot() -> ((u32, Direction), [Neighbor; 6]) {
        todo!("Not Implemented")
    }
}

impl std::fmt::Display for Lattice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Track coordinates for each node
        let mut coords = vec![None; self.conn.len()];
        let mut visited = vec![false; self.conn.len()];

        // Assign coordinates to all nodes using flood fill from existing nodes
        for start_node in 0..self.conn.len() {
            if std::mem::replace(&mut visited[start_node], true)
                || !self.contains(start_node as u32)
            {
                continue;
            }

            let mut stack = vec![(start_node as u32, 0isize, 0isize)];

            while let Some((node, x, y)) = stack.pop() {
                let idx = node as usize;
                if coords[idx].is_some() {
                    continue;
                }

                coords[idx] = Some((x, y));

                const OFFSETS: [(isize, isize); 6] =
                    [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)];

                for dir in 0..6 {
                    if let Some(neighbor) = self.neighbor(node, Direction(dir)) {
                        let neighbor_idx = neighbor as usize;
                        if !std::mem::replace(&mut visited[neighbor_idx], true) {
                            let (dx, dy) = OFFSETS[dir as usize];
                            stack.push((neighbor, x + dx, y + dy));
                        }
                    }
                }
            }
        }

        // Collect all nodes with coordinates and sort by top-left order
        let mut all_nodes: Vec<(isize, isize, u32)> = coords
            .iter()
            .enumerate()
            .filter_map(|(idx, coord)| coord.map(|(x, y)| (x, y, idx as u32)))
            .collect();

        all_nodes.sort_by(|(ax, ay, _), (cx, cy, _)| {
            let a_sum = ax + ay;
            let c_sum = cx + cy;
            (std::cmp::Reverse(ay), a_sum).cmp(&(std::cmp::Reverse(cy), c_sum))
        });

        // Reset visited for component processing
        visited.fill(false);

        for &(start_x, start_y, start_node) in &all_nodes {
            if std::mem::replace(&mut visited[start_node as usize], true) {
                continue;
            }
            writeln!(f)?;
            // Flood fill this component and collect nodes by row
            let mut component_nodes = Vec::new();
            let mut stack = vec![start_node];

            while let Some(node) = stack.pop() {
                let idx = node as usize;
                if let Some((x, y)) = coords[idx] {
                    component_nodes.push((x, y, node));
                }

                for dir in 0..6 {
                    if let Some(neighbor) = self.neighbor(node, Direction(dir)) {
                        let neighbor_idx = neighbor as usize;
                        if !std::mem::replace(&mut visited[neighbor_idx], true) {
                            stack.push(neighbor);
                        }
                    }
                }
            }

            component_nodes.sort_by(|(ax, ay, _), (cx, cy, _)| {
                let a_sum = ax + ay;
                let c_sum = cx + cy;
                (std::cmp::Reverse(ay), a_sum).cmp(&(std::cmp::Reverse(cy), c_sum))
            });

            let (xmin, _ymin, _xmax, _ymax) = component_nodes.iter().fold(
                (isize::MAX, isize::MAX, isize::MIN, isize::MIN),
                |(xmin, ymin, xmax, ymax), &(x, y, _)| {
                    let x = x * 4 + 2 * y;
                    let y = y * 2;
                    (
                        isize::min(xmin, x - 1),
                        isize::min(ymin, y - 1),
                        isize::max(xmax, x + 1),
                        isize::max(ymax, y + 1),
                    )
                },
            );

            for row in component_nodes.chunk_by(|(_, ay1, _), (_, ay2, _)| ay1 == ay2) {
                let mut xoff = 0usize;
                for &(ix, iy, node) in row {
                    let has_right = self.neighbor(node, Direction::RIGHT).is_some();
                    let x = ((ix * 4 + 2 * iy) - xmin) as usize;
                    for _ in 0..(x - xoff) {
                        write!(f, " ")?;
                    }
                    write!(f, "{:^3}{}", node, if has_right { "-" } else { " " })?;
                    xoff = x + 4;
                }
                writeln!(f)?;

                // Print downlinks
                xoff = 0usize;
                for &(ix, iy, node) in row {
                    let has_bottom_left = self.neighbor(node, Direction::BOTTOM_LEFT).is_some();
                    let has_bottom_right = self.neighbor(node, Direction::BOTTOM_RIGHT).is_some();
                    let x = ((ix * 4 + 2 * iy) - xmin) as usize;
                    for _ in 0..(x - xoff) {
                        write!(f, " ")?;
                    }
                    write!(
                        f,
                        "{} {} ",
                        if has_bottom_left { "/" } else { " " },
                        if has_bottom_right { "\\" } else { " " }
                    )?;
                    xoff = x + 4;
                }
                writeln!(f)?;
            }

            writeln!(f)?;
        }
        Ok(())
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
