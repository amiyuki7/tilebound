use crate::*;
use serde::Deserialize;

#[derive(Reflect, Component, Default, Deserialize)]
#[reflect(Component)]
pub struct Tile {
    pub coord: HexCoord,
    pub is_obstructed: bool,
    pub is_hovered: bool,
    pub is_clicked: bool,
    pub can_be_clicked: bool,
}
impl Tile {
    pub fn new(q: i32, r: i32, is_obstructed: bool) -> Tile {
        Tile {
            coord: HexCoord::new(q, r),
            is_obstructed,
            is_hovered: false,
            is_clicked: false,
            can_be_clicked: false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Reflect, Default, Deserialize)]
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

#[derive(Copy, Clone, PartialEq, Eq)]
struct HexNode {
    coord: HexCoord,
    is_obstructed: bool,
    g_score: i32,
    f_score: i32,
    parent: Option<HexCoord>,
}

impl Ord for HexNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for HexNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn offset2axial(a: &HexCoord) -> HexCoord {
    let q = a.q - (a.r - (a.r & 1)) / 2;
    let r = a.r;
    HexCoord::new(q, r)
}

pub fn hex_distance(a: &HexCoord, b: &HexCoord) -> i32 {
    let a = offset2axial(a);
    let b = offset2axial(b);
    let dq = (a.q - b.q).abs();
    let dr = (a.r - b.r).abs();
    match vec![dq, dr, (a.q - b.q + a.r - b.r).abs()].iter().max() {
        Some(max) => *max,
        None => dq,
    }
}

/// Backtraces the path
fn reconstruct_path(came_from: HashMap<HexCoord, HexCoord>, mut current: HexCoord) -> Vec<HexCoord> {
    let mut path = vec![current];
    while let Some(&parent) = came_from.get(&current) {
        path.push(parent);
        current = parent;
    }
    path.reverse();
    path
}

fn check_obstructed(obstructed_tiles: &Vec<HexCoord>, current_coord: HexCoord) -> bool {
    obstructed_tiles.iter().find(|&&coord| coord == current_coord).is_some()
}

pub fn astar(start: HexCoord, goal: HexCoord, obstructed_tiles: &Vec<HexCoord>) -> Option<Vec<HexCoord>> {
    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<HexCoord, HexCoord> = HashMap::new();
    let mut g_score: HashMap<HexCoord, i32> = HashMap::new();

    g_score.insert(start, 0);
    open_set.push(HexNode {
        coord: start,
        is_obstructed: false,
        g_score: 0,
        f_score: hex_distance(&start, &goal),
        parent: None,
    });

    while let Some(current_node) = open_set.pop() {
        let current = current_node.coord;
        if current == goal {
            return Some(reconstruct_path(came_from, current));
        }

        for neighbor in get_neighbors(&current) {
            if check_obstructed(obstructed_tiles, current) {
                continue;
            }
            let tentative_g_score = g_score[&current] + 1;

            if !g_score.contains_key(&neighbor) || tentative_g_score < g_score[&neighbor] {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative_g_score);
                let f_score = tentative_g_score + hex_distance(&neighbor, &goal);
                open_set.push(HexNode {
                    coord: neighbor,
                    is_obstructed: check_obstructed(obstructed_tiles, current),
                    g_score: tentative_g_score,
                    f_score,
                    parent: Some(current),
                });
            }
        }
    }

    None
}

fn get_neighbors(coord: &HexCoord) -> Vec<HexCoord> {
    let q = coord.q;
    let r = coord.r;

    if r % 2 == 0 {
        vec![
            HexCoord::new(q + 1, r),
            HexCoord::new(q, r + 1),
            HexCoord::new(q - 1, r + 1),
            HexCoord::new(q - 1, r),
            HexCoord::new(q - 1, r - 1),
            HexCoord::new(q, r - 1),
        ]
    } else {
        vec![
            HexCoord::new(q + 1, r),
            HexCoord::new(q + 1, r + 1),
            HexCoord::new(q, r + 1),
            HexCoord::new(q - 1, r),
            HexCoord::new(q, r - 1),
            HexCoord::new(q + 1, r - 1),
        ]
    }
}
