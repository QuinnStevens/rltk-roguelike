use super::{
    MapBuilder, Map, TileType, Position, spawner, SHOW_MAPGEN_VISUALISER,
    get_most_distant_area, generate_voronoi_spawn_regions
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::HashMap;

#[derive(PartialEq, Clone)]
#[allow(dead_code)]
pub enum PrefabMode {
    RexLevel{ template: &'static str }
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    spawns: Vec<(usize, String)>,
}

impl MapBuilder for PrefabBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.spawns.iter() {
            spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALISER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl PrefabBuilder {
    pub fn new(new_depth: i32) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position{ x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::RexLevel{ template: "../resources/wfc-demo1.xp" },
            spawns: Vec::new(),
        }
    }

    fn build(&mut self) {
        match self.mode {
            PrefabMode::RexLevel{template} => self.load_rex_map(&template)
        }
        self.take_snapshot();

        if self.starting_position.x == 0 {
            // Find a starting point
            self.starting_position = Position{ x: self.map.width / 2, y: self.map.height / 2 };
            let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
            while self.map.tiles[start_idx] != TileType:: Floor {
                self.starting_position.x -= 1;
                start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
            }
            self.take_snapshot();

            // Find all the tiles we can reach from the starting point
            let exit_tile = get_most_distant_area(&mut self.map, start_idx, true);
            self.take_snapshot();

            // Place the stairs
            self.map.tiles[exit_tile] = TileType::DownStairs;
            self.take_snapshot();
        }
    }

    #[allow(dead_code)]
    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < self.map.width as usize && y < self.map.height as usize {
                        let idx = self.map.xy_idx(x as i32, y as i32);
                        // Set tiles
                        match (cell.ch as u8) as char {
                            ' ' => self.map.tiles[idx] = TileType::Floor, // space
                            '#' => self.map.tiles[idx] = TileType::Wall, // #
                            '@' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.starting_position = Position{ x: x as i32, y: y as i32 };
                            }
                            '>' => self.map.tiles[idx] = TileType::DownStairs,
                            'g' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "Goblin".to_string()));
                            }
                            'o' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "Orc".to_string()));
                            }
                            '^' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "Bear Trap".to_string()));
                            }
                            '%' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "Rations".to_string()));
                            }
                            '!' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "Health Potion".to_string()));
                            }
                            _ => {
                                rltk::console::log(format!("Unknown glyph loading map: {}", (cell.ch as u8) as char));
                            }
                        }
                    }
                }
            }
        }
    }
}
