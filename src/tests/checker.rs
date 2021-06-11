use crate::cell::Working;

#[derive(Debug, Clone, PartialEq)]
pub struct Possible([bool; 2]);
impl Working<4> for Possible {
    type Tile = u8;
    type Rules = ();

    const NEIGHBORS: [(i32, i32); 4] = [(-1, 0), (0, -1), (0, 1), (1, 0)];

    fn new(_rules: &Self::Rules) -> Self {
        Possible([true; 2])
    }
    fn refine(&mut self, neighbors: [Result<&Self::Tile, &Self>; 4], _rules: &Self::Rules) -> Result<Self::Tile, bool> {
        let mut change = false;
        for &val in neighbors.iter().filter_map(|r| r.ok()) {
            change = change || std::mem::replace(&mut self.0[val as usize], false);
        }
        let iter = self.0.iter().filter(|b| **b);
        match iter.count() {
            0 | 1 => Ok(self.force_collapse()),
            _ => Err(change)
        }
    }
    fn force_collapse(&self) -> Self::Tile {
        self.0
            .iter()
            .enumerate()
            .filter(|(_i, v)| **v)
            .map(|(i, _)| i as u8)
            .next()
            .unwrap_or(2)
    }
}
