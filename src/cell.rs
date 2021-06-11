use crate::world::World;

use std::mem::MaybeUninit;
use std::borrow::Borrow;
use std::mem::forget;

pub trait Working<const N: usize>: Sized + Clone {
    type Tile;
    type Rules;
    /// Put an array of (i32, i32) here with a length of your choice
    type Grabber: Grabber<N>;

    /// Put a list of offsets from the current cell to get as neighbors
    const NEIGHBORS: Self::Grabber;

    /// Narrows possibilities based on other tiles.
    /// For best performance:
    /// only return with changes if certain there were changes (may crash/freeze if you return unecessarily) and
    /// get neighbors with try_read to make sure they exist before cloning self (less important)
    fn refine(
        &mut self,
        neighbors: [Result<&Self::Tile, &Self>; N],
        rules: &Self::Rules
    ) -> Result<Self::Tile, bool>;
    /// Collapses the tile to a random value
    fn force_collapse(&self) -> Self::Tile;
    /// Creates the base tile from rules
    fn new(rules: &Self::Rules) -> Self;
}

pub trait Grabber<const N: usize>: Borrow<[(i32, i32)]> {
    fn grab<'a, W: Working<N>>(&self, x: i32, y: i32, world: &'a World<W, N>) -> Option<[Result<&'a W::Tile, &'a W>; N]> {
        let mut try_grab: [Option<Result<&W::Tile, &W>>; N] = none_array();
        let slc: &[(i32, i32)] = self.borrow();
        for ((x, y), dest) in slc.iter().map(|&(a, b)| (x+a, y+b)).zip(&mut try_grab) {
            *dest = world.try_read(x, y);
        }
        if try_grab.iter().all(|o| o.is_none()) {
            return None;
        }
        let base = world.base();
        Some(arr_map(try_grab, |o| o.unwrap_or(Err(base))))
    }
}
impl<const N: usize> Grabber<N> for [(i32, i32); N] {
}

fn none_array<T, const N: usize>() -> [Option<T>; N] {
    let mut unin: MaybeUninit<[Option<T>; N]> = MaybeUninit::uninit();
    let ptr = unin.as_mut_ptr() as *mut Option<T>;
    for i in 0..N {
        unsafe{ptr.add(i).write(None)}
    }
    unsafe{unin.assume_init()}
}

fn arr_map<A, B, F: FnMut(A) -> B, const N: usize>(a: [A; N], mut f: F) -> [B; N] {
    let mut b = MaybeUninit::uninit();
    let ptr = b.as_mut_ptr() as *mut B;
    for (i, aitem) in a.iter().enumerate() {
        let aptr = aitem as *const A;
        let bptr = unsafe{ptr.add(i)};
        unsafe{bptr.write(f(aptr.read()))};
    }
    forget(a);
    unsafe{b.assume_init()}
}