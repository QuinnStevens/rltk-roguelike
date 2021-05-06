use super::{
    InitialMapBuilder, BuilderMap, TileType
};
use rltk::RandomNumberGenerator;

pub struct CellularAutomataBuilder {}

impl InitialMapBuilder for CellularAutomataBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}
impl CellularAutomataBuilder {
    #![allow(dead_code)]
    pub fn new() -> Box<CellularAutomataBuilder> {
        Box::new(CellularAutomataBuilder{})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // First we completely randomise the map, setting 55% of it to be floor.
        for y in 1..build_data.map.height-1 {
            for x in 1..build_data.map.width-1 {
                let roll = rng.roll_dice(1, 100);
                let idx = build_data.map.xy_idx(x, y);
                if roll > 55 { build_data.map.tiles[idx] = TileType::Floor }
                else { build_data.map.tiles[idx] = TileType::Wall }
            }
        }
        build_data.take_snapshot();

        // Now we iteratively apply cellular automata rules.
        for _i in 0..15 {
            let mut newtiles = build_data.map.tiles.clone();

            for y in 1..build_data.map.height-1 {
                for x in 1..build_data.map.width-1 {
                    let idx = build_data.map.xy_idx(x, y);
                    // Count the tile's neighbours.
                    let mut neighbours = 0;
                    if build_data.map.tiles[idx - 1] == TileType::Wall { neighbours += 1; } // west
                    if build_data.map.tiles[idx + 1] == TileType::Wall { neighbours += 1; } // east
                    if build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Wall { neighbours += 1; } // north
                    if build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Wall { neighbours += 1; } // south
                    if build_data.map.tiles[idx - build_data.map.width as usize - 1] == TileType::Wall { neighbours += 1; } // northwest
                    if build_data.map.tiles[idx - build_data.map.width as usize + 1] == TileType::Wall { neighbours += 1; } // northeast
                    if build_data.map.tiles[idx + build_data.map.width as usize - 1] == TileType::Wall { neighbours += 1; } // southwest
                    if build_data.map.tiles[idx + build_data.map.width as usize + 1] == TileType::Wall { neighbours += 1; } // southeast

                    if neighbours > 4 || neighbours == 0 {
                        newtiles[idx] = TileType::Wall;
                    } else {
                        newtiles[idx] = TileType::Floor;
                    }

                }
            }
            build_data.map.tiles = newtiles.clone();
            build_data.take_snapshot();
        }
    }
}
