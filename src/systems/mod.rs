mod chasing;
mod combat;
mod end_turn;
mod entity_render;
mod fov;
mod hud;
mod map_render;
mod movement;
mod player_input;
mod random_move;
mod tooltip;
mod use_items;
use crate::prelude::*;

pub fn build_input_scheduler() -> Schedule {
    // NOTE: is fov::fov_system() only usefull on the first turn of input schedule??
    // other turns will have it refreshed before render on player and monster turn.

    Schedule::builder()
        .add_system(player_input::player_input_system())
        .add_system(fov::fov_system())
        .flush()
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(hud::hud_system())
        .add_system(tooltip::tooltip_system())
        .build()
}

pub fn build_turn_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(player_input::player_input_system())
        .flush()
        .add_system(chasing::chasing_system())
        .add_system(random_move::random_move_system())
        .flush()
        .add_system(use_items::use_items_system())
        .add_system(combat::combat_system())
        .flush()
        .add_system(movement::movement_system())
        .flush()
        .add_system(fov::fov_system())
        .flush()
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(hud::hud_system())
        .add_system(tooltip::tooltip_system())
        .add_system(end_turn::end_turn_system())
        .build()
}
