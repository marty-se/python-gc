mod traits;
mod scc;

pub use crate::scc::{SCCCollector, SCCIterator};
pub use crate::traits::node::FindAdjacent;