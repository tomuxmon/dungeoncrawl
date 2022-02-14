mod camera;
mod components;
mod map;
mod map_builder;
mod spawner;
mod systems;
mod turn_state;
mod prelude {
    pub use bracket_lib::prelude::*;
    pub use legion::systems::CommandBuffer;
    pub use legion::world::SubWorld;
    pub use legion::*;
    pub const SCREEN_WIDTH: i32 = 80;
    pub const SCREEN_HEIGHT: i32 = 50;
    pub const DISPLAY_WIDTH: i32 = SCREEN_WIDTH / 2;
    pub const DISPLAY_HEIGHT: i32 = SCREEN_HEIGHT / 2;
    pub use crate::camera::*;
    pub use crate::components::*;
    pub use crate::map::*;
    pub use crate::map_builder::*;
    pub use crate::spawner::*;
    pub use crate::systems::*;
    pub use crate::turn_state::*;
}
use prelude::*;

struct State {
    ecs: World,
    resources: Resources,
    input_systems: Schedule,
    turn_systems: Schedule,
}

impl State {
    fn new() -> Self {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng);
        spawn_player(&mut ecs, map_builder.player_start);
        let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
        map_builder.map.tiles[exit_idx] = TileType::Exit;
        spawn_level(
            &mut ecs,
            &mut resources,
            &mut rng,
            0,
            &map_builder.monster_spawns,
        );
        resources.insert(map_builder.map);
        resources.insert(Camera::new(map_builder.player_start));
        resources.insert(TurnState::AwaitingInput);
        resources.insert(map_builder.theme);
        // TODO: reduce prone to confusion effect.
        // input_systems coorelates to input_scheduler and so on..
        // begs for a different state structure definition
        Self {
            ecs,
            resources,
            input_systems: build_input_scheduler(),
            turn_systems: build_turn_scheduler(),
        }
    }

    fn game_over(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(2);
        ctx.print_color_centered(2, RED, BLACK, "Tavo misija baigesi.");
        ctx.print_color_centered(
            4,
            WHITE,
            BLACK,
            "Monstrai tave nugalejo, tavo herojaus kelione baigesi anksciau...",
        );
        ctx.print_color_centered(
            5,
            WHITE,
            BLACK,
            "YALA amuletas vis dar nerastas ir namu miestelis neisgelbetas.",
        );
        ctx.print_color_centered(
            8,
            YELLOW,
            BLACK,
            "Nesijaudink, galesi pameginti su kitu herojumi.",
        );
        ctx.print_color_centered(9, GREEN, BLACK, "Spausk 1 ir zaisk is naujo.");

        if let Some(VirtualKeyCode::Key1) = ctx.key {
            self.reset_game_state();
        }
    }

    fn victory(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(2);
        ctx.print_color_centered(2, GREEN, BLACK, "Tu laimejai!");
        ctx.print_color_centered(
            4,
            WHITE,
            BLACK,
            "Herojus uzsidejo YALA amuleta ir pajuto galia tekancia jo venomis.",
        );
        ctx.print_color_centered(
            5,
            WHITE,
            BLACK,
            "Tavo miestelis isgelbetas, galima gryszti i normalu gyvenima.",
        );
        ctx.print_color_centered(7, GREEN, BLACK, "Arba spausk 1 ir zaisk is naujo.");
        if let Some(VirtualKeyCode::Key1) = ctx.key {
            self.reset_game_state();
        }
    }

    fn reset_game_state(&mut self) {
        self.ecs = World::default();
        self.resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng);
        spawn_player(&mut self.ecs, map_builder.player_start);
        let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
        map_builder.map.tiles[exit_idx] = TileType::Exit;
        spawn_level(
            &mut self.ecs,
            &mut self.resources,
            &mut rng,
            0,
            &map_builder.monster_spawns,
        );
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
    }

    fn next_level(&mut self) {
        use std::collections::HashSet;
        let player_entity = *<Entity>::query()
            .filter(component::<Player>())
            .iter(&self.ecs)
            .nth(0)
            .unwrap();
        let mut entities_to_keep = HashSet::new();
        entities_to_keep.insert(player_entity);

        <(Entity, &Carried)>::query()
            .iter(&self.ecs)
            .filter(|(_, c)| c.0 == player_entity)
            .map(|(e, _)| *e)
            .for_each(|e| {
                entities_to_keep.insert(e);
            });

        let mut cb = CommandBuffer::new(&mut self.ecs);
        for e in Entity::query().iter(&self.ecs) {
            if !entities_to_keep.contains(e) {
                cb.remove(*e);
            }
        }
        cb.flush(&mut self.ecs, &mut self.resources);

        <&mut FieldOfView>::query()
            .iter_mut(&mut self.ecs)
            .for_each(|fov| fov.is_dirty = true);

        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng);
        let mut map_level = 0;
        <(&mut Player, &mut Point)>::query()
            .iter_mut(&mut self.ecs)
            .for_each(|(player, pos)| {
                player.map_level += 1;
                map_level = player.map_level;
                pos.x = map_builder.player_start.x;
                pos.y = map_builder.player_start.y;
            });

        if map_level == 2 {
            spawn_amulet_of_yala(&mut self.ecs, map_builder.amulet_start);
        } else {
            let exist_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
            map_builder.map.tiles[exist_idx] = TileType::Exit;
        }

        spawn_level(
            &mut self.ecs,
            &mut self.resources,
            &mut rng,
            map_level as usize,
            &map_builder.monster_spawns,
        );
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(0);
        ctx.cls();
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(2);
        ctx.cls();
        self.resources.insert(ctx.key);
        ctx.set_active_console(0);
        self.resources.insert(Point::from_tuple(ctx.mouse_pos()));
        // TODO: fix the dirty unwrap
        let current_state = self.resources.get::<TurnState>().unwrap().clone();
        match current_state {
            TurnState::AwaitingInput => self
                .input_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::GameOver => self.game_over(ctx),
            TurnState::Victory => self.victory(ctx),
            TurnState::NextLevel => self.next_level(),
            _ => self // TurnState::PlayerTurn || TurnState::MonsterTurn
                .turn_systems
                .execute(&mut self.ecs, &mut self.resources),
        }
        render_draw_buffer(ctx).expect("Render Error");
    }
}

fn main() -> BError {
    main_loop(
        BTermBuilder::new()
            .with_title("Dread Crawler")
            .with_fps_cap(30.0)
            .with_dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .with_tile_dimensions(32, 32)
            .with_resource_path("resources/")
            .with_font("dungeonfont.png", 32, 32)
            .with_font("terminal8x8.png", 8, 8)
            .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
            .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
            .with_simple_console_no_bg(SCREEN_WIDTH * 2, SCREEN_HEIGHT * 2, "terminal8x8.png")
            .build()?,
        State::new(),
    )
}
