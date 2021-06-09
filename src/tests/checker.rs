use crate::cell::{Side, Working};

#[derive(Debug, Clone, PartialEq)]
pub struct Possible([bool; 2]);
impl Working for Possible {
    type Tile = u8;
    type Rules = ();

    fn new(_rules: &Self::Rules) -> Self {
        Possible([true; 2])
    }
    fn refine(
        &mut self,
        _rules: &Self::Rules,
        _side: Side,
        neighbor: Result<&Self::Tile, &Self>,
    ) -> Result<Self::Tile, bool> {
        let change = match neighbor {
            Ok(val) => std::mem::replace(&mut self.0[*val as usize], false),
            Err(_) => false,
        };
        let count = self.0.iter().filter(|v| **v).count();
        match count {
            0 => Ok(0),
            1 => Ok(self.force_collapse()),
            _ => Err(change),
        }
    }
    fn force_collapse(&self) -> Self::Tile {
        self.0
            .iter()
            .enumerate()
            .filter(|(_i, v)| **v)
            .map(|(i, _)| i as u8)
            .next()
            .unwrap_or(3)
    }
}
