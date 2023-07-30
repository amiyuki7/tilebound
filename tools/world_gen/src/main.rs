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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Enemy {
    pub hex_coord: HexCoord,
    #[serde(default, skip_serializing)]
    pub path: Option<Vec<HexCoord>>,
    pub attack_range: i32,
    pub movement_range: i32,
    pub damage: f32,
    pub health: Health,
}

impl Enemy {
    pub fn new(q: i32, r: i32, attack_range: i32, movement_range: i32, damage: f32, hp: f32) -> Enemy {
        Enemy {
            hex_coord: HexCoord::new(q, r),
            path: None,
            attack_range,
            movement_range,
            damage,
            health: Health::new(hp),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Health {
    pub max_hp: f32,
    pub hp: f32,
}
impl Health {
    pub fn new(hp: f32) -> Health {
        Health { max_hp: hp, hp }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Region {
    pub tiles: Vec<Tile>,
    pub enemies: Option<Vec<Enemy>>,
}

fn main() {
    let mut map: HashMap<String, Region> = HashMap::new();

    let mut tile_subregion_ids: HashMap<(i32, i32), String> = HashMap::new();
    let mut region_subregion_ids: HashMap<String, Option<HashMap<(i32, i32), String>>> = HashMap::new();

    let mut enemy_locations: HashMap<String, Option<Vec<Enemy>>> = HashMap::new();
    let mut enemy_list: Vec<Enemy> = Vec::new();
    enemy_list.push(Enemy::new(0, 1, 2, 1, 10.0, 10.0));
    enemy_locations.insert("1".to_string(), Some(enemy_list));
    enemy_locations.insert("1.1".to_string(), None);
    enemy_locations.insert("1.2".to_string(), None);

    tile_subregion_ids.insert((1, 1), "1.1".to_string());
    tile_subregion_ids.insert((2, 2), "1.2".to_string());

    region_subregion_ids.insert("1".to_string(), Some(tile_subregion_ids));
    region_subregion_ids.insert("1.1".to_string(), None);
    region_subregion_ids.insert("1.2".to_string(), None);

    for (key, value) in region_subregion_ids.iter() {
        let mut tile_vec: Vec<Tile> = Vec::new();
        let mut enemy_vec: Vec<Enemy> = Vec::new();

        for q in 0..3 {
            for r in 0..3 {
                let mut current_tile = Tile::new(q, r, false, None);
                if let Some(subregions) = value {
                    for (coord, id) in subregions {
                        if (q, r) == *coord {
                            current_tile.sub_region_id = Some(id.clone());
                        }
                    }
                }

                tile_vec.push(current_tile)
            }
        }

        if let Some(enemies) = &enemy_locations[key] {
            for enemy in enemies {
                enemy_vec.push(enemy.clone())
            }
        }
        let mut enemies: Option<Vec<Enemy>> = None;
        if enemy_vec.len() > 0 {
            enemies = Some(enemy_vec.clone())
        }
        let current_region = Region {
            tiles: tile_vec,
            enemies,
        };

        map.insert(key.clone(), current_region);
    }
    let serialised = serde_json::to_string(&map).unwrap();
    fs::write("tools/world_gen/src/data.json", serialised).expect("Unable to write to file");

    let contents = fs::read_to_string("tools/world_gen/src/data.json").expect("Something went wrong reading the file");

    let deserialized: HashMap<String, Region> = serde_json::from_str(&contents).unwrap();

    for (id, region) in deserialized {
        println!("{:?}", id);
        for tile in region.tiles {
            println!("{:?}", tile)
        }
        if let Some(enemy_list) = region.enemies {
            for enemy in enemy_list {
                println!("{:?}", enemy)
            }
        }
    }
}
