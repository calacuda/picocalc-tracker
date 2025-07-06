use bevy::prelude::*;
use rand::Rng;
use std::{ops::Deref, sync::Mutex, task::Poll};

#[derive(Debug)]
pub struct GridMaker {
    n_rooms: usize,
    // cmd_queue: Arc<Mutex<CommandQueue>>,
    // rooms: Vec<RoomAt>,
    gen_rooms: Mutex<usize>,
    cursor: Mutex<(isize, isize)>,
    generated: Mutex<Vec<(isize, isize)>>,
    // rng: Arc<Mutex<ThreadRng>>,
}

impl GridMaker {
    pub fn new(min: usize, max: usize) -> Self {
        let mut rng = rand::rng();

        Self {
            n_rooms: rng.random_range(min..max),
            // cmd_queue: Arc::new(Mutex::new(CommandQueue::default())),
            gen_rooms: Mutex::new(0),
            cursor: Mutex::new((0, 0)),
            generated: Mutex::new(vec![(0, 0)]),
            // rng: Arc::new(Mutex::new(rng)),
        }
    }
}

impl Future for GridMaker {
    type Output = Vec<(isize, isize)>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut gen_rooms = self.gen_rooms.lock().unwrap();
        let mut generated = self.generated.lock().unwrap();
        let mut cursor = self.cursor.lock().unwrap();
        let mut rng = rand::rng();

        while generated.contains(&cursor) {
            // up, down, left, or right
            let n = rng.random_range(0..4);

            // set cursor acourdingly
            if n == 0 {
                // up
                (*cursor).0 += 1;
            } else if n == 1 {
                // down
                (*cursor).0 -= 1;
            } else if n == 2 {
                // left
                (*cursor).1 -= 1;
            } else if n == 3 {
                // right
                (*cursor).1 += 1;
            }
        }

        generated.push(cursor.deref().clone());
        *gen_rooms += 1;

        if *gen_rooms == self.n_rooms {
            Poll::Ready(generated.clone())
        } else {
            info!("generated {}/{} rooms", gen_rooms, self.n_rooms);
            cx.waker().clone().wake();
            Poll::Pending
        }
    }
}
