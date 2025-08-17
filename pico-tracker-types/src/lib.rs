#![no_std]

pub extern crate alloc;

pub use ron;

use alloc::{string::String, vec::Vec};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Event)]
pub enum FromHost {
    /// a list of known devices
    Devs { dev_names: Vec<String> },
    /// a message was sent on the message bus
    MessageBus { message: String },
    /// a midi note was played on a pre-configured midi controller
    MidiNoteOn { note: u8, vel: u8, channel: u8 },
    /// a midi note was released on a pre-configured midi controller
    MidiNoteOff { note: u8, channel: u8 },
    /// a midi CC param was sent on a pre-configured midi controller
    MidiCC { control: u8, param: u8, channel: u8 },
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Event)]
pub enum FromTracker {
    /// Log a message to the Hosts terminal
    Log { message: String },
    /// Request an updated list of device names from the host.
    RequestDevs,
    /// send a message to the message-bus
    MessageBus { message: String },
    /// instructs the host to send messages that match this message.
    ListenFor { message: String },
}
