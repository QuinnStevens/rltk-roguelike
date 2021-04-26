use super::{
    MapBuilder, Map, TileType, Position, spawner, SHOW_MAPGEN_VISUALISER,
    generate_voronoi_spawn_regions, get_most_distant_area,
};
use rltk::RandomNumberGenerator;
use std::collections::HashMap;
use specs::prelude::*;

mod image_loader;
use image_loader::*;
mod constraints;
use constraints::*;
mod solver;
use solver::*;
mod common;
use common::*;

#[derive(PartialEq, Copy, Clone)]
pub enum WaveformMode { TestMap, Derived }

pub struct WaveformCollapseBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    mode: WaveformMode,
    derive_from: Option<Box<dyn MapBuilder>>,
}

impl MapBuilder for WaveformCollapseBuilder {
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
        for area in self.noise_areas.iter() {
            spawner::spawn_region(ecs, area.1, self.depth);
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

impl WaveformCollapseBuilder {
    pub fn new(new_depth: i32, mode: WaveformMode, derive_from: Option<Box<dyn MapBuilder>>) -> WaveformCollapseBuilder {
        WaveformCollapseBuilder {
            map: Map::new(new_depth),
            starting_position: Position{x: 0, y: 0},
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            mode,
            derive_from,
        }
    }

    // Constructors
    pub fn test_map(new_depth: i32) -> WaveformCollapseBuilder {
        WaveformCollapseBuilder::new(new_depth, WaveformMode::TestMap, None)
    }

    pub fn derived_map(new_depth:i32, builder: Box<dyn MapBuilder>) -> WaveformCollapseBuilder {
        WaveformCollapseBuilder::new(new_depth, WaveformMode::Derived, Some(builder))
    }

    fn build(&mut self) {
        if self.mode == WaveformMode::TestMap {
            self.map = load_rex_map(self.depth, &rltk::rex::XpFile::from_resource("../resources/wfc-demo1.xp").unwrap());
            self.take_snapshot();
            return;
        }

        let mut rng = RandomNumberGenerator::new();

        // Size of a chunk
        const CHUNK_SIZE: i32 = 8;

        let prebuilder = &mut self.derive_from.as_mut().unwrap();
        prebuilder.build_map();
        self.map = prebuilder.get_map();
        for t in self.map.tiles.iter_mut() {
            if *t == TileType::DownStairs { * t = TileType::Floor; }
        }
        self.take_snapshot();

        let patterns = build_patterns(&self.map, CHUNK_SIZE, true, true);
        let constraints = patterns_to_constraints(patterns, CHUNK_SIZE);
        self.render_tile_gallery(&constraints, CHUNK_SIZE);

        self.map = Map::new(self.depth);
        loop {
            let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &self.map);
            while !solver.iteration(&mut self.map, &mut rng) {
                self.take_snapshot();
            }
            self.take_snapshot();
            if solver.possible { break; } // If it has hit an impossible condition, try again.
        }

        // Find a starting point; start at the middle & walk left until we find an open space
        self.starting_position = Position{ x: self.map.width / 2, y: self.map.height / 2 };
        let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        }

        // Find all tiles we can reach from the starting point
        let exit_tile = get_most_distant_area(&mut self.map, start_idx, true);
        self.take_snapshot();

        // Place stairs
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        // Build a noise map
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
    }

    fn render_tile_gallery(&mut self, constraints: &Vec<MapChunk>, chunk_size: i32) {
        self.map = Map::new(0);
        let mut counter = 0;
        let mut x = 1;
        let mut y = 1;
        while counter < constraints.len() {
            render_pattern_to_map(&mut self.map, &constraints[counter], chunk_size, x, y);

            x += chunk_size +1;
            if x + chunk_size > self.map.width {
                // Move to next row
                x = 1;
                y += chunk_size + 1;

                if y + chunk_size > self.map.height {
                    // Move to next page
                    self.take_snapshot();
                    self.map = Map::new(0);

                    x = 1;
                    y = 1;
                }
            }

            counter += 1;
        }
        self.take_snapshot();
    }
}