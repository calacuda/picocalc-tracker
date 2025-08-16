#![no_std]
#![no_main]

extern crate alloc;

use bevy::prelude::*;
use embedded_alloc::LlffHeap as Heap;
use hal::entry;
// use picocalc_bevy::PicoCalcDefaultPlugins;
pub use picocalc_bevy::hal;
use picocalc_tracker::{CmdPallet, FirstViewTrack, Track, TrackID, base_plugin::BasePlugin};

// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

#[global_allocator]
static HEAP: Heap = Heap::empty();
const HEAP_SIZE: usize = 128 * 1024;

#[entry]
fn main() -> ! {
    init_heap();

    App::new()
        .add_plugins(BasePlugin)
        .insert_resource(CmdPallet(false))
        .init_resource::<FirstViewTrack>()
        .add_systems(Startup, setup)
        .run();

    loop {}
}

fn setup(mut cmds: Commands) {
    // cmds.spawn(TextComponent {
    //     text: "Frames Rendered:".into(),
    //     point: Point::new(10, 10),
    // });

    cmds.spawn((TrackID(0), Track::default()));
    cmds.spawn((TrackID(1), Track::default()));
    cmds.spawn((TrackID(2), Track::default()));
    cmds.spawn((TrackID(3), Track::default()));
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
    hal::binary_info::rp_program_description!(c"PicoCalc-Tracker"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];
