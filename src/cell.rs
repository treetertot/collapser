use std::borrow::Borrow;

pub trait Working: Sized + Clone {
    type Tile;
    type Rules;
    /// The type of Self::NEIGHBORS
    /// Only usable if fixed size array
    type Grabber: Borrow<[(i32, i32)]>;

    /// An Array of offsets from the current cell to get as neighbors
    const NEIGHBORS: Self::Grabber;

    /// Narrows possibilities based on other tiles.
    /// Neighbors is guaranteed to be the same length as Self::NEIGHBORS.
    /// Const generics just aren't at a place where I can include that in the type
    /// For best performance
    /// only return with changes if certain there were changes (may crash/freeze if you return unecessarily)
    fn refine(
        &mut self,
        neighbors: &[Result<&Self::Tile, &Self>],
        rules: &Self::Rules,
    ) -> Result<Self::Tile, bool>;
    /// Collapses the tile to a random value
    fn force_collapse(&self) -> Self::Tile;
    /// Creates the base tile from rules
    fn new(rules: &Self::Rules) -> Self;
}
