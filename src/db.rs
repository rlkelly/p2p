use chrono::{DateTime, Utc};
use std::collections::HashMap;
use legion::world::{
    World, Universe,
};

// datastore // store library // store peers

pub struct WorldState {
    world: World,
    ip_addr_map: HashMap<String, u32>,
}

impl WorldState {
    pub fn new(world: World) -> WorldState {
        let mut ip_addr_map: HashMap<String, u32> = HashMap::new();
        WorldState {
            world,
            ip_addr_map,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Active(bool);

#[derive(Clone, Debug, PartialEq)]
struct Peer {
    ip_addr: String,
    name: String,
    location: String,
    active: bool,
    last_contact: DateTime<Utc>,
    referrer: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct Album {
    artist: String,
    name: String,
    hash: String,
}

#[derive(Clone, Debug, PartialEq)]
struct Blocked;

pub fn create_world_state() -> WorldState {
    let universe = Universe::new();
    WorldState::new(universe.create_world())
}
