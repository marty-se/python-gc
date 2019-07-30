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

pub struct SCCCollector<T> {
    root_entry: T,
}

impl<T: Hash + Eq + Clone + FindAdjacent> SCCCollector<T> {
    pub fn new(root_entry: T) -> Self {
        Self {
            root_entry: root_entry,
        }
    }

    pub fn iter(&self) -> SCCIterator<T> {
        SCCIterator::new(self.root_entry.clone())
    }
}

pub struct SCCIterator<T: Hash + Eq + Clone> {
    traversal_stack: Vec<T>,
    scc_stack: Vec<T>,
    nodes: HashMap<T, NodeState<T>>,
    index_counter: u64,
}

impl<T: Hash + Eq + Clone> SCCIterator<T> {
    fn new(root_entry: T) -> Self {
        let mut scc_state = SCCIterator {
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
        let node_state = NodeState::new(entry_ref.clone(), self.next_index());

        self.traversal_stack.push(entry_ref.clone());

        self.nodes.insert(entry, node_state);
    }

    fn next_index(&mut self) -> u64 {
        let next_index = self.index_counter;
        self.index_counter = self.index_counter + 1;
        next_index
    }

    fn handle_found_entry(&mut self, entry: T) {
        if self.nodes.get(&entry).is_none() {
            self.add_entry(entry);
        }
    }
}

impl<'p, T: FindAdjacent + Hash + Eq + Clone> Iterator for SCCIterator<T> {
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
                    let current_entry_ref = &current_entry;

                    self.scc_stack.push(current_entry_ref.clone());
                    current_node.on_scc_stack = true;

                    self.traversal_stack.push(current_entry);

                    let mut adjacent = entry_data.find_adjacent();
                    adjacent.reverse();
                    for entry in adjacent {
                        self.handle_found_entry(entry);
                    }
                }
                TraversalState::Traversed => {
                    let entry_data = &current_node.data;
                    let mut updated_lowlink = current_node.lowlink;
                    let mut adjacent = entry_data.find_adjacent();
                    adjacent.reverse();
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
                                let mut cur_scc_node = self.nodes.get_mut(&cur_scc_entry).unwrap();
                                cur_scc_node.on_scc_stack = false;
                                let mut done = false;
                                if cur_scc_entry == current_entry {
                                    done = true;
                                }
                                scc_nodes.push(cur_scc_entry);
                                if done {
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
