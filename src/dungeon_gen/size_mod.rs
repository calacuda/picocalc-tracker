use bevy::prelude::*;
use rand::{Rng, rngs::SmallRng};

#[derive(Component, Debug)]
pub struct RoomSize {
    pub x: isize,
    pub y: isize,
    pub w: f32,
    pub h: f32,
}

pub fn mk_room_sizes(value: Vec<(isize, isize)>, rng: &mut SmallRng) -> Vec<RoomSize> {
    // let mut rng = rand::rng();

    value
        .into_iter()
        .map(|(x, y)| {
            let w = 12.5 + rng.random_range(-2.5..6.0);
            let h = 12.5 + rng.random_range(-2.5..6.0);

            RoomSize { x, y, w, h }
        })
        .collect()
}

// #[derive(Debug)]
// pub struct SizeModder {
//     rooms: Vec<RoomSize>,
// }

// impl SizeModder {
//     pub fn new(rooms: Vec<RoomSize>) -> Self {
//         Self { rooms }
//     }
// }

// impl Future for SizeModder {
//     type Output = Vec<RoomSize>;
// }
