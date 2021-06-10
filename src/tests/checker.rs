use crate::{cell::Working, world::World};

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
        x: i32,
        y: i32,
        world: &World<Self>,
    ) -> Result<Self::Tile, bool> {
        let neighbors = [
            world.read(x, y + 1),
            world.read(x, y - 1),
            world.read(x + 1, y),
            world.read(x - 1, y),
        ];
        if neighbors.iter().all(|n| n.is_err()) {
            return Err(false);
        }
        let mut change = false;
        for &neighbor in neighbors.iter().filter_map(|r| r.ok()) {
            change = change || std::mem::replace(&mut self.0[neighbor as usize], false);
        }
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
