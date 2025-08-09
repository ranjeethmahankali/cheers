use std::{
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

    fn clear(&mut self) {
        self.0 = None;
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

    const ALL_CCW: [Direction; 6] = [
        Direction::RIGHT,
        Direction::TOP_RIGHT,
        Direction::TOP_LEFT,
        Direction::LEFT,
        Direction::BOTTOM_LEFT,
        Direction::BOTTOM_RIGHT,
    ];

    const fn opposite(self) -> Self {
        Self((self.0 + 3) % 6)
    }

    const fn rotate_cw(self) -> Self {
        Self((self.0 + 1) % 6)
    }

    const fn rotate_ccw(self) -> Self {
        Self((self.0 + 5) % 6)
    }

    const fn offset(&self) -> (isize, isize) {
        const OFFSETS: [(isize, isize); 6] = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)];
        return OFFSETS[self.0 as usize];
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
            if self.neighbor(nb, dir).is_some() {
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
            if self.neighbor(nb, dir).is_some() {
                return Some((nb, dir, rotations));
            }
            dir = dir.rotate_cw();
            rotations += 1;
        }
        None
    }

    fn neighbor(&self, from: u32, dir: Direction) -> Option<u32> {
        self.conn[from as usize][dir].get()
    }

    fn neighbors(&self, id: u32) -> impl Iterator<Item = u32> {
        self.conn[id as usize].iter().filter_map(|n| n.get())
    }

    fn neighbors_with_dirs(&self, id: u32) -> impl Iterator<Item = (u32, Direction)> {
        self.conn[id as usize]
            .iter()
            .zip(Direction::ALL_CCW.iter())
            .filter_map(|(n, &d)| n.get().map(|n| (n, d)))
    }

    fn contains(&self, id: u32) -> bool {
        self.neighbors(id).next().is_some()
    }

    pub fn remove(&mut self, id: u32) {
        let mut nbs = [u32::MAX; 6];
        let mut dirs = [Direction::RIGHT; 6];
        let mut count = 0usize;
        // Collect them first, to avoid borrowing and modifying
        for (nb, dir) in self.conn[id as usize].iter().zip(Direction::ALL_CCW) {
            if let Some(nb) = nb.get() {
                nbs[count] = nb;
                dirs[count] = dir;
                count += 1;
            }
        }
        for (&nb, &dir) in nbs.iter().zip(dirs.iter()).take(count) {
            self.conn[id as usize][dir].clear();
            self.conn[nb as usize][dir.opposite()].clear();
        }
    }

    pub fn insert(&mut self, id: u32, dir: Direction, newid: u32) {
        // Remove if something was there.
        if let Some(nb) = self.neighbor(id, dir) {
            self.remove(nb);
        }
        // Now insert.
        self.conn[id as usize][dir].put(newid);
        self.conn[newid as usize][dir.opposite()].put(id);
        {
            // Orbit the loop clockwise and link nodes.
            let mut id = id;
            let mut dir = dir.rotate_ccw();
            while let Some(next) = self.neighbor(id, dir) {
                dir = dir.opposite().rotate_ccw();
                self.conn[next as usize][dir].put(newid);
                self.conn[newid as usize][dir.opposite()].put(next);
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
                self.conn[next as usize][dir].put(newid);
                self.conn[newid as usize][dir.opposite()].put(next);
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
        let mut visited = vec![false; self.conn.len()];
        let mut stack = Vec::new();
        let mut component_nodes = Vec::new();
        for start_node in 0..self.conn.len() {
            if std::mem::replace(&mut visited[start_node], true)
                || !self.contains(start_node as u32)
            {
                continue;
            }
            component_nodes.clear();
            stack.clear();
            stack.push((start_node as u32, 0isize, 0isize));
            while let Some((node, x, y)) = stack.pop() {
                component_nodes.push((x, y, node));
                for (neighbor, dir) in self.neighbors_with_dirs(node) {
                    if !std::mem::replace(&mut visited[neighbor as usize], true) {
                        let (dx, dy) = dir.offset();
                        stack.push((neighbor, x + dx, y + dy));
                    }
                }
            }
            component_nodes.sort_by(|(ax, ay, _), (cx, cy, _)| {
                (std::cmp::Reverse(ay), ax + ay).cmp(&(std::cmp::Reverse(cy), cx + cy))
            });
            let (xmin, _ymin, _xmax, _ymax) = component_nodes.iter().fold(
                (isize::MAX, isize::MAX, isize::MIN, isize::MIN),
                |(xmin, ymin, xmax, ymax), &(x, y, _)| {
                    let x = x * 4 + 2 * y;
                    let y = y * 2;
                    (
                        xmin.min(x - 1),
                        ymin.min(y - 1),
                        xmax.max(x + 1),
                        ymax.max(y + 1),
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
                xoff = 0;
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
            assert!(!neighbor.get().is_some());
            neighbor.put(id);
            assert_eq!(neighbor.get(), Some(id));
        }
    }

    #[test]
    fn test_neighbor_max_value_edge_case() {
        let mut neighbor = Neighbor::default();
        neighbor.put(u32::MAX);
        assert!(neighbor.get().is_none());
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
