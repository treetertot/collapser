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
        &self,
        x: i32,
        y: i32,
        _rules: &Self::Rules,
        world: &World<Self>,
    ) -> Result<Self::Tile, Option<Self>> {
        let neighbors = [
            world.read(x, y + 1),
            world.read(x, y - 1),
            world.read(x + 1, y),
            world.read(x - 1, y),
        ];
        if neighbors.iter().all(|r| r.is_err()) {
            return Err(None);
        }
        let mut change = false;
        let mut me = self.clone();
        let iter = neighbors.iter().filter_map(|r| r.ok());
        for &i in iter {
            change = change || std::mem::replace(&mut me.0[i as usize], false);
        }
        match change {
            true => Err(Some(me)),
            false => Err(None),
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
