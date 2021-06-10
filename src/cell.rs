use crate::world::World;

pub trait Working: Sized + Clone {
    type Tile;
    type Rules;

    /// Eliminates the impossibilities given a side
    fn refine(
        &mut self,
        rules: &Self::Rules,
        x: i32,
        y: i32,
        world: &World<Self>,
    ) -> Result<Self::Tile, bool>;
    /// Collapses the tile to a random value
    fn force_collapse(&self) -> Self::Tile;
    /// Creates the base tile from rules
    fn new(rules: &Self::Rules) -> Self;
}
