use std::collections::{HashMap, hash_map::Entry};

enum Error {
    AlreadyOccupied(isize, isize),
}

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

#[derive(Default)]
struct Grid {
    table: HashMap<(isize, isize), usize>,
}

impl Grid {
    fn put(&mut self, (x, y): (isize, isize), item: usize) -> Result<(), Error> {
        match self.table.entry((x, y)) {
            Entry::Occupied(_) => Err(Error::AlreadyOccupied(x, y)),
            Entry::Vacant(slot) => {
                slot.insert(item);
                Ok(())
            }
        }
    }

    fn get(&self, pos: (isize, isize)) -> Option<usize> {
        self.table.get(&pos).copied()
    }

    fn adjacent((x, y): (isize, isize)) -> impl Iterator<Item = (isize, isize)> {
        const OFFSETS: [(isize, isize); 6] = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)];
        OFFSETS.iter().map(move |(xoff, yoff)| (x + xoff, y + yoff))
    }

    fn neighbors(&self, (x, y): (isize, isize)) -> impl Iterator<Item = usize> {
        Self::adjacent((x, y)).filter_map(|(x, y)| self.table.get(&(x, y)).copied())
    }
}

fn main() {}
