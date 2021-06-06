#[derive(Debug, Clone)]
pub enum Collapseable<T, S> {
    Superimposed(S),
    Collapsed(T),
}
impl<T, S> Collapseable<T, S> {
    pub fn superimposed(&self) -> Option<&S> {
        match self {
            Collapseable::Superimposed(s) => Some(s),
            _ => None,
        }
    }
    pub fn borrow_collapse(&self) -> Collapseable<&T, &S> {
        match self {
            Collapseable::Collapsed(t) => Collapseable::Collapsed(t),
            Collapseable::Superimposed(s) => Collapseable::Superimposed(s),
        }
    }
}
impl<T, S> Default for Collapseable<T, S>
where
    S: Default,
{
    fn default() -> Self {
        Collapseable::Superimposed(S::default())
    }
}

pub struct Neighbors<T> {
    pub top: T,
    pub left: T,
    pub bottom: T,
    pub right: T,
}

pub trait Superposition: Clone {
    type Tile;
    type Rules;

    /// Modifies self possibilities based on its neighbor. Returns with the collapsed result or if it changed
    fn refine(
        &mut self,
        sides: Neighbors<Collapseable<&Self::Tile, &Self>>,
        rules: &Self::Rules,
    ) -> Result<Self::Tile, bool>;

    /// Removes a random possibility
    fn reduce(&mut self);
}
