use std::cmp::min;
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::vec::Vec;

use crate::traits::node::*;

enum TraversalState {
    Initial,
    Traversed,
}

struct NodeState<T> {
    data: T,
    traversal_state: TraversalState,
    on_scc_stack: bool,
    index: u64,
    lowlink: u64,
}

impl<T> NodeState<T> {
    fn new(data: T, index: u64) -> Self {
        NodeState {
            data: data,
            traversal_state: TraversalState::Initial,
            on_scc_stack: false,
            index: index,
            lowlink: index,
        }
    }
}

pub struct SCCCollector<T: Hash + Eq> {
    traversal_stack: Vec<T>,
    scc_stack: Vec<T>,
    nodes: HashMap<T, NodeState<T>>,
    index_counter: u64,
}

impl<T: Hash + Eq + Clone> SCCCollector<T> {
    pub fn new(root_entry: T) -> Self {
        let mut scc_state = SCCCollector {
            traversal_stack: Vec::new(),
            scc_stack: Vec::new(),
            nodes: HashMap::new(),
            index_counter: 0,
        };
        scc_state.add_entry(root_entry);
        scc_state
    }

    fn add_entry(&mut self, entry: T) {
        let entry_ref = &entry;
        self.add_to_traversal_stack(entry_ref.clone());
        self.add_to_scc_stack(entry_ref.clone());
        let node_state = NodeState::new(entry_ref.clone(), self.next_index());

        self.nodes.insert(entry, node_state);
    }

    fn next_index(&mut self) -> u64 {
        let next_index = self.index_counter;
        self.index_counter = self.index_counter + 1;
        next_index
    }

    fn add_to_traversal_stack(&mut self, entry: T) {
        self.traversal_stack.push(entry);
    }

    fn add_to_scc_stack(&mut self, entry: T) {
        let entry_clone = &entry.clone();
        self.scc_stack.push(entry);
        let mut node_state = self.nodes.get_mut(&entry_clone).unwrap();
        node_state.on_scc_stack = true;
    }

    fn handle_found_entry(&mut self, entry: T) {
        if self.nodes.get(&entry).is_none() {
            self.add_entry(entry);
        }
    }
}

impl<'p, T: FindAdjacent + Hash + Eq + Clone> Iterator for SCCCollector<T> {
    type Item = Vec<T>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current_entry) = self.traversal_stack.pop() {
            let current_node = self
                .nodes
                .get_mut(&current_entry)
                .expect("Traversal stack inconsistent");
            match &current_node.traversal_state {
                TraversalState::Initial => {
                    let entry_data = &current_node.data;
                    current_node.traversal_state = TraversalState::Traversed;
                    self.traversal_stack.push(current_entry);

                    let adjacent = entry_data.find_adjacent();
                    for entry in adjacent {
                        self.handle_found_entry(entry);
                    }
                }
                TraversalState::Traversed => {
                    let entry_data = &current_node.data;
                    let adjacent = entry_data.find_adjacent();
                    let mut updated_lowlink = current_node.lowlink;
                    for entry in adjacent {
                        let node_state = self.nodes.get(&entry).unwrap();
                        if node_state.on_scc_stack {
                            updated_lowlink = min(updated_lowlink, node_state.lowlink);
                        }
                    }

                    let mut_current_node = self
                        .nodes
                        .get_mut(&current_entry)
                        .expect("Traversal stack inconsistent");
                    mut_current_node.lowlink = updated_lowlink;

                    let reborrowed_current_node = self
                        .nodes
                        .get(&current_entry)
                        .expect("Traversal stack inconsistent");

                    if reborrowed_current_node.lowlink == reborrowed_current_node.index {
                        let mut scc_nodes = Vec::<T>::new();
                        loop {
                            if let Some(cur_scc_entry) = self.scc_stack.pop() {
                                let cur_scc_node = self.nodes.remove(&cur_scc_entry).unwrap();
                                scc_nodes.push(cur_scc_node.data);
                                if cur_scc_entry == current_entry {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        return Some(scc_nodes);
                    }
                }
            }
        }

        None
    }
}
