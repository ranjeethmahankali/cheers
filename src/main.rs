use std::{
    collections::{HashMap, hash_map::Entry},
    fmt::Display,
};

#[derive(Debug)]
enum Error {
    AlreadyOccupied(isize, isize),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AlreadyOccupied(x, y) => write!(f, "Position ({}, {}) is already occupied", x, y),
        }
    }
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

    fn neighbors(
        &self,
        (x, y): (isize, isize),
    ) -> impl Iterator<Item = (isize, isize, Option<usize>)> {
        const OFFSETS: [(isize, isize); 6] = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)];
        OFFSETS
            .iter()
            .map(move |(xoff, yoff)| (x + xoff, y + yoff))
            .map(|(x, y)| (x, y, self.table.get(&(x, y)).copied()))
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (xmin, _ymin, _xmax, _ymax) = self.table.keys().fold(
            (isize::MAX, isize::MAX, isize::MIN, isize::MIN),
            |(xmin, ymin, xmax, ymax), (x, y)| {
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
        let nodes = {
            let mut nodes: Vec<(isize, isize, usize)> = self
                .table
                .iter()
                .map(|((x, y), item)| (*x, *y, *item))
                .collect();
            nodes.sort_by(|(a, b, _), (c, d, _)| {
                let a = a + b;
                let c = c + d;
                (std::cmp::Reverse(b), a).cmp(&(std::cmp::Reverse(d), c))
            });
            nodes
        };
        /* Neighbor indices.

             2   1
              \ /
           3 - * - 0
              / \
             4   5
        */
        for row in nodes.chunk_by(|(_ix1, iy1, _item1), (_ix2, iy2, _item2)| iy1 == iy2) {
            let mut xoff = 0usize;
            for (ix, iy, item) in row {
                let next = self.neighbors((*ix, *iy)).next();
                let x = ((ix * 4 + 2 * iy) - xmin) as usize;
                for _ in 0..(x - xoff) {
                    write!(f, " ")?;
                }
                write!(
                    f,
                    "{:^3}{}",
                    item,
                    if let Some((_nx, _ny, Some(_nb))) = next {
                        "-"
                    } else {
                        " "
                    }
                )?;
                xoff = x + 4;
            }
            writeln!(f, "")?;
            // Print the downlinks.
            xoff = 0usize;
            for (ix, iy, _item) in row {
                let mut nbs = self
                    .neighbors((*ix, *iy))
                    .skip(4)
                    .map(|(_, _, item)| item.is_some());
                let x = ((ix * 4 + 2 * iy) - xmin) as usize;
                for _ in 0..(x - xoff) {
                    write!(f, " ")?;
                }
                let left = nbs.next().unwrap_or(false);
                let right = nbs.next().unwrap_or(false);
                write!(
                    f,
                    "{} {} ",
                    if left { "/" } else { " " },
                    if right { "\\" } else { " " }
                )?;
                xoff = x + 4;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let mut grid = Grid::default();
    grid.put((0, 0), 0)?;
    grid.put((0, 1), 1)?;
    grid.put((1, 0), 2)?;
    grid.put((1, 1), 3)?;
    grid.put((0, 2), 4)?;
    grid.put((-1, 2), 5)?;
    grid.put((2, 1), 6)?;
    grid.put((2, 2), 7)?;
    grid.put((1, 2), 8)?;
    println!("\n{}\n", grid);
    Ok(())
}
