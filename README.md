# The Problem

Given a group of people each eith an identical glass, that are trying to clink
the glasses. If the group size is three, they can all clink once in a triangle
pattern, and everyone will have touched everyone else's glass. If the group size
is 4 people, this is not possible with one clink. The best you can do with one
clink is a rhombus pattern, where the two furthest glasses don't touch each
other. So those two have to go in for a second clink. The goal is for everyone
to touch everyone else's glasses. Given a group size, what is the smallest
number of clinks required for everyone's glass to touch everyone else's?

The problem can be described another way. Imagine a complete graph with `n`
nodes, where `n` is the number of people in the group. Each edge represents a
clink of glasses by two people. We need to decompose this complete graph into
the minimal number of subgraphs, that can be embedded in a planar triangular
lattice. It's a planar triangular lattice, because that's the best we can do
when clinking glasses. There is existing research on minimal decomposition of a
graph into planar sub-graphs. But in this case, it is more specific. They have
to be triangular lattices, and not just planar.
