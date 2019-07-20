use std::cmp::min;
use std::collections::HashMap;
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

impl<T: NodeHash> NodeHash for NodeState<T> {
    fn node_hash(&self) -> u64 {
        self.data.node_hash()
    }
}

struct SCCTraversalState<T: NodeHash> {
    traversal_stack: Vec<NodeHashType>,
    scc_stack: Vec<NodeHashType>,
    nodes: HashMap<NodeHashType, NodeState<T>>,
    index_counter: u64,
}

impl<T: NodeHash> SCCTraversalState<T> {
    pub fn new(root_entry: T) -> Self {
        let mut scc_state = SCCTraversalState {
            traversal_stack: Vec::new(),
            scc_stack: Vec::new(),
            nodes: HashMap::new(),
            index_counter: 0,
        };
        let root_node_hash = scc_state.add_entry(root_entry);
        scc_state.add_to_traversal_stack(root_node_hash);
        scc_state
    }

    fn next_index(&mut self) -> u64 {
        let next_index = self.index_counter;
        self.index_counter = self.index_counter + 1;
        next_index
    }

    fn add_to_traversal_stack(&mut self, node_hash: NodeHashType) {
        self.traversal_stack.push(node_hash);
    }

    fn add_to_scc_stack(&mut self, node_hash: NodeHashType) {
        self.scc_stack.push(node_hash);
        let mut node_state = self.nodes.get_mut(&node_hash).unwrap();
        node_state.on_scc_stack = true;
    }

    fn add_node(&mut self, node_state: NodeState<T>) -> NodeHashType {
        let node_hash = node_state.node_hash();
        self.nodes.insert(node_hash, node_state);
        node_hash
    }

    fn add_entry(&mut self, entry: T) -> NodeHashType {
        let node_state = NodeState::new(entry, self.next_index());
        let node_hash = node_state.node_hash();
        self.add_node(node_state);
        self.add_to_traversal_stack(node_hash);
        self.add_to_scc_stack(node_hash);
        node_hash
    }

    fn handle_found_entry(&mut self, entry: T) -> &NodeState<T> {
        let node_hash = entry.node_hash();
        if self.get_node_state_for_entry(&entry).is_none() {
            self.add_entry(entry);
        }
        self.nodes.get(&node_hash).unwrap()
    }

    fn get_node_state_for_entry(&self, entry: &T) -> Option<&NodeState<T>> {
        let node_hash = entry.node_hash();
        self.nodes.get(&node_hash)
    }
}

impl<'p, T: FindAdjacent + NodeHash> Iterator for SCCTraversalState<T> {
    type Item = Vec<T>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current_hash) = self.traversal_stack.pop() {
            let current_node = self
                .nodes
                .get_mut(&current_hash)
                .expect("Traversal stack inconsistent");
            match &current_node.traversal_state {
                TraversalState::Initial => {
                    let entry_data = &current_node.data;
                    current_node.traversal_state = TraversalState::Traversed;
                    self.traversal_stack.push(current_hash);

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
                        let node_state = self.get_node_state_for_entry(&entry).unwrap();
                        if node_state.on_scc_stack {
                            updated_lowlink = min(updated_lowlink, node_state.lowlink);
                        }
                    }

                    let mut_current_node = self
                        .nodes
                        .get_mut(&current_hash)
                        .expect("Traversal stack inconsistent");
                    mut_current_node.lowlink = updated_lowlink;

                    let reborrowed_current_node = self
                        .nodes
                        .get(&current_hash)
                        .expect("Traversal stack inconsistent");

                    if reborrowed_current_node.lowlink == reborrowed_current_node.index {
                        let mut scc_nodes = Vec::<T>::new();
                        loop {
                            if let Some(cur_scc_node_hash) = self.scc_stack.pop() {
                                let cur_scc_node = self.nodes.remove(&cur_scc_node_hash).unwrap();
                                scc_nodes.push(cur_scc_node.data);
                                if cur_scc_node_hash == current_hash {
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
