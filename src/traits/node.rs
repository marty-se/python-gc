pub type NodeHashType = u64;

pub trait FindAdjacent
where
    Self: Sized,
{
    fn find_adjacent(&self) -> Vec<Self>;
}

pub trait NodeHash {
    fn node_hash(&self) -> NodeHashType;
}

