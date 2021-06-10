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
    }
}

pub mod cell;
pub mod world;
