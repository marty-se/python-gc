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

#[cfg(test)]
mod tests {
    use super::FindAdjacent;
    use super::Hash;
    use super::HashMap;
    use super::SCCCollector;
    use std::cell::RefCell;
    use std::cmp::Ordering;

    #[derive(Debug)]
    struct Graph {
        edges: HashMap<u64, Vec<u64>>,
        num_visits: RefCell<HashMap<u64, u64>>,
    }

    impl Graph {
        fn new() -> Self {
            Graph {
                edges: HashMap::new(),
                num_visits: RefCell::default(),
            }
        }

        fn add_edge(&mut self, src: u64, dst: u64) {
            if let Some(edges_vec) = self.edges.get_mut(&src) {
                edges_vec.push(dst);
            } else {
                self.edges.insert(src, vec![dst]);
            }
        }
    }

    #[derive(Clone, Debug)]
    struct Entry<'g> {
        index: u64,
        graph: &'g Graph,
    }

    impl<'g> Ord for Entry<'g> {
        fn cmp(&self, other: &Self) -> Ordering {
            self.index.cmp(&other.index)
        }
    }

    impl<'g> PartialOrd for Entry<'g> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl<'g> Entry<'g> {
        fn new(graph: &'g Graph, index: u64) -> Self {
            Self {
                index: index,
                graph: graph,
            }
        }
    }

    impl<'g> PartialEq for Entry<'g> {
        fn eq(&self, other: &Self) -> bool {
            self.index == other.index
        }
    }

    impl<'g> Eq for Entry<'g> {}

    impl<'g> Hash for Entry<'g> {
        fn hash<H>(&self, state: &mut H)
        where
            H: std::hash::Hasher,
        {
            self.index.hash(state)
        }
    }

    impl<'g> FindAdjacent for Entry<'g> {
        fn find_adjacent(&self) -> Vec<Self> {
            let mut num_visits_map = self.graph.num_visits.borrow_mut();
            if let Some(num_visits) = num_visits_map.get_mut(&self.index) {
                *num_visits = *num_visits + 1;
            } else {
                num_visits_map.insert(self.index, 1);
            }

            if let Some(edges) = self.graph.edges.get(&self.index) {
                edges
                    .iter()
                    .map(|index| Self {
                        index: *index,
                        graph: self.graph,
                    })
                    .collect()
            } else {
                Vec::new()
            }
        }
    }

    fn get_sorted_sccs<'g>(root_entry: Entry<'g>) -> Vec<Vec<Entry<'g>>> {
        let scc_collector = SCCCollector::new(root_entry);
        let mut result: Vec<Vec<Entry>> = scc_collector
            .iter()
            .map(|mut v| {
                v.sort();
                v
            })
            .collect();
        result.sort();
        result
    }

    #[test]
    fn test_single_node_with_cycle() {
        let mut graph = Graph::new();
        graph.add_edge(1, 1);

        let result = get_sorted_sccs(Entry::new(&graph, 1));

        assert_eq!(result, vec![vec![Entry::new(&graph, 1)]]);
    }

    #[test]
    fn test_simple_cycle() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 1);

        let result = get_sorted_sccs(Entry::new(&graph, 1));

        assert_eq!(
            result,
            vec![vec![Entry::new(&graph, 1), Entry::new(&graph, 2)]]
        );
    }

    #[test]
    fn test_simple_non_cycle() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);

        let result = get_sorted_sccs(Entry::new(&graph, 1));

        assert_eq!(
            result,
            vec![vec![Entry::new(&graph, 1)], vec![Entry::new(&graph, 2)]]
        );
    }

    #[test]
    fn test_single_node_and_cycle() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 2);

        let result = get_sorted_sccs(Entry::new(&graph, 1));

        assert_eq!(
            result,
            vec![
                vec![Entry::new(&graph, 1)],
                vec![Entry::new(&graph, 2), Entry::new(&graph, 3)]
            ]
        );
    }

    #[test]
    fn test_cycle_and_single_node() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 1);
        graph.add_edge(2, 3);

        let result = get_sorted_sccs(Entry::new(&graph, 1));

        assert_eq!(
            result,
            vec![
                vec![Entry::new(&graph, 1), Entry::new(&graph, 2)],
                vec![Entry::new(&graph, 3)]
            ]
        );
    }

    #[test]
    fn test_dual_cycle() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 1);

        graph.add_edge(2, 3);

        graph.add_edge(3, 4);
        graph.add_edge(4, 3);

        let result = get_sorted_sccs(Entry::new(&graph, 1));

        assert_eq!(
            result,
            vec![
                vec![Entry::new(&graph, 1), Entry::new(&graph, 2)],
                vec![Entry::new(&graph, 3), Entry::new(&graph, 4)]
            ]
        );
    }

    #[test]
    fn test_cycle_with_dual_edges_to_node() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 1);

        graph.add_edge(1, 3);
        graph.add_edge(2, 3);

        let result = get_sorted_sccs(Entry::new(&graph, 1));

        assert_eq!(
            result,
            vec![
                vec![Entry::new(&graph, 1), Entry::new(&graph, 2)],
                vec![Entry::new(&graph, 3)]
            ]
        );
    }

    #[test]
    fn test_fully_connected_graph() {
        let mut graph = Graph::new();
        const NUM_NODES: u64 = 100;
        for i in 1..NUM_NODES {
            for j in 1..NUM_NODES {
                graph.add_edge(i, j);
            }
        }

        let result = get_sorted_sccs(Entry::new(&graph, 1));

        let mut expected = Vec::new();
        for i in 1..NUM_NODES {
            expected.push(Entry::new(&graph, i));
        }

        assert_eq!(result, vec![expected]);
    }

    #[test]
    fn test_large_acyclic_graph() {
        let mut graph = Graph::new();
        const NUM_NODES: u64 = 1000000;
        for i in 1..NUM_NODES {
            graph.add_edge(i, i + 1);
        }

        let result = get_sorted_sccs(Entry::new(&graph, 1));

        let mut expected = Vec::new();
        for i in 1..NUM_NODES + 1 {
            expected.push(vec![Entry::new(&graph, i)]);
        }

        assert_eq!(result, expected);
    }

    #[test]
    fn test_does_not_traverse_entire_graph_when_fetching_single_scc() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 2);

        graph.add_edge(1, 4);

        let scc_collector = SCCCollector::new(Entry::new(&graph, 1));
        let result: Vec<Vec<Entry>> = scc_collector
            .iter()
            .map(|mut v| {
                v.sort();
                v
            })
            .take(1)
            .collect();

        assert_eq!(
            result,
            vec![vec![Entry::new(&graph, 2), Entry::new(&graph, 3)],]
        );

        // The 4 node should never be reached when only a single SCC is fetched from the iterator.
        assert!(graph.num_visits.borrow().get(&4).is_none());
    }
}
