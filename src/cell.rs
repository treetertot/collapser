use crate::world::World;

pub trait Working: Sized {
    type Tile;
    type Rules;

    /// Narrows possibilities based on other tiles.
    /// For best performance:
    /// only return with changes if certain there were changes (may crash/freeze if you return unecessarily) and
    /// get neighbors with try_read to make sure they exist before cloning self (less important)
    fn refine(
        &self,
        x: i32,
        y: i32,
        rules: &Self::Rules,
        world: &World<Self>,
    ) -> Result<Self::Tile, Option<Self>>;
    /// Collapses the tile to a random value
    fn force_collapse(&self) -> Self::Tile;
    /// Creates the base tile from rules
    fn new(rules: &Self::Rules) -> Self;
}
