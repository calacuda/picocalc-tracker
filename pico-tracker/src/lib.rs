#![no_std]
#![no_main]

extern crate alloc;

use cortex_m_semihosting::debug;

// use defmt_rtt as _; // global logger

use crate::helpers::less_then::UsizeLessThan;
use bevy::prelude::*;
use core::fmt::Display;
use strum_macros::{Display, EnumString};

use panic_probe as _;

#[cfg(not(all(test, target_arch = "x86_64")))]
pub use picocalc_bevy::hal;

#[cfg(not(all(test, target_arch = "x86_64")))]
pub mod base_plugin;
#[cfg(not(all(test, target_arch = "x86_64")))]
pub mod embedded;
pub mod helpers;

pub type MidiNote = u8;

pub const SCREEN_W: usize = 320;
pub const SCREEN_H: usize = 320;
pub const N_STEPS: usize = 32;
pub const CHAR_W: usize = 40;
pub const CHAR_H: usize = 24;
pub const Y_OFFSET: i32 = 11;
pub const COL_W: usize = 18;
pub const CHAR_PIX_W: i32 = 8;
pub const CHAR_PIX_H: i32 = 13;

// // same panicking *behavior* as `panic-probe` but doesn't print a panic message
// // this prevents the panic message being printed *twice* when `defmt::panic` is invoked
// #[defmt::panic_handler]
// fn panic() -> ! {
//     cortex_m::asm::udf()
// }

/// Terminates the application and makes a semihosting-capable debug tool exit
/// with status code 0.
pub fn exit() -> ! {
    loop {
        debug::exit(debug::EXIT_SUCCESS);
    }
}

/// Hardfault handler.
///
/// Terminates the application and makes a semihosting-capable debug tool exit
/// with an error. This seems better than the default, which is to spin in a
/// loop.
#[cortex_m_rt::exception]
unsafe fn HardFault(_frame: &cortex_m_rt::ExceptionFrame) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE);
    }
}

#[derive(Clone, Copy, Default, Debug, States, PartialEq, Eq, Hash)]
pub enum MainState {
    #[default]
    StartUp,
    Edit,
    ShutDown,
}

#[derive(Clone, Copy, Default, Debug, States, PartialEq, Eq, Hash, Resource, Deref, DerefMut)]
pub struct CmdPallet(pub bool);

#[derive(Clone, Debug, Component, PartialEq, PartialOrd)]
pub enum Track {
    Midi { steps: Vec<Step<MidiCmd>> },
    SF2 { steps: Vec<Step<Sf2Cmd>> },
}

impl Default for Track {
    fn default() -> Self {
        Self::Midi {
            steps: (0..N_STEPS).map(|_| Step::default()).collect(),
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct Step<Cmd>
where
    Cmd:
        Clone + Default + PartialEq + PartialOrd + core::fmt::Display + ToString + core::fmt::Debug,
{
    pub note: Option<MidiNote>,
    pub cmds: (TrackerCmd<Cmd>, TrackerCmd<Cmd>),
}

#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd, Eq, Hash)]
pub enum Intervals {
    #[default]
    Root,
    MajThird,
    MinThird,
    FlatFifth,
    Fifth,
    SharpFifth,
    FlatSeventh,
    Seventh,
    SharpSeventh,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Hash, EnumString, Display)]
pub enum TrackerCmd<Cmd>
where
    Cmd: Clone + Default + PartialEq + PartialOrd + ToString + Display,
{
    #[default]
    #[strum(to_string = "----")]
    None,
    #[strum(to_string = "Chrd")]
    Chord { chord: Vec<Intervals> },
    #[strum(to_string = "Roll")]
    Roll {
        /// how many extra times to "roll" what ever is being played. a value of 1 would produce
        /// two 64th notes.
        times: usize,
    },
    // NOTE: maybe remove Swing
    #[strum(to_string = "Swng")]
    Swing {
        /// the amount of swing to put on the note
        amt: UsizeLessThan<128>,
    },
    #[strum(to_string = "Hold")]
    HoldFor {
        notes: UsizeLessThan<{ N_STEPS + 1 }>,
    },
    /// stop all notes on device
    #[strum(to_string = "Stop")]
    Panic,
    #[strum(transparent)]
    Custom(Cmd),
}

// TODO: impl Display for TrackerCmd

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct MidiCmd {
    cc_param: u8,
    arg_1: u8,
    arg_2: u8,
}

impl Display for MidiCmd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CC--")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, EnumString, strum_macros::Display)]
pub enum Sf2Cmd {
    #[strum(to_string = "Atk-")]
    Atk(usize),
    #[strum(to_string = "Dcy-")]
    Dcy(usize),
    #[strum(to_string = "Dcy2")]
    Dcy2(usize),
    #[strum(to_string = "Sus-")]
    Sus(usize),
    #[strum(to_string = "Rel-")]
    Rel(usize),
    #[strum(to_string = "Vol-")]
    Volume(f32),
}

impl Default for Sf2Cmd {
    fn default() -> Self {
        Self::Volume(1.0)
    }
}

#[derive(Clone, Copy, Default, Debug, States, PartialEq, Eq, Hash, Component)]
pub struct TrackID(pub usize);

#[derive(Clone, Copy, Default, Debug, States, PartialEq, Eq, Hash, Resource, Deref, DerefMut)]
pub struct FirstViewTrack(pub usize);

pub fn row_from_line(row_i: usize) -> i32 {
    Y_OFFSET + (CHAR_PIX_H * row_i as i32)
}

pub fn x_from_col(col_i: usize) -> i32 {
    CHAR_PIX_W * col_i as i32
}

pub fn display_midi_note(midi_note: MidiNote) -> String {
    let note_name_i = midi_note % 12;
    let octave = midi_note / 12;

    let note_names = [
        "C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-", "B#",
    ];
    let note_name = note_names[note_name_i as usize];

    format!("{note_name}{octave:X}")
}

// #[cfg(all(test, target_arch = "x86_64"))]
// #[macro_use]

// defmt-test 0.3.0 has the limitation that this `#[tests]` attribute can only be used
// once within a crate. the module can be in any file but there can only be at most
// one `#[tests]` module in this library crate
#[cfg(test)]
#[defmt_test::tests]
mod test {
    // extern crate std;
    use crate::*;
    use defmt::assert_eq;
    use embedded_alloc::LlffHeap as Heap;

    #[global_allocator]
    static HEAP: Heap = Heap::empty();

    #[test]
    fn tracker_cmd_display() {
        assert_eq!(
            TrackerCmd::<Sf2Cmd>::None.to_string().as_str(),
            "----",
            "{} != None",
            TrackerCmd::<Sf2Cmd>::None.to_string().as_str()
        );

        assert_eq!(
            TrackerCmd::<Sf2Cmd>::Custom(Sf2Cmd::Volume(0.5))
                .to_string()
                .as_str(),
            "Vol-",
            "{} != None",
            TrackerCmd::<Sf2Cmd>::Custom(Sf2Cmd::Volume(0.5))
                .to_string()
                .as_str()
        )
    }
}
