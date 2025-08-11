use std::{
    fmt::{Debug, Display},
    num::NonZeroU32,
    ops::{Index, IndexMut},
};

// Slot where a vertex maybe stored. The nonzerou32 stuff is to optimize the storage
// for the two states when the vertex does and does not exist in the slot.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Neighbor(Option<NonZeroU32>);

impl Default for Neighbor {
    fn default() -> Self {
        Self(None)
    }
}

impl Debug for Neighbor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.get() {
            Some(id) => write!(f, "{id}"),
            None => write!(f, "NONE"),
        }
    }
}

impl Neighbor {
    fn put(&mut self, id: u32) {
        self.0 = NonZeroU32::new(id ^ u32::MAX);
    }

    pub fn get(&self) -> Option<u32> {
        self.0.map(|v| v.get() ^ u32::MAX)
    }

    fn clear(&mut self) {
        self.0 = None;
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Direction(u8);

impl Debug for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

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

    const fn rotate_ccw(self) -> Self {
        Self((self.0 + 1) % 6)
    }

    const fn rotate_cw(self) -> Self {
        Self((self.0 + 5) % 6)
    }

    const fn offset(&self) -> (isize, isize) {
        const OFFSETS: [(isize, isize); 6] = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)];
        return OFFSETS[self.0 as usize];
    }

    const fn as_str(&self) -> &str {
        match self.0 {
            0 => "RIGHT",
            1 => "TOP_RIGHT",
            2 => "TOP_LEFT",
            3 => "LEFT",
            4 => "BOTTOM_LEFT",
            5 => "BOTTOM_RIGHT",
            _ => panic!("Invalid direction. This should never happen."),
        }
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

#[derive(Clone)]
pub struct Lattice {
    conn: Box<[[Neighbor; 6]]>,
}

impl Lattice {
    pub fn new(num_nodes: usize) -> Self {
        Self {
            conn: vec![Default::default(); num_nodes].into_boxed_slice(),
        }
    }

    pub fn len(&self) -> usize {
        self.conn.len()
    }

    pub fn clear(&mut self) {
        for nbs in &mut self.conn {
            nbs.fill(Neighbor::default());
        }
    }

    fn step_loop_ccw(&self, node_id: u32, direction: Direction) -> Option<(u32, Direction, u8)> {
        let nb = self.neighbor(node_id, direction)?;
        let stop = direction.opposite();
        let mut dir = stop.rotate_ccw();
        let mut rotations = 1;
        while dir != stop {
            if self.neighbor(nb, dir).is_some() {
                return Some((nb, dir, rotations));
            }
            dir = dir.rotate_ccw();
            rotations += 1;
        }
        Some((nb, stop, 6))
    }

    fn step_loop_cw(&self, node_id: u32, direction: Direction) -> Option<(u32, Direction, u8)> {
        let nb = self.neighbor(node_id, direction)?;
        let stop = direction.opposite();
        let mut dir = stop.rotate_cw();
        let mut rotations = 1;
        while dir != stop {
            if self.neighbor(nb, dir).is_some() {
                return Some((nb, dir, rotations));
            }
            dir = dir.rotate_cw();
            rotations += 1;
        }
        Some((nb, stop, 6))
    }

    fn neighbor(&self, from: u32, dir: Direction) -> Option<u32> {
        self.conn[from as usize][dir].get()
    }

    pub fn neighbors(&self, id: u32) -> impl Iterator<Item = u32> {
        self.conn[id as usize].iter().filter_map(|n| n.get())
    }

    pub fn edges(&self) -> impl Iterator<Item = (u32, u32)> {
        (0..self.len()).flat_map(|id| {
            self.conn[id]
                .iter()
                .filter_map(move |nb| nb.get().map(|nb| (id as u32, nb)))
                .filter(|(a, b)| a < b)
        })
    }

    fn neighbors_with_dirs(&self, id: u32) -> impl Iterator<Item = (u32, Direction)> {
        self.conn[id as usize]
            .iter()
            .zip(Direction::ALL_CCW.iter())
            .filter_map(|(n, &d)| n.get().map(|n| (n, d)))
    }

    pub fn contains(&self, id: u32) -> bool {
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
        if id == newid {
            return;
        }
        // Remove if something was there.
        if let Some(nb) = self.neighbor(id, dir) {
            self.remove(nb);
        }
        if let Some(nb) = self.neighbor(newid, dir.opposite()) {
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
    ///
    /// `visited` and `nb_buf` are temporary buffers used in this function,
    /// passed in by the caller to avoid allocations.
    pub fn empty_slots(
        &self,
        visited: &mut Vec<bool>,
        out: &mut Vec<(u32, Direction, [Neighbor; 6])>,
    ) {
        visited.clear();
        visited.resize(self.len(), false);
        out.clear();
        for id in 0u32..(self.len() as u32) {
            if visited[id as usize] {
                continue;
            }
            // Find boundary edge.
            let dir = match self
                .neighbors_with_dirs(id)
                .find(|(_, dir)| self.neighbor(id, dir.rotate_cw()).is_none())
            {
                Some((_, dir)) => dir,
                None => continue,
            };
            let mut curid = id;
            let mut dir = dir;
            // If we happen to be in the middle of a concavity, we don't want to
            // start counting from here.  So we try to walk backwards to the
            // start of this concavity before we start counting.
            curid = self
                .neighbor(curid, dir)
                .expect("Topology is broken if we don't get this");
            dir = dir.opposite();
            loop {
                let (next, ndir, nrot) = self
                    .step_loop_cw(curid, dir)
                    .expect("We're on the boundary loop. This should never happen");
                match nrot {
                    1 => panic!("This implies broken topology. This should never happen"),
                    2 => {
                        curid = next;
                        dir = ndir;
                    }
                    _ => break,
                }
            }
            curid = self
                .neighbor(curid, dir)
                .expect("Topology is broken if we don't get this");
            dir = dir.opposite();
            let mut curndir = dir.rotate_cw();
            let mut curnb = [Neighbor::default(); 6];
            loop {
                visited[curid as usize] = true;
                curnb[curndir.opposite()].put(curid);
                let (next, ndir, nrot) = self
                    .step_loop_ccw(curid, dir)
                    .expect("This is a boundary edge, so the loop step should never fail");
                match nrot {
                    0 | 1 => panic!("This implies broken topology. This should never happen"),
                    2 => {} // Keep going.
                    _ => {
                        curnb[curndir.opposite().rotate_cw()].put(next);
                        out.push((curid, curndir, curnb));
                        curnb.fill(Neighbor::default());
                        {
                            let mut odir = dir.opposite();
                            for _ in 0..(nrot - 2) {
                                odir = odir.rotate_ccw();
                                let mut nbs = [Neighbor::default(); 6];
                                nbs[odir.opposite()].put(next);
                                out.push((next, odir, nbs));
                            }
                        }
                    }
                }
                if next == id {
                    break;
                }
                curid = next;
                dir = ndir;
                curndir = dir.rotate_cw();
            }
        }
    }

    /// Check lattice for consistency - verifies all lattice invariants
    pub fn validate(&self) {
        for node in 0u32..(self.len() as u32) {
            // Skip empty nodes
            if !self.contains(node) {
                continue;
            }
            // Check bidirectional connections
            for dir in Direction::ALL_CCW {
                if let Some(neighbor_id) = self.neighbor(node, dir) {
                    // Verify neighbor points back to this node
                    let back_neighbor = self.neighbor(neighbor_id, dir.opposite());
                    assert_eq!(
                        back_neighbor,
                        Some(node),
                        "Node {} has neighbor {} in direction {:?}, but neighbor {} doesn't point back (has {:?} instead of Some({}))",
                        node,
                        neighbor_id,
                        dir,
                        neighbor_id,
                        back_neighbor,
                        node
                    );
                    // Verify neighbor exists in lattice
                    assert!(
                        self.contains(neighbor_id),
                        "Node {} has neighbor {} in direction {:?}, but neighbor {} doesn't exist in lattice",
                        node,
                        neighbor_id,
                        dir,
                        neighbor_id
                    );
                }
            }
            // Check triangular loops using step_loop functions
            for (_, dir) in self.neighbors_with_dirs(node) {
                if let Some((last, _)) = (0..3).fold(Some((node, dir)), |current, _| {
                    if let Some((id, dir)) = current
                        && let Some((next, ndir, nrot)) = self.step_loop_cw(id, dir)
                        && nrot == 1
                    {
                        Some((next, ndir))
                    } else {
                        None
                    }
                }) {
                    assert_eq!(last, node);
                }
                if let Some((last, _)) = (0..3).fold(Some((node, dir)), |current, _| {
                    if let Some((id, dir)) = current
                        && let Some((next, ndir, nrot)) = self.step_loop_ccw(id, dir)
                        && nrot == 1
                    {
                        Some((next, ndir))
                    } else {
                        None
                    }
                }) {
                    assert_eq!(last, node);
                }
            }
            // Check that no node references itself as a neighbor
            for neighbor in self.neighbors(node) {
                assert_ne!(neighbor, node, "Node {} has itself as a neighbor", node);
            }
        }
    }
}

impl Display for Lattice {
    /*
    The hexagonal grids are stored in a coordinate system where the axes are
    squished together to 60 degrees.

               (0, 1) * ------- * (1, 1)
                     / \       / \
                    /   \     /   \
                   /     \   /     \
                  /       \ /       \
          (0, 0) * ------- * ------- * (2, 0)
                / \     (1, 0)      /
               /   \     /   \     /
              /     \   /     \   /
             /       \ /       \ /
            * ------- * ------- *
        (0, -1)     (1, -1)     (2, -1)

    This is because a hexagonal grid and a rectangular grid are topologically
    equivalent. The only difference is that, in the convention of the above diagram,
    the diagonal in the (-1, 1) direction and the diagonal (1, -1) are considered
    neighbors of the point in the middle. This means I can store the points in a
    rectangular grid, and infer the connectivity from the indices.
     */

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
                |(xmin, ymin, xmax, ymax), &(x, y, _node)| {
                    let x = x * 4 + 2 * y;
                    let y = y * 2;
                    eprintln!("When doing min: {x}, {y} for node {_node}");
                    (
                        xmin.min(x - 1),
                        ymin.min(y - 1),
                        xmax.max(x + 1),
                        ymax.max(y + 1),
                    )
                },
            );
            eprintln!("Bounds: {xmin}, {_ymin}, {_xmax}, {_ymax}");
            for row in component_nodes.chunk_by(|(_, ay1, _), (_, ay2, _)| ay1 == ay2) {
                let mut xoff = 0usize;
                for &(ix, iy, node) in row {
                    let has_right = self.neighbor(node, Direction::RIGHT).is_some();
                    let x = ((ix * 4 + 2 * iy) - xmin) as usize;
                    eprintln!("node {node}; ({ix}, {iy}); {xmin}; {x} and {xoff}");
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
    fn t_neighbor_put_get() {
        let values = [0, 1, 42, 1000, u32::MAX / 2, u32::MAX - 1];
        for &id in &values {
            let mut neighbor = Neighbor::default();
            assert!(!neighbor.get().is_some());
            neighbor.put(id);
            assert_eq!(neighbor.get(), Some(id));
        }
    }

    #[test]
    fn t_neighbor_max_value_edge_case() {
        let mut neighbor = Neighbor::default();
        neighbor.put(u32::MAX);
        assert!(neighbor.get().is_none());
    }

    #[test]
    fn t_neighbor_overwrite() {
        let mut neighbor = Neighbor::default();
        neighbor.put(42);
        assert_eq!(neighbor.get(), Some(42));
        neighbor.put(0);
        assert_eq!(neighbor.get(), Some(0));
    }

    #[test]
    fn t_print_empty_lattice() {
        let lattice = Lattice::new(5);
        lattice.validate();
        let output = format!("{}", lattice);
        assert_eq!(output, "");
    }

    #[test]
    fn t_print_single_node_lattice() {
        let mut lattice = Lattice::new(1);
        // Insert node 0 to itself to create a self-loop
        lattice.insert(0, Direction::RIGHT, 0); // Has no effect.
        lattice.validate();
        assert!(format!("{}", lattice).is_empty());
    }

    #[test]
    fn t_print_two_node_connection() {
        let mut lattice = Lattice::new(2);
        lattice.insert(0, Direction::RIGHT, 1);
        lattice.validate();
        assert_eq!(format!("{}", lattice).trim(), "0 - 1");
    }

    #[test]
    fn t_print_triangle_formation() {
        let mut lattice = Lattice::new(3);
        // First connect two nodes
        lattice.insert(0, Direction::RIGHT, 1);
        // Then insert the third node to form a triangle
        lattice.insert(0, Direction::TOP_RIGHT, 2);
        lattice.validate();
        let output = format!("{}", lattice);
        assert!(output.contains(" 0 "));
        assert!(output.contains(" 1 "));
        assert!(output.contains(" 2 "));
        // Should have connections
        assert!(output.contains("-"));
        assert!(output.contains("/") || output.contains("\\"));
    }

    #[test]
    fn t_print_linear_chain() {
        let mut lattice = Lattice::new(4);
        lattice.insert(0, Direction::RIGHT, 1);
        lattice.insert(1, Direction::RIGHT, 2);
        lattice.insert(2, Direction::RIGHT, 3);
        lattice.validate();
        let output = format!("{}", lattice);
        for i in 0..4 {
            assert!(output.contains(&format!(" {} ", i)));
        }
        // Should have multiple horizontal connections
        let dash_count = output.matches("-").count();
        assert!(!output.contains("/") && !output.contains("\\"));
        assert!(dash_count >= 3);
    }

    #[test]
    fn t_print_disjoint_components() {
        let mut lattice = Lattice::new(6);
        // Create first triangle component
        lattice.insert(0, Direction::RIGHT, 1);
        lattice.insert(0, Direction::TOP_RIGHT, 2);
        // Create second linear component
        lattice.insert(3, Direction::RIGHT, 4);
        lattice.insert(4, Direction::RIGHT, 5);
        lattice.validate();
        let output = format!("{}", lattice);
        // All nodes should be present
        for i in 0..6 {
            assert!(output.contains(&format!(" {} ", i)));
        }
        // Should have multiple component separations (empty lines between components)
        let component_separations = output.matches("\n\n").count();
        assert!(component_separations >= 2);
    }

    #[test]
    fn t_print_mixed_components() {
        let mut lattice = Lattice::new(5);
        // Build mixed triangle and linear components
        lattice.insert(0, Direction::RIGHT, 1);
        lattice.insert(0, Direction::TOP_RIGHT, 2);
        // Create a separate small component
        lattice.insert(3, Direction::RIGHT, 4);
        lattice.validate();
        let output = format!("{}", lattice);
        // All nodes should be present
        assert!(output.contains(" 0 "));
        assert!(output.contains(" 1 "));
        assert!(output.contains(" 2 "));
        assert!(output.contains(" 3 "));
        assert!(output.contains(" 4 "));
        // Should have both horizontal and diagonal connections
        assert!(output.contains("-")); // horizontal
        assert!(output.contains("/") || output.contains("\\")); // diagonals
    }

    #[test]
    fn t_print_star_pattern() {
        let mut lattice = Lattice::new(7);
        // Create star pattern with center node connected in all directions
        lattice.insert(0, Direction::RIGHT, 1);
        lattice.insert(0, Direction::TOP_RIGHT, 2);
        lattice.insert(0, Direction::TOP_LEFT, 3);
        lattice.insert(0, Direction::LEFT, 4);
        lattice.insert(0, Direction::BOTTOM_LEFT, 5);
        lattice.insert(0, Direction::BOTTOM_RIGHT, 6);
        lattice.validate();
        let output = format!("{}", lattice);
        // All nodes in star pattern should be present
        for i in 0..7 {
            assert!(output.contains(&format!(" {} ", i)));
        }
        // Should contain all connection types in star pattern
        assert!(output.contains("-")); // horizontal connections
        assert!(output.contains("/")); // diagonal connections
        assert!(output.contains("\\")); // diagonal connections
    }

    #[test]
    fn t_print_non_contiguous_nodes() {
        let mut lattice = Lattice::new(10);
        // Test lattice with gaps in node IDs
        lattice.insert(0, Direction::RIGHT, 2);
        lattice.insert(2, Direction::TOP_RIGHT, 5);
        // Separate component with high node IDs
        lattice.insert(7, Direction::RIGHT, 9);
        lattice.validate();
        let output = format!("{}", lattice);
        // Should contain only the nodes that were actually connected
        assert!(output.contains(" 0 "));
        assert!(output.contains(" 2 "));
        assert!(output.contains(" 5 "));
        assert!(output.contains(" 7 "));
        assert!(output.contains(" 9 "));
        // Should not contain the skipped node IDs
        assert!(!output.contains(" 1 "));
        assert!(!output.contains(" 3 "));
        assert!(!output.contains(" 4 "));
        assert!(!output.contains(" 6 "));
        assert!(!output.contains(" 8 "));
        // Should have component separation
        let component_separations = output.matches("\n\n").count();
        assert!(component_separations >= 2);
    }

    #[test]
    fn t_edges_empty_lattice() {
        let lattice = Lattice::new(5);
        let edges: Vec<_> = lattice.edges().collect();
        assert!(edges.is_empty(), "Empty lattice should have no edges");
    }

    #[test]
    fn t_edges_single_edge() {
        let mut lattice = Lattice::new(2);
        lattice.insert(0, Direction::RIGHT, 1);
        let edges: Vec<_> = lattice.edges().collect();
        assert_eq!(edges.len(), 1, "Single connection should produce one edge");
        assert_eq!(edges[0], (0, 1), "Edge should be (0, 1)");
    }

    #[test]
    fn t_edges_triangle() {
        let mut lattice = Lattice::new(3);
        lattice.insert(0, Direction::RIGHT, 1);
        lattice.insert(0, Direction::TOP_RIGHT, 2);
        let mut edges: Vec<_> = lattice.edges().collect();
        edges.sort();
        assert_eq!(edges.len(), 3, "Triangle should have 3 edges");
        assert_eq!(
            edges,
            vec![(0, 1), (0, 2), (1, 2)],
            "Triangle edges should be (0,1), (0,2), (1,2)"
        );
    }

    #[test]
    fn t_edges_linear_chain() {
        let mut lattice = Lattice::new(4);
        lattice.insert(0, Direction::RIGHT, 1);
        lattice.insert(1, Direction::RIGHT, 2);
        lattice.insert(2, Direction::RIGHT, 3);
        let mut edges: Vec<_> = lattice.edges().collect();
        edges.sort();
        assert_eq!(
            edges.len(),
            3,
            "Linear chain of 4 nodes should have 3 edges"
        );
        assert_eq!(
            edges,
            vec![(0, 1), (1, 2), (2, 3)],
            "Chain edges should be consecutive"
        );
    }

    #[test]
    fn t_edges_disjoint_components() {
        let mut lattice = Lattice::new(6);
        // First component: triangle
        lattice.insert(0, Direction::RIGHT, 1);
        lattice.insert(0, Direction::TOP_RIGHT, 2);
        // Second component: linear pair
        lattice.insert(3, Direction::RIGHT, 4);
        // Third component: single edge
        lattice.insert(5, Direction::RIGHT, 5); // Should have no effect
        let mut edges: Vec<_> = lattice.edges().collect();
        edges.sort();
        assert_eq!(edges.len(), 4, "Two components should have 4 total edges");
        assert_eq!(
            edges,
            vec![(0, 1), (0, 2), (1, 2), (3, 4)],
            "Edges from all components"
        );
    }

    #[test]
    fn t_edges_star_pattern() {
        let mut lattice = Lattice::new(7);
        // Create star pattern with center node 0
        lattice.insert(0, Direction::RIGHT, 1);
        lattice.insert(0, Direction::TOP_RIGHT, 2);
        lattice.insert(0, Direction::TOP_LEFT, 3);
        lattice.insert(0, Direction::LEFT, 4);
        lattice.insert(0, Direction::BOTTOM_LEFT, 5);
        lattice.insert(0, Direction::BOTTOM_RIGHT, 6);
        let mut edges: Vec<_> = lattice.edges().collect();
        edges.sort();
        // The insert function creates additional edges to maintain triangular lattice structure
        // Actual edges created in hexagonal pattern around center node 0
        let expected = vec![
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            (0, 5),
            (0, 6), // center to all
            (1, 2),
            (1, 6),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6), // perimeter connections
        ];
        assert_eq!(
            edges.len(),
            12,
            "Star pattern creates a hexagon with 12 edges"
        );
        assert_eq!(
            edges, expected,
            "Should have center edges plus perimeter connections"
        );
    }

    #[test]
    fn t_edges_no_duplicates() {
        let mut lattice = Lattice::new(3);
        lattice.insert(0, Direction::RIGHT, 1);
        lattice.insert(1, Direction::LEFT, 0); // This should be redundant due to bidirectional nature
        let edges: Vec<_> = lattice.edges().collect();
        assert_eq!(
            edges.len(),
            1,
            "Bidirectional connection should only produce one edge"
        );
        assert_eq!(
            edges[0],
            (0, 1),
            "Edge should be in canonical form (smaller, larger)"
        );
    }

    #[test]
    fn t_edges_canonical_order() {
        let mut lattice = Lattice::new(5);
        // Create simple separate edges to test canonical ordering
        lattice.insert(4, Direction::RIGHT, 1); // Edge (1,4)
        lattice.insert(3, Direction::RIGHT, 0); // Edge (0,3)
        lattice.insert(2, Direction::RIGHT, 2); // Self-connection, should be ignored

        let mut edges: Vec<_> = lattice.edges().collect();
        edges.sort();

        // All edges should be in (smaller, larger) format
        for &(a, b) in &edges {
            assert!(a < b, "Edge ({}, {}) should have smaller ID first", a, b);
        }

        assert_eq!(edges, vec![(0, 3), (1, 4)], "Edges in canonical order");
    }

    #[test]
    fn t_edges_complex_pattern() {
        let mut lattice = Lattice::new(6);
        // Create triangle: 0-1-2
        lattice.insert(0, Direction::RIGHT, 1);
        lattice.insert(0, Direction::TOP_RIGHT, 2);
        // lattice automatically connects 1-2 to complete the triangle

        // Create separate chain: 3-4-5
        lattice.insert(3, Direction::RIGHT, 4);
        lattice.insert(4, Direction::RIGHT, 5);

        let mut edges: Vec<_> = lattice.edges().collect();
        edges.sort();
        let expected = vec![(0, 1), (0, 2), (1, 2), (3, 4), (4, 5)];
        assert_eq!(
            edges.len(),
            expected.len(),
            "Triangle plus chain should have 5 edges"
        );
        assert_eq!(
            edges, expected,
            "Should have triangle edges plus chain edges"
        );
    }

    #[test]
    fn t_edges_after_removal() {
        let mut lattice = Lattice::new(4);
        // Create a linear chain
        lattice.insert(0, Direction::RIGHT, 1);
        lattice.insert(1, Direction::RIGHT, 2);
        lattice.insert(2, Direction::RIGHT, 3);
        // Remove middle node
        lattice.remove(1);
        let mut edges: Vec<_> = lattice.edges().collect();
        edges.sort();
        assert_eq!(
            edges.len(),
            1,
            "After removing middle node, should have 1 edge left"
        );
        assert_eq!(edges[0], (2, 3), "Remaining edge should be (2, 3)");
    }

    #[test]
    fn t_edges_sparse_ids() {
        let mut lattice = Lattice::new(10);
        // Use non-contiguous node IDs with separate components
        lattice.insert(1, Direction::RIGHT, 3); // Component 1: 1-3
        lattice.insert(5, Direction::RIGHT, 7); // Component 2: 5-7
        lattice.insert(8, Direction::RIGHT, 9); // Component 3: 8-9
        let mut edges: Vec<_> = lattice.edges().collect();
        edges.sort();
        assert_eq!(edges.len(), 3, "Three separate edges with sparse IDs");
        assert_eq!(
            edges,
            vec![(1, 3), (5, 7), (8, 9)],
            "Edges should use actual sparse node IDs"
        );
    }
}
