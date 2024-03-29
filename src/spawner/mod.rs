mod template;

use crate::prelude::*;
use template::Templates;

pub fn spawn_level(
    ecs: &mut World,
    res: &mut Resources,
    rng: &mut RandomNumberGenerator,
    level: usize,
    spawn_points: &[Point],
) {
    let template = Templates::load("resources/template.ron");
    template.spawn_entities(ecs, res, rng, level, spawn_points);
}

pub fn spawn_player(ecs: &mut World, pos: Point) {
    ecs.push((
        Player { map_level: 0 },
        pos,
        Render {
            color: ColorPair::new(WHITE, BLACK),
            glyph: to_cp437('@'),
        },
        Health {
            current: 16,
            max: 16,
        },
        Name("Glob Metalis".to_string()),
        FieldOfView::new(8),
        Damage(1),
    ));
}
pub fn spawn_amulet_of_yala(ecs: &mut World, pos: Point) {
    ecs.push((
        Item,
        AmuletOfYala,
        pos,
        Render {
            color: ColorPair::new(WHITE, BLACK),
            glyph: to_cp437('|'),
        },
        Name("Yala amuletas".to_string()),
    ));
}
