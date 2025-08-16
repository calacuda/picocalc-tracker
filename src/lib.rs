#![no_std]
#![no_main]

extern crate alloc;

use core::fmt::Display;

use crate::helpers::less_then::UsizeLessThan;
use bevy::prelude::*;
use strum_macros::{Display, EnumString};

pub use picocalc_bevy::hal;

pub mod base_plugin;
pub mod helpers;

pub type MidiNote = u8;

pub const SCREEN_W: usize = 320;
pub const SCREEN_H: usize = 320;
pub const N_STEPS: usize = 32;

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
    cmds: (TrackerCmd<Cmd>, TrackerCmd<Cmd>),
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
    // NOTE: maybe remove
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
    #[strum(to_string = "Sus-")]
    Sus(usize),
    #[strum(to_string = "Rel-")]
    Rel(usize),
    #[strum(to_string = "Vol-")]
    Volume(f32),
}

// impl Display for Sf2Cmd {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         write!(f, self.to)
//     }
// }

impl Default for Sf2Cmd {
    fn default() -> Self {
        Self::Volume(1.0)
    }
}

#[derive(Clone, Copy, Default, Debug, States, PartialEq, Eq, Hash, Component)]
pub struct TrackID(pub usize);

#[derive(Clone, Copy, Default, Debug, States, PartialEq, Eq, Hash, Resource, Deref, DerefMut)]
pub struct FirstViewTrack(pub usize);
