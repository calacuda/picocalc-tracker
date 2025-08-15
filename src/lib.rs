#![no_std]
#![no_main]

extern crate alloc;

use bevy::prelude::*;
pub use picocalc_bevy::hal;

pub mod base_plugin;

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
    Cmd: Clone + Default + PartialEq + PartialOrd,
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

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub enum TrackerCmd<Cmd>
where
    Cmd: Clone + Default + PartialEq + PartialOrd,
{
    #[default]
    None,
    Chord {
        chord: Vec<Intervals>,
    },
    Roll {
        /// how many extra times to "roll" what ever is being played. a value of 1 would produce
        /// two 64th notes.
        times: usize,
    },
    Custom(Cmd),
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct MidiCmd {
    cc_param: u8,
    arg_1: u8,
    arg_2: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Sf2Cmd {
    Atk(usize),
    Dcy(usize),
    Sus(usize),
    Rel(usize),
    Volume(f32),
}

impl Default for Sf2Cmd {
    fn default() -> Self {
        Self::Volume(1.0)
    }
}

#[derive(Clone, Copy, Default, Debug, States, PartialEq, Eq, Hash, Component)]
pub struct TrackID(pub usize);
