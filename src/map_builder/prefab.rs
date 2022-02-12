use crate::prelude::*;

const FORTRESS: (&str, i32, i32) = (
    "------------
---######---
---#----#---
---#-M--#---
-###----###-
--M------M--
-###----###-
---#----#---
---#----#---
---######---
------------",
    12,
    11,
);

pub fn apply_prefab(mb: &mut MapBuilder, rng: &mut RandomNumberGenerator) {
    let mut placement = None;
    let dijkstra_map = mb.get_player_dijkstra_flow();
    let mut attempts = 0;
    while placement.is_none() && attempts < 10 {
        let dimentions = Rect::with_size(
            rng.range(0, SCREEN_WIDTH - FORTRESS.1),
            rng.range(0, SCREEN_HEIGHT - FORTRESS.2),
            FORTRESS.1,
            FORTRESS.2,
        );
        let mut can_place = false;
        dimentions.for_each(|pt| {
            let idx = mb.map.point2d_to_index(pt);
            let distance = dijkstra_map.map[idx];
            if distance < 2000.0 && distance > 20.0 && mb.amulet_start != pt {
                can_place = true;
            }
        });
        if can_place {
            placement = Some(Point::new(dimentions.x1, dimentions.y1));
            let points = dimentions.point_set();
            mb.monster_spawns.retain(|pt| !points.contains(pt));
        }
        attempts += 1;

        if let Some(placement) = placement {
            let string_vec = FORTRESS
                .0
                .chars()
                .filter(|a| *a != '\r' && *a != '\n')
                .collect::<Vec<char>>();

            let mut i = 0;
            for ty in placement.y..placement.y + FORTRESS.2 {
                for tx in placement.x..placement.x + FORTRESS.1 {
                    let idx = map_idx(tx, ty);
                    let c = string_vec[i];
                    match c {
                        'M' => {
                            mb.map.tiles[idx] = TileType::Floor;
                            mb.monster_spawns.push(Point::new(tx, ty));
                        }
                        '-' => mb.map.tiles[idx] = TileType::Floor,
                        '#' => mb.map.tiles[idx] = TileType::Wall,
                        _ => print!("No idea what to do with [{}]", c),
                    }
                    i += 1;
                }
            }
        }
    }
}
