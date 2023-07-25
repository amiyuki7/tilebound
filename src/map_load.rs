use std::fs;

use crate::*;
#[derive(Resource)]
pub struct MapContext<'a> {
    pub id: &'a str,
    pub current_map: Vec<Tile>,
}
impl<'a> MapContext<'a> {
    pub fn from_map(id: &'a str) -> Self {
        MapContext {
            id,
            current_map: load_new_map_data(id),
        }
    }
    // fn fetch(id: &str) -> Vec<Tile>
}

pub fn load_new_map_data(id: &str) -> Vec<Tile> {
    let contents = fs::read_to_string("src/world.json").expect("Something went wrong reading the file");
    let mut deserialized: HashMap<&str, Vec<Tile>> = serde_json::from_str(&contents).unwrap();

    deserialized.remove(id).unwrap()
}

// pub fn id_to_map(id: &str) -> Vec<Tile> {}
