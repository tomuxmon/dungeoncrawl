use super::MapArchitect;
use crate::prelude::*;

const NUM_ROOMS: usize = 20;

pub struct RoomsArchitect {}

impl MapArchitect for RoomsArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder {
        let mut mb = MapBuilder::empty();
        mb.fill(TileType::Wall);
        RoomsArchitect::build_random_rooms(&mut mb, rng);
        RoomsArchitect::build_corridors(&mut mb, rng);
        mb.player_start = mb.rooms[0].center();
        mb.amulet_start = mb.find_most_distant();
        for room in mb.rooms.iter().skip(1) {
            mb.monster_spawns.push(room.center());
        }
        mb
    }
}

impl RoomsArchitect {
    fn build_random_rooms(mb: &mut MapBuilder, rng: &mut RandomNumberGenerator) {
        while mb.rooms.len() < NUM_ROOMS {
            let room = Rect::with_size(
                rng.range(1, SCREEN_WIDTH - 10),
                rng.range(1, SCREEN_HEIGHT - 10),
                rng.range(2, 15),
                rng.range(2, 15),
            );
            let mut overlap = false;
            for r in mb.rooms.iter() {
                if r.intersect(&room) {
                    overlap = true;
                }
            }
            if !overlap {
                room.for_each(|p| {
                    if p.x > 0 && p.x < SCREEN_WIDTH && p.y > 0 && p.y < SCREEN_HEIGHT {
                        let idx = map_idx(p.x, p.y);
                        mb.map.tiles[idx] = TileType::Floor;
                    }
                });
                mb.rooms.push(room);
            }
        }
    }
    fn apply_vertical_tunnel(mb: &mut MapBuilder, y1: i32, y2: i32, x: i32) {
        use std::cmp::{max, min};
        for y in min(y1, y2)..=max(y1, y2) {
            if let Some(idx) = mb.map.try_idx(Point::new(x, y)) {
                mb.map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }
    fn apply_horizontal_tunnel(mb: &mut MapBuilder, x1: i32, x2: i32, y: i32) {
        use std::cmp::{max, min};
        for x in min(x1, x2)..=max(x1, x2) {
            if let Some(idx) = mb.map.try_idx(Point::new(x, y)) {
                mb.map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }
    fn build_corridors(mb: &mut MapBuilder, rng: &mut RandomNumberGenerator) {
        let mut rooms = mb.rooms.clone();
        rooms.sort_by(|a, b| a.center().x.cmp(&b.center().x));
        for (i, room) in rooms.iter().enumerate().skip(1) {
            let prev = rooms[i - 1].center();
            let new = room.center();
            if rng.range(0, 2) == 1 {
                RoomsArchitect::apply_horizontal_tunnel(mb, prev.x, new.x, prev.y);
                RoomsArchitect::apply_vertical_tunnel(mb, prev.y, new.y, new.x);
            } else {
                RoomsArchitect::apply_vertical_tunnel(mb, prev.y, new.y, prev.x);
                RoomsArchitect::apply_horizontal_tunnel(mb, prev.x, new.x, new.y);
            }
        }
    }
}
