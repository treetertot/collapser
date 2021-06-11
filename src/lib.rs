#[cfg(test)]
mod tests {
    mod checker;
    use crate::world::World;
    use checker::Possible;
    #[test]
    fn checkertest() {
        let size = 10;
        let mut world: World<Possible> = World::new((), [0, 0]..[size, size]);
        for i in 0..size {
            for j in 0..size {
                world.collapse(i, j);
            }
        }
        assert_ne!(world.read(0, 0), world.read(0, 1));
        assert_eq!(world.read(0, 0), Ok(&0));
        assert_eq!(world.read(0, 1), Ok(&1));
    }
}
///Collapser is a library for writing custom tile generators similar to WFC.
/// The library focuses on the Working trait. To see how to implement it, I reccomend looking at the checker test.
/// (You'll need to go to the github for that).
pub mod cell;
pub mod world;
