use serde::{Deserialize, Serialize};
use serde_json;
use std::{collections::HashMap, fs};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tile {
    pub coord: HexCoord,
    pub is_obstructed: bool,
    pub can_be_clicked: bool,
    pub sub_region_id: Option<String>,
    #[serde(default, skip_serializing)]
    pub is_hovered: bool,
    #[serde(default, skip_serializing)]
    pub is_clicked: bool,
}
impl Tile {
    pub fn new(q: i32, r: i32, is_obstructed: bool, sub_region_id: Option<String>) -> Tile {
        Tile {
            coord: HexCoord::new(q, r),
            is_obstructed,
            can_be_clicked: false,
            sub_region_id,
            is_hovered: false,
            is_clicked: false,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub fn new(q: i32, r: i32) -> HexCoord {
        HexCoord { q, r }
    }
    pub fn new_from_tupple((q, r): (i32, i32)) -> HexCoord {
        HexCoord { q, r }
    }
    pub fn to_tupple(&self) -> (i32, i32) {
        (self.q, self.r)
    }
}

fn main() {
    let mut tile_coords: Vec<HexCoord> = Vec::new();
    let mut subregions: HashMap<(i32, i32), String> = HashMap::new();
    let mut regions: HashMap<String, Option<HashMap<(i32, i32), String>>> = HashMap::new();

    subregions.insert((0, 0), "1.1".to_string());
    regions.insert("1".to_string(), Some(subregions));
    subregions = HashMap::new();
    subregions.insert((1, 0), "1".to_string());
    regions.insert("1.1".to_string(), Some(subregions));

    for q in 0..2 {
        for r in 0..2 {
            tile_coords.push(HexCoord::new(q, r))
        }
    }

    let obstructed_tiles: Vec<HexCoord> = vec![HexCoord { q: 10, r: 10 }];
    let mut map: HashMap<String, Vec<Tile>> = HashMap::new();
    for (key, value) in regions.iter() {
        let mut tiles: Vec<Tile> = Vec::new();
        for coord in &tile_coords {
            let x = coord.q;
            let z = coord.r;
            let mut is_obstructed = false;
            if obstructed_tiles.contains(&HexCoord::new(x, z)) {
                is_obstructed = true;
            }
            let mut current_tile = Tile::new(x, z, is_obstructed, None);
            if let Some(subregions) = value {
                for (coord, subregion) in subregions {
                    if x == coord.0 && z == coord.1 {
                        current_tile.sub_region_id = Some(subregion.clone());
                    }
                }
            }

            tiles.push(current_tile);
        }
        map.insert(key.to_string(), tiles);
    }
    let serialised = serde_json::to_string(&map).unwrap();
    fs::write("src/data.json", serialised).expect("Unable to write to file");

    let contents =
        fs::read_to_string("src/data.json").expect("Something went wrong reading the file");

    let deserialized: HashMap<&str, Vec<Tile>> = serde_json::from_str(&contents).unwrap();

    for (id, tile_list) in deserialized {
        println!("{:?}", id);
        for tile in tile_list {
            println!("{:?}", tile);
        }
    }
}
