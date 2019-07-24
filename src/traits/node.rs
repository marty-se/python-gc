pub trait FindAdjacent
where
    Self: Sized,
{
    fn find_adjacent(&self) -> Vec<Self>;
}
