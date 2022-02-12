mod automata;
mod drunkard;
mod empty;
mod prefab;
mod rooms;
mod themes;
use crate::prelude::*;
use automata::CelularAutomataArchitect;
use drunkard::DrunkardWalkArchitect;
use empty::EmptyArchitect;
use prefab::apply_prefab;
use rooms::RoomsArchitect;
use themes::*;

trait MapArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder;
}
pub trait MapTheme: Sync + Send {
    fn tile_to_render(&self, tile_type: TileType) -> FontCharType;
}

pub struct MapBuilder {
    pub map: Map,
    pub rooms: Vec<Rect>,
    pub monster_spawns: Vec<Point>,
    pub player_start: Point,
    pub amulet_start: Point,
    pub theme: Box<dyn MapTheme>,
}

impl MapBuilder {
    pub fn empty() -> Self {
        Self {
            map: Map::new(),
            rooms: Vec::new(),
            monster_spawns: Vec::new(),
            player_start: Point::zero(),
            amulet_start: Point::zero(),
            theme: DungeonTheme::new(),
        }
    }

    pub fn new(rng: &mut RandomNumberGenerator) -> Self {
        let mut architect: Box<dyn MapArchitect> = match rng.range(0, 3) {
            0 => Box::new(DrunkardWalkArchitect {}),
            1 => Box::new(RoomsArchitect {}),
            2 => Box::new(CelularAutomataArchitect {}),
            _ => Box::new(EmptyArchitect {}),
        };
        let mut mb = architect.new(rng);
        apply_prefab(&mut mb, rng);
        mb.theme = match rng.range(0, 3) {
            0 => DungeonTheme::new(),
            1 => SandFortressTheme::new(),
            _ => ForestTheme::new(),
        };
        mb
    }

    fn fill(&mut self, tile: TileType) {
        self.map.tiles.iter_mut().for_each(|t| *t = tile);
    }

    fn find_most_distant(&self) -> Point {
        self.map.index_to_point2d(
            self.get_player_dijkstra_flow()
                .map
                .iter()
                .enumerate()
                .filter(|(_, dist)| *dist < &f32::MAX)
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap()
                .0,
        )
    }

    fn spawn_monsters(&self, start: &Point, rng: &mut RandomNumberGenerator) -> Vec<Point> {
        const NUM_MONSTERS: usize = 50;
        let mut spawnable_tiles: Vec<Point> = self
            .map
            .tiles
            .iter()
            .enumerate()
            .filter(|(idx, t)| {
                **t == TileType::Floor
                    && DistanceAlg::Pythagoras.distance2d(*start, self.map.index_to_point2d(*idx))
                        > 10.0
            })
            .map(|(idx, _)| self.map.index_to_point2d(idx))
            .collect();

        let mut spawns = Vec::new();
        for _ in 0..NUM_MONSTERS {
            let target_index = rng.random_slice_index(&spawnable_tiles).unwrap();
            spawns.push(spawnable_tiles[target_index].clone());
            spawnable_tiles.remove(target_index);
        }
        spawns
    }

    fn get_player_dijkstra_flow(&self) -> DijkstraMap {
        DijkstraMap::new(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            &vec![self.map.point2d_to_index(self.player_start)],
            &self.map,
            1024.0,
        )
    }
}
