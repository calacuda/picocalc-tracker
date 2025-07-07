use bevy::prelude::*;

use crate::{InGameState, MainGameState};

pub struct AutoTransition;

impl Plugin for AutoTransition {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, to_in_game.run_if(in_state(MainGameState::StartUp)))
            .add_systems(Update, to_expanding.run_if(in_state(MainGameState::InGame)));
    }
}

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
