// use std::collections::HashMap;
//
// use chrono::{DateTime, Utc};
// use legion::prelude::*;
// use legion::world::{
//     World, Universe,
// };
//
// // store library // store peers //
//
// pub struct WorldState {
//     world: World,
//     ip_addr_map: HashMap<String, u32>,
//     schedule: Option<Schedule>,
// }
//
// impl WorldState {
//     pub fn new(world: World) -> WorldState {
//         let mut ip_addr_map: HashMap<String, u32> = HashMap::new();
//         WorldState {
//             world,
//             ip_addr_map,
//             schedule: None,
//         }
//     }
//
//     pub fn setup(&mut self) {
//         let update_peers = SystemBuilder::new("update_peers")
//             .with_query(<(Write<Peer>, Read<IpAddress>)>::query())
//             .build(|_, mut world, _, query| {
//                 for (mut peer, ip_addr) in query.iter(&mut world) {
//                     // check if active, poll for peers
//                 }
//             });
//         let schedule = Schedule::builder().add_system(update_peers).build();
//         // schedule.execute
//     }
//
//     pub fn add_peer(&mut self, peer: Peer, ip_addr: &str, active: bool) {
//         let entities: &[Entity] = self.world.insert(
//             (IpAddress(ip_addr.to_string()), Active(active)),
//             vec![(peer ,)],
//         );
//     }
// }
//
// #[derive(Clone, Debug, PartialEq)]
// struct Active(bool);
//
// #[derive(Clone, Debug, PartialEq)]
// struct IpAddress(String);
//
// #[derive(Clone, Debug, PartialEq)]
// pub struct Peer {
//     name: String,
//     location: String,
//     last_contact: DateTime<Utc>,
//     referrer: u32,
// }
//
// impl Peer {
//     pub fn new(name: string, location: String, last_contact: DateTime<Utc>, referrer: u32) -> Self {
//         Peer {
//             name,
//             location,
//             last_contact,
//             referrer,
//         }
//     }
// }
//
// #[derive(Clone, Debug, PartialEq)]
// struct Album {
//     artist: String,
//     name: String,
//     hash: String,
// }
//
// #[derive(Clone, Debug, PartialEq)]
// struct Blocked;
//
// pub fn create_world_state() -> WorldState {
//     let universe = Universe::new();
//     WorldState::new(universe.create_world())
// }
//
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_world_state() {
//         let mut world_state = create_world_state();
//         world_state.setup();
//         world_state.add_peer(
//             Peer::new(
//                 "test".to_string(),
//                 "test".to_string(),
//
//             )
//         )
//     }
// }
