use crate::cell::Working;
use std::mem::forget;
use std::mem::MaybeUninit;
use std::{borrow::Borrow, ops::Range};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
#[cfg(feature = "serde")]
#[derive(Debug, Deserialize, Serialize)]
pub struct WorldSave<W, T, R> {
    pub rules: R,
    pub cells: Vec<((i32, i32), Result<T, W>)>,
    pub bounding: Range<[i32; 2]>
}

#[derive(Debug, Clone)]
pub struct World<W: Working> {
    rules: W::Rules,
    base: W,
    tiles: Twolayer<W::Tile, W>,
    bounding: Bounding,
}
impl<W, const N: usize> World<W>
where
    W: Working<Grabber = [(i32, i32); N]>,
{
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
        self.propagate(x, y)
    }
    fn refine(&mut self, x: i32, y: i32) {
        if !self.bounding.contains(x, y) {
            return;
        }
        let neighbors = match self.grab(W::NEIGHBORS, x, y) {
            Some(ns) => ns,
            None => return,
        };
        let mut tile = match self.read(x, y) {
            Ok(_) => return,
            Err(w) => w.clone(),
        };
        let change = match tile.refine(&neighbors, &self.rules) {
            Ok(t) => {
                self.tiles.insert_p(x, y, t);
                true
            }
            Err(true) => {
                self.tiles.insert_s(x, y, tile);
                true
            }
            Err(false) => false,
        };
        if change {
            self.propagate(x, y);
        }
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
                self.refine(x, y);
            }
        }
    }
    fn propagate(&mut self, x: i32, y: i32) {
        let ns = W::NEIGHBORS;
        let slc: &[_] = ns.borrow();
        for (x, y) in slc.iter().map(|(a, b)| (a + x, b + y)) {
            self.refine(x, y);
        }
    }
    pub fn base(&self) -> &W {
        &self.base
    }
    pub(crate) fn try_read(&self, x: i32, y: i32) -> Option<Result<&W::Tile, &W>> {
        self.tiles.get(x, y)
    }
    pub fn read(&self, x: i32, y: i32) -> Result<&W::Tile, &W> {
        self.tiles.get(x, y).unwrap_or(Err(&self.base))
    }
    fn grab(&self, grabber: [(i32, i32); N], x: i32, y: i32) -> Option<[Result<&W::Tile, &W>; N]> {
        let mut try_grab: [Option<Result<&W::Tile, &W>>; N] = none_array();
        for ((x, y), dest) in grabber
            .iter()
            .map(|&(a, b)| (x + a, y + b))
            .zip(&mut try_grab)
        {
            *dest = self.try_read(x, y);
        }
        if try_grab.iter().all(|o| o.is_none()) {
            return None;
        }
        let base = self.base();
        Some(arr_map(try_grab, |o| o.unwrap_or(Err(base))))
    }
}

#[cfg(feature = "serde")]
impl<W: Working> World<W> {
    pub fn get_save(&self) -> WorldSave<W, W::Tile, W::Rules>
    where
        W::Tile: Clone,
        W::Rules: Clone,
    {
        WorldSave {
            rules: self.rules.clone(),
            cells: self
                .tiles
                .primary
                .0
                .iter()
                .map(|((x, y), t)| ((*x, *y), Ok(t.clone())))
                .chain(
                    self.tiles
                        .secondary
                        .0
                        .iter()
                        .map(|((x, y), w)| ((*x, *y), Err(w.clone()))),
                )
                .collect(),
            bounding: self.bounding.0.clone()
        }
    }
}

#[cfg(feature = "serde")]
impl<W: Working> From<WorldSave<W, W::Tile, W::Rules>> for World<W> {
    fn from(save: WorldSave<W, W::Tile, W::Rules>) -> Self {
        let mut tiles = Twolayer::new();
        let rules = save.rules;
        for ((x, y), tile) in save.cells {
            match tile {
                Ok(t) => tiles.insert_p(x, y, t),
                Err(w) => tiles.insert_s(x, y, w)
            }
        }
        let bounding = Bounding(save.bounding);
        let base = W::new(&rules);
        Self {
            tiles,
            rules,
            bounding,
            base
        }
    }
}

fn none_array<T, const N: usize>() -> [Option<T>; N] {
    let mut unin: MaybeUninit<[Option<T>; N]> = MaybeUninit::uninit();
    let ptr = unin.as_mut_ptr() as *mut Option<T>;
    for i in 0..N {
        unsafe { ptr.add(i).write(None) }
    }
    unsafe { unin.assume_init() }
}

fn arr_map<A, B, F: FnMut(A) -> B, const N: usize>(a: [A; N], mut f: F) -> [B; N] {
    let mut b = MaybeUninit::uninit();
    let ptr = b.as_mut_ptr() as *mut B;
    for (i, aitem) in a.iter().enumerate() {
        let aptr = aitem as *const A;
        let bptr = unsafe { ptr.add(i) };
        unsafe { bptr.write(f(aptr.read())) };
    }
    forget(a);
    unsafe { b.assume_init() }
}
