pub trait Working<const N: usize>: Sized + Clone {
    type Tile;
    type Rules;

    /// An Array of offsets from the current cell to get as neighbors
    const NEIGHBORS: [(i32, i32); N];

    /// Narrows possibilities based on other tiles.
    /// For best performance:
    /// only return with changes if certain there were changes (may crash/freeze if you return unecessarily) and
    /// get neighbors with try_read to make sure they exist before cloning self (less important)
    fn refine(
        &mut self,
        neighbors: [Result<&Self::Tile, &Self>; N],
        rules: &Self::Rules
    ) -> Result<Self::Tile, bool>;
    /// Collapses the tile to a random value
    fn force_collapse(&self) -> Self::Tile;
    /// Creates the base tile from rules
    fn new(rules: &Self::Rules) -> Self;
}
