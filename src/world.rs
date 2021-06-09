use std::ops::Range;

use crate::cell::{Collapseable, Neighbors, Superposition};
use arrayvec::ArrayVec;

pub struct TileWorld<S: Superposition> {
    base: S,
    rules: S::Rules,
    collapsed: TagList<S::Tile>,
    superimposed: TagList<S>,
    allocated: [Range<i32>; 2],
}
impl<S: Superposition> TileWorld<S> {
    pub fn new(base: S, rules: S::Rules, allocated: [Range<i32>; 2]) -> Self {
        TileWorld {
            base,
            rules,
            allocated,
            collapsed: TagList(Vec::new()),
            superimposed: TagList(Vec::new()),
        }
    }
    pub fn read(&self, addr: (i32, i32)) -> Collapseable<&S::Tile, &S> {
        match self.collapsed.bin_search(&addr) {
            Ok(i) => Collapseable::Collapsed(&self.collapsed.0[i].data),
            Err(_) => match self.superimposed.bin_search(&addr) {
                Ok(i) => Collapseable::Superimposed(&self.superimposed.0[i].data),
                Err(_) => Collapseable::Superimposed(&self.base),
            },
        }
    }
    fn get_sup_mut(&mut self, addr: (i32, i32)) -> Result<&mut S, Option<S>> {
        match self.superimposed.bin_search(&addr) {
            Ok(i) => Ok(&mut self.superimposed.0[i].data),
            Err(_) => match self.collapsed.bin_search(&addr) {
                Ok(_) => Err(None),
                Err(_) => Err(Some(self.base.clone())),
            },
        }
    }
    fn neighbors(&self, (x, y): (i32, i32)) -> Neighbors<Collapseable<&S::Tile, &S>> {
        Neighbors {
            top: self.read((x, y + 1)),
            bottom: self.read((x, y - 1)),
            left: self.read((x - 1, y)),
            right: self.read((x + 1, y)),
        }
    }
    fn center_ns(
        &mut self,
        addr: (i32, i32),
    ) -> Option<(
        Result<&mut S, S>,
        Neighbors<Collapseable<&S::Tile, &S>>,
        &S::Rules,
    )> {
        match unsafe { &mut *(self as *mut Self) }.get_sup_mut(addr) {
            Ok(s) => Some((Ok(s), self.neighbors(addr), &self.rules)),
            Err(Some(s)) => Some((Err(s), self.neighbors(addr), &self.rules)),
            Err(None) => None,
        }
    }
    pub fn set_active(&mut self, allocated: [Range<i32>; 2]) {
        if self.allocated != allocated {
            self.allocated = allocated;
            self.refine();
        }
    }
    // returns the changed
    fn refine_cycle<I: IntoIterator<Item = (i32, i32)>>(&mut self, to_check: I) -> Vec<(i32, i32)> {
        let to_check = to_check.into_iter();
        let mut changed = Vec::with_capacity(to_check.size_hint().0);
        for coords in to_check {
            match self.center_ns(coords) {
                Some((s, ns, rules)) => match s {
                    Ok(s) => match s.refine(ns, rules) {
                        Ok(tile) => {
                            self.collapsed.insert(&coords, tile);
                            match self.superimposed.bin_search(&coords) {
                                Ok(i) => {
                                    self.superimposed.0.remove(i);
                                }
                                Err(_i) => (),
                            }
                            changed.push(coords);
                        }
                        Err(true) => changed.push(coords),
                        Err(false) => (),
                    },
                    Err(mut s) => match s.refine(ns, rules) {
                        Ok(tile) => {
                            self.collapsed.insert(&coords, tile);
                            changed.push(coords);
                        }
                        Err(true) => {
                            self.superimposed.insert(&coords, s);
                            changed.push(coords)
                        }
                        Err(false) => (),
                    },
                },
                None => (),
            }
        }
        changed
    }
    pub fn refine(&mut self) {
        let to_check: Vec<_> = self
            .superimposed
            .0
            .iter()
            .map(|t| t.tag)
            .filter(|t| self.contains(*t))
            .collect();
        self.refine_start(to_check);
    }
    fn refine_start(&mut self, mut to_check: Vec<(i32, i32)>) {
        while to_check.len() > 0 {
            let changed = self.refine_cycle(to_check);
            to_check = changed
                .into_iter()
                .map(|(x, y)| ArrayVec::from([(x + 1, y), (x, y + 1), (x, y - 1), (x - 1, y)]))
                .flatten()
                .collect();
            to_check.sort_unstable();
            to_check.dedup();
            to_check.retain(|f| self.contains(*f));
        }
    }
    fn contains(&self, addr: (i32, i32)) -> bool {
        self.allocated[0].start <= addr.0
            && addr.0 <= self.allocated[0].end
            && self.allocated[1].start <= addr.1
            && addr.1 <= self.allocated[1].end
    }
    /// returns true if reduction sucessful
    pub fn reduce(&mut self, addr: (i32, i32)) -> bool {
        let (x, y) = addr;
        match self.superimposed.bin_search(&addr) {
            Ok(i) => {
                self.superimposed.0[i].data.reduce();
                // Add refine that starts on area instead of refining everything
                self.refine_start(vec![(x + 1, y), (x, y + 1), (x, y - 1), (x - 1, y)]);
                true
            }
            Err(_) => match self.collapsed.bin_search(&addr) {
                Ok(_) => false,
                Err(i) => {
                    let mut tile = self.base.clone();
                    // update array refine result
                    match tile.refine(self.neighbors(addr), &self.rules) {
                        Ok(tile) => {
                            self.collapsed.0.insert(
                                i,
                                Tagged {
                                    tag: addr,
                                    data: tile,
                                },
                            );
                            self.refine_start(vec![(x + 1, y), (x, y + 1), (x, y - 1), (x - 1, y)]);
                        }
                        Err(true) => {
                            self.superimposed.insert(&addr, tile);
                            self.refine_start(vec![(x + 1, y), (x, y + 1), (x, y - 1), (x - 1, y)]);
                        }
                        Err(false) => (),
                    }
                    true
                }
            },
        }
    }
}

struct Tagged<T> {
    tag: (i32, i32),
    data: T,
}
struct TagList<T>(Vec<Tagged<T>>);
impl<T> TagList<T> {
    fn bin_search(&self, tag: &(i32, i32)) -> Result<usize, usize> {
        self.0.binary_search_by_key(tag, |f| f.tag)
    }
    fn insert(&mut self, tag: &(i32, i32), data: T) {
        match self.bin_search(tag) {
            Ok(i) => self.0[i].data = data,
            Err(i) => self.0.insert(i, Tagged { tag: *tag, data }),
        }
    }
}
