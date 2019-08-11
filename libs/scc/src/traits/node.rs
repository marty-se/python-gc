use std::cmp::Eq;
use std::hash::Hash;

pub trait Node: Eq + Hash + Clone + FindAdjacent {}

/// This trait can be implemented for any struct which is logically part of a (directed) graph.
/// `find_adjacent()` should return the nodes pointed to by outgoing edges from the current node.
/// There may be duplicate edges (that is, the same node may be present multiple times in the result.
/// Also, there may edges pointing to the current node itself.
pub trait FindAdjacent
where
    Self: Sized,
{
    fn find_adjacent(&self) -> Vec<Self>;
}
