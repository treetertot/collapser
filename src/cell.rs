pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

pub trait Working: Sized + Clone {
    type Tile;
    type Rules;

    /// Eliminates the impossibilities given a side
    fn refine(
        &mut self,
        rules: &Self::Rules,
        side: Side,
        neighbor: Result<&Self::Tile, &Self>,
    ) -> Result<Self::Tile, bool>;
    /// Collapses the tile to a random value
    fn force_collapse(&self) -> Self::Tile;
    /// Creates the base tile from rules
    fn new(rules: &Self::Rules) -> Self;
}
