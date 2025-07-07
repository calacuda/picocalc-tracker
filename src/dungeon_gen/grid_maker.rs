use bevy::prelude::*;
use rand::{Rng, SeedableRng, rngs::SmallRng};
// use std::{ops::Deref, sync::Mutex, task::Poll};
use defmt::*;

#[derive(Debug)]
pub struct GridMaker {
    n_rooms: usize,
    // cmd_queue: Arc<Mutex<CommandQueue>>,
    // rooms: Vec<RoomAt>,
    gen_rooms: usize,
    cursor: (isize, isize),
    generated: Vec<(isize, isize)>,
    // rng: Arc<Mutex<ThreadRng>>,
    pub rng: SmallRng,
}

impl GridMaker {
    pub fn new(min: usize, max: usize, seed: u32) -> Self {
        let mut rng = SmallRng::seed_from_u64(seed as u64);

        Self {
            n_rooms: rng.random_range(min..max),
            // cmd_queue: Arc::new(Mutex::new(CommandQueue::default())),
            gen_rooms: 0,
            cursor: (0, 0),
            generated: vec![(0, 0)],
            rng, // rng: Arc::new(Mutex::new(rng)),
        }
    }
}

impl GridMaker {
    // type Output = Vec<(isize, isize)>;

    pub fn step(self: &mut Self) -> Option<Vec<(isize, isize)>> {
        // let mut gen_rooms = self.gen_rooms.lock().unwrap();
        // let mut generated = self.generated.lock().unwrap();
        // let mut cursor = self.cursor.lock().unwrap();
        // let mut rng = rand::rng();
        // let mut cursor = &mut self.cursor;
        let rng = &mut self.rng;

        while self.generated.contains(&self.cursor) {
            // up, down, left, or right
            let n = rng.random_range(0..4);

            // set cursor acourdingly
            if n == 0 {
                // up
                self.cursor.0 += 1;
            } else if n == 1 {
                // down
                self.cursor.0 -= 1;
            } else if n == 2 {
                // left
                self.cursor.1 -= 1;
            } else if n == 3 {
                // right
                self.cursor.1 += 1;
            }
        }

        self.generated.push(self.cursor.clone());
        self.gen_rooms += 1;

        if self.gen_rooms == self.n_rooms {
            Some(self.generated.clone())
        } else {
            info!("generated {}/{} rooms", self.gen_rooms, self.n_rooms);
            // cx.waker().clone().wake();
            // Poll::Pending
            None
        }
    }
}
