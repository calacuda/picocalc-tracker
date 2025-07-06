#![no_std]
#![no_main]

extern crate alloc;

use bevy::prelude::*;
use embedded_alloc::LlffHeap as Heap;
use hal::entry;
use nalgebra::{ComplexField, Point3, Vector3};
use picocalc_bevy::{PicoCalcDefaultPlugins, hal};

// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

#[derive(Clone, Copy, Default, Debug, States, PartialEq, Eq, Hash)]
pub enum MainGameState {
    #[default]
    StartUp,
    StartScreen,
    SettingsScreen,
    InGame,
}

#[derive(Clone, Copy, Default, Debug, SubStates, PartialEq, Eq, Hash)]
#[source(MainGameState = MainGameState::InGame)]
pub enum InGameState {
    #[default]
    NotInGame,
    Normal,
    LevelGen,
}

// #[derive(Clone, Copy, Default, Debug, States, PartialEq, Eq, Hash)]
// // #[source(InGameState = InGameState::LevelGen)]
// pub enum LevelGenState {
//     #[default]
//     NotGen,
//     /// randommly expanding left, right, up, or down.
//     Expanding,
//     /// makes some rooms bigger or smaller, or funky shapes
//     ModingRoomGeom,
//     /// move the rooms closser, and waits for them to lock in position.
//     WaitingForLock,
//     /// distribute loot chests, monster spawns, and other prefabricated structures.
//     SprinklePreFab,
// }

fn to_in_game(mut game_state: ResMut<NextState<MainGameState>>) {
    game_state.set(MainGameState::InGame)
}

fn to_expanding(
    mut in_game_state: ResMut<NextState<InGameState>>,
    // mut level_gen_state: ResMut<NextState<LevelGenState>>,
) {
    in_game_state.set(InGameState::LevelGen);
    // level_gen_state.set(LevelGenState::Expanding);
}

#[global_allocator]
static HEAP: Heap = Heap::empty();
const HEAP_SIZE: usize = 128 * 1024;

mod dungeon_gen;

#[entry]
fn main() -> ! {
    init_heap();

    let pos = Point3::new(0.0, 2.0, 0.0);
    let looking_at = pos + Vector3::new(0.0_f32.cos(), 0.0_f32.sin(), 0.0_f32.sin());

    App::new()
        .add_plugins(PicoCalcDefaultPlugins)
        .insert_resource(DoubleBufferRes::new(PlayerLocation {
            pos,
            looking_at,
            ..default()
        }))
        .insert_resource(Engine3d::new(320, 320))
        .add_systems(Startup, (clear_display, setup))
        .add_systems(Update, to_in_game.run_if(in_state(MainGameState::StartUp)))
        .add_systems(Update, to_expanding.run_if(in_state(MainGameState::InGame)))
        .run();

    loop {}
}

#[allow(static_mut_refs)]
fn init_heap() {
    use core::mem::MaybeUninit;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
}

/// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"Bevy test-3"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];
