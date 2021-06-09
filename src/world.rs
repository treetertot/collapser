use crate::cell::{Side, Working};
use std::ops::Range;

#[derive(Debug, Clone)]
struct Tagged<T>(Vec<((i32, i32), T)>);
impl<T> Tagged<T> {
    fn search(&self, x: i32, y: i32) -> Result<usize, usize> {
        self.0.binary_search_by_key(&(x, y), |(tag, _d)| *tag)
    }
    fn get(&self, x: i32, y: i32) -> Option<&T> {
        let idx = self.search(x, y).ok()?;
        Some(&self.0[idx].1)
    }
    fn insert(&mut self, x: i32, y: i32, data: T) {
        match self.search(x, y) {
            Ok(i) => self.0[i].1 = data,
            Err(i) => self.0.insert(i, ((x, y), data)),
        }
    }
}

#[test]
fn tag_test() {
    let mut tagged = Tagged(Vec::new());
    tagged.insert(0, 0, 0u8);
    tagged.insert(0, 1, 1u8);
    tagged.insert(1, 0, 2u8);
    assert_eq!(tagged.get(0, 1), Some(&1))
}

#[derive(Debug, Clone)]
struct Twolayer<P, S> {
    primary: Tagged<P>,
    secondary: Tagged<S>,
}
impl<P, S> Twolayer<P, S> {
    fn new() -> Self {
        Twolayer {
            primary: Tagged(Vec::new()),
            secondary: Tagged(Vec::new()),
        }
    }
    fn get(&self, x: i32, y: i32) -> Option<Result<&P, &S>> {
        match self.primary.get(x, y) {
            Some(p) => Some(Ok(p)),
            None => self.secondary.get(x, y).map(|s| Err(s)),
        }
    }
    fn insert_p(&mut self, x: i32, y: i32, data: P) {
        self.rm_s(x, y);
        self.primary.insert(x, y, data);
    }
    fn insert_s(&mut self, x: i32, y: i32, data: S) {
        if self.primary.search(x, y).is_err() {
            self.secondary.insert(x, y, data);
        }
    }
    fn rm_s(&mut self, x: i32, y: i32) {
        if let Ok(idx) = self.secondary.search(x, y) {
            self.secondary.0.remove(idx);
        }
    }
}

#[test]
fn twolayer_test() {
    let mut layers = Twolayer::new();
    layers.insert_s(0, 1, 1);
    layers.insert_p(0, 1, 1);
    layers.insert_s(1, 0, 2);
    assert_eq!(layers.get(0, 1), Some(Ok(&1)));
    assert_eq!(layers.get(1, 0), Some(Err(&2)));
}

#[derive(Debug, Clone)]
struct Bounding(Range<[i32; 2]>);
impl Bounding {
    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.0.start[0] && x < self.0.end[0] && y >= self.0.start[1] && y < self.0.end[1]
    }
    fn cells(&self) -> impl Iterator<Item = (i32, i32)> {
        let ys = self.0.start[1]..self.0.end[1];
        (self.0.start[0]..self.0.end[0])
            .map(move |x| ys.clone().map(move |y| (x, y)))
            .flatten()
    }
}

#[derive(Debug, Clone)]
pub struct World<W: Working> {
    rules: W::Rules,
    base: W,
    tiles: Twolayer<W::Tile, W>,
    bounding: Bounding,
}
impl<W: Working> World<W> {
    /// Bounding limits which tiles will be actively updated
    pub fn new(rules: W::Rules, bounding: Range<[i32; 2]>) -> Self {
        let base = W::new(&rules);
        let bounding = Bounding(bounding);
        let tiles = Twolayer::new();
        World {
            rules,
            base,
            tiles,
            bounding,
        }
    }
    /// force collapses the tile at that location
    pub fn collapse(&mut self, x: i32, y: i32) {
        if !self.bounding.contains(x, y) {
            return;
        }
        let tile = match self.tiles.get(x, y) {
            Some(Err(w)) => w.force_collapse(),
            None => self.base.force_collapse(),
            _ => return,
        };
        self.tiles.insert_p(x, y, tile);
        self.refine(x, y + 1);
        self.refine(x, y - 1);
        self.refine(x + 1, y);
        self.refine(x - 1, y);
    }
    /// Changes the bounding and updates the newly awakened tiles
    pub fn set_bounding(&mut self, bounding: Range<[i32; 2]>) {
        if bounding != self.bounding.0 {
            let old_iter = self.bounding.clone();
            self.bounding = Bounding(bounding);
            let iter = self
                .bounding
                .cells()
                .filter(|&(x, y)| !old_iter.contains(x, y));
            for (x, y) in iter {
                if self.has_neighbors(x, y) {
                    self.refine(x, y);
                }
            }
        }
    }
    fn refine(&mut self, x: i32, y: i32) {
        if !self.bounding.contains(x, y) {
            return;
        }
        let mut tile = match self.tiles.get(x, y) {
            Some(Err(t)) => t.clone(),
            None => self.base.clone(),
            _ => return,
        };
        let mut change = false;
        match tile.refine(&self.rules, Side::Top, self.read(x, y + 1)) {
            Ok(t) => {
                self.tiles.insert_p(x, y, t);
                change = true;
            }
            Err(diff) => change = change || diff,
        }
        match tile.refine(&self.rules, Side::Bottom, self.read(x, y - 1)) {
            Ok(t) => {
                self.tiles.insert_p(x, y, t);
                change = true;
            }
            Err(diff) => change = change || diff,
        }
        match tile.refine(&self.rules, Side::Right, self.read(x + 1, y)) {
            Ok(t) => {
                self.tiles.insert_p(x, y, t);
                change = true;
            }
            Err(diff) => change = change || diff,
        }
        match tile.refine(&self.rules, Side::Left, self.read(x - 1, y)) {
            Ok(t) => {
                self.tiles.insert_p(x, y, t);
                change = true;
            }
            Err(diff) => change = change || diff,
        }
        if change {
            self.tiles.insert_s(x, y, tile);
            self.refine(x, y + 1);
            self.refine(x, y - 1);
            self.refine(x + 1, y);
            self.refine(x - 1, y);
        }
    }
    fn has_neighbors(&self, x: i32, y: i32) -> bool {
        // fix so if only some are none, it behaves like
        self.tiles.get(x, y + 1).is_some()
            || self.tiles.get(x, y - 1).is_some()
            || self.tiles.get(x + 1, y).is_some()
            || self.tiles.get(x - 1, y).is_some()
    }
    pub fn read(&self, x: i32, y: i32) -> Result<&W::Tile, &W> {
        self.tiles.get(x, y).unwrap_or(Err(&self.base))
    }
}
