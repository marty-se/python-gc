use scc::FindAdjacent;
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;

pub struct RefCounter {}

impl RefCounter {
    /// Counts internal references between a set of nodes.
    /// The returned vec is of equal size to the provided `nodes` vec, where each position
    /// contains the number of internal references to the corresponding input node.
    pub fn count_internal_refs<T: Eq + Hash + FindAdjacent>(nodes: Vec<T>) -> Vec<u64> {
        let mut ref_map: HashMap<T, u64> = Default::default();

        for node in &nodes {
            for adjacent in node.find_adjacent() {
                if let Some(count) = ref_map.get_mut(&adjacent) {
                    *count += 1;
                } else {
                    ref_map.insert(adjacent, 1);
                }
            }
        }

        nodes
            .iter()
            .map(|node| {
                if let Some(internal_refs_for_node) = ref_map.get(&node) {
                    *internal_refs_for_node
                } else {
                    0u64
                }
            })
            .collect()
    }
}
