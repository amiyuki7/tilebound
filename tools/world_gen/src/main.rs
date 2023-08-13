use serde::{Deserialize, Serialize};
use serde_json;
use std::{collections::HashMap, fs};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tile {
    pub coord: HexCoord,
    pub is_obstructed: bool,
    pub can_be_clicked: bool,
    pub sub_region_id: Option<SubregionData>,
    #[serde(default, skip_serializing)]
    pub is_hovered: bool,
    #[serde(default, skip_serializing)]
    pub is_clicked: bool,
}
impl Tile {
    pub fn new(q: i32, r: i32, is_obstructed: bool, sub_region_id: Option<SubregionData>) -> Tile {
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
pub struct Chest {
    pub hex_coord: HexCoord,
    /// (Item ID, Item Count)
    pub contents: Vec<(usize, u32)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Region {
    pub tiles: Vec<Tile>,
    pub enemies: Option<Vec<Enemy>>,
    pub player_spawn_spot: HexCoord,
    pub chests: Option<Vec<Chest>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubregionData {
    pub id: String,
    pub subregion_type: SubregionType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SubregionType {
    UnclearedCombat,
    ClearedCombat,
    Other,
}

fn main() {
    // Overarching Hashmap that is going to be written to the .json file at the end
    let mut map: HashMap<String, Region> = HashMap::new();

    // Within some region, describes describes whether any tile has its subregion
    let mut tile_subregion_ids: HashMap<(i32, i32), SubregionData> = HashMap::new();
    // Assigns a region to the above data
    let mut region_subregion_ids: HashMap<String, Option<HashMap<(i32, i32), SubregionData>>> = HashMap::new();
    // For each world, describes what tile the player spawns on
    let mut region_spawn_position: HashMap<String, (i32, i32)> = HashMap::new();
    // holds the combat data in (region id - enemies) pairs
    let mut enemy_locations: HashMap<String, Option<Vec<Enemy>>> = HashMap::new();
    let mut enemy_list: Vec<Enemy> = Vec::new();

    // Chest data
    let mut chest_locations: HashMap<String, Option<Vec<Chest>>> = HashMap::new();

    // Example: Lets create the overworld ("1") with 2 subregions ("1.1", "1.2")
    // To do this, we need to first need to define what tiles will house these subregions
    tile_subregion_ids.insert(
        (1, 1),
        SubregionData {
            id: "1.1".to_string(),
            subregion_type: SubregionType::UnclearedCombat,
        },
    );
    tile_subregion_ids.insert(
        (2, 2),
        SubregionData {
            id: "1.2".to_string(),
            subregion_type: SubregionType::Other,
        },
    );
    // So now, we tell the generative code to create a region ("1")
    // The region will have 2 tiles, located at the above coordinates, that will send the player to the subregion "1.1" and "1.2"

    region_subregion_ids.insert("1".to_string(), Some(tile_subregion_ids));
    // Also we tell the code which position to spawn the player in when the region is loaded
    region_spawn_position.insert("1".to_string(), (0, 0));
    // Since it is the overworld, it will not have any enemies, so `enemy_locations` will be empty for "1"
    enemy_locations.insert("1".to_string(), None);
    // I want "1.1" to be a combat, so I add a basic enemy to the enemy list and then push that list to `enemy_locations`
    enemy_list.push(Enemy::new(0, 1, 2, 1, 10.0, 10.0));
    enemy_locations.insert("1.1".to_string(), Some(enemy_list));
    // Give the player a spawn point in the region
    region_spawn_position.insert("1.1".to_string(), (5, 5));
    // I don't want "1.2" to be a combat, so `enemy_locations` will be empty for it also
    enemy_locations.insert("1.2".to_string(), None);
    // Give the player a spawn point in the region
    region_spawn_position.insert("1.2".to_string(), (2, 3));
    // To keep it simple, the subregions won't have their own subregions. However, it is possible to do so. Just keep in mind that all the Hashmaps that are String: Something will need to have data on that subregion
    region_subregion_ids.insert("1.1".to_string(), None);
    region_subregion_ids.insert("1.2".to_string(), None);

    chest_locations.insert(
        "1".to_string(),
        Some(vec![Chest {
            hex_coord: HexCoord::new(4, 2),
            contents: vec![(0, 10), (1, 4), (2, 1)],
        }]),
    );
    chest_locations.insert("1.1".to_string(), None);
    chest_locations.insert("1.2".to_string(), None);

    // Generates a world based on data provided in hashmaps
    for (key, value) in region_subregion_ids.iter() {
        let mut tile_vec: Vec<Tile> = Vec::new();
        let mut enemy_vec: Vec<Enemy> = Vec::new();
        let mut chest_vec: Vec<Chest> = Vec::new();

        for q in -1..=6 {
            for r in -1..=6 {
                let mut obstructed = false;
                if q == -1 || q == 6 || r == -1 || r == 6 {
                    obstructed = true
                }
                let mut current_tile = Tile::new(q, r, obstructed, None);
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

        if let Some(chests) = &chest_locations[key] {
            for chest in chests {
                chest_vec.push(chest.clone());
            }
        }
        let mut chests: Option<Vec<Chest>> = None;
        if !chest_vec.is_empty() {
            chests = Some(chest_vec.clone());
        }

        let current_region = Region {
            tiles: tile_vec,
            enemies,
            player_spawn_spot: HexCoord::new_from_tupple(*region_spawn_position.get(key).clone().unwrap()),
            chests,
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
