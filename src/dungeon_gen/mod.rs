use crate::InGameState;
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};
use grid_maker::GridMaker;
use size_mod::mk_room_sizes;

pub mod grid_maker;
pub mod size_mod;

pub const MAX_ROOMS: usize = 20;
pub const MIN_ROOMS: usize = 12;

#[derive(Component)]
struct GenLevelTask(pub usize, pub Task<Vec<(isize, isize)>>);

#[derive(Component)]
struct Level;

pub struct DungeonGen;

impl Plugin for DungeonGen {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InGameState::LevelGen), start_level_gen)
            .add_systems(
                Update,
                wait_for_level.run_if(in_state(InGameState::LevelGen)),
            );
    }
}

fn start_level_gen(mut cmds: Commands, levels: Query<&Level>) {
    let thread_pool = AsyncComputeTaskPool::get();

    let entity = cmds.spawn_empty().id();
    let task = thread_pool.spawn(GridMaker::new(MIN_ROOMS, MAX_ROOMS));

    info!("Starting level generation");

    // Spawn new entity and add our new task as a component
    cmds.entity(entity)
        .insert(GenLevelTask(levels.iter().len(), task));
}

fn wait_for_level(mut cmds: Commands, mut transform_tasks: Query<(&mut GenLevelTask, Entity)>) {
    // info!("wait_for_level 1");
    for mut task in &mut transform_tasks {
        // info!("wait_for_level 2");
        if let Some(room_locs) = block_on(future::poll_once(&mut task.0.1)) {
            // append the returned command queue to have it execute later
            // cmds.append(&mut commands_queue.lock().unwrap());

            info!("despawning rooms generation task");
            cmds.entity(task.1).despawn_recursive();
            info!("rooms generated");
            let mut rooms = mk_room_sizes(room_locs);
            info!("rooms sized");
            let max_w = rooms
                .iter()
                .max_by(|size_1, size_2| size_1.w.partial_cmp(&size_2.w).unwrap())
                .unwrap()
                .w;
            let max_h = rooms
                .iter()
                .max_by(|size_1, size_2| size_1.h.partial_cmp(&size_2.h).unwrap())
                .unwrap()
                .h;
            let min_x = rooms
                .iter()
                .min_by(|size_1, size_2| size_1.x.partial_cmp(&size_2.x).unwrap())
                .unwrap()
                .x;
            let min_y = rooms
                .iter()
                .min_by(|size_1, size_2| size_1.y.partial_cmp(&size_2.y).unwrap())
                .unwrap()
                .y;
            rooms.iter_mut().for_each(|size| {
                if min_x <= 0 {
                    size.x += min_x.abs();
                } else {
                    size.x -= min_x;
                }

                if min_y <= 0 {
                    size.y += min_y.abs();
                } else {
                    size.y -= min_y;
                }
            });
            // rooms.
            // TODO: Calculate an X/Y position of the rooms
            // TODO: draw a straight line from the middle of one to the middle of the next.
            for room_measurements in rooms {}

            // let thread_pool = AsyncComputeTaskPool::get();
            //
            // let entity = cmds.spawn_empty().id();
            // let task = thread_pool.spawn(SizeModder::new(rooms));
            //
            // info!("modifying the size of the rooms");
            //
            // // Spawn new entity and add our new task as a component
            // cmds.entity(entity).insert(task);
        } else {
            info!("processing");
        }
    }
}
