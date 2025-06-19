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

fn main() {}
