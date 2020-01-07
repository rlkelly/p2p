use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::net::IpAddr;

use hibitset::BitSetLike;
use shrev::EventChannel;
use specs::prelude::{
    BitSet, Component, ComponentEvent, Entities, Entity, Join, ReadStorage, ReaderId, ResourceId,
    System, SystemData, Tracked, World, WriteExpect, WriteStorage,
};
use specs::world::Index;

// TODO: deserialize and serialize nodes


#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum NodeEvent {
    Discovered(Entity),
    Modified(Entity),
    Removed(Entity),
}

pub struct WorldState<P> {
    sorted: Vec<Entity>,
    addresses: HashMap<IpAddr, usize>,
    entities: HashMap<Index, usize>,

    changed: EventChannel<NodeEvent>,
    reader_id: ReaderId<ComponentEvent>,

    modified: BitSet,
    inserted: BitSet,
    removed: BitSet,
    scratch_set: HashSet<Entity>,

    _phantom: PhantomData<P>,
}

impl<P> WorldState<P> {
    pub fn new(reader_id: ReaderId<ComponentEvent>) -> Self
    where
        P: Component,
        P::Storage: Tracked,
    {
        WorldState {
            sorted: Vec::new(),
            addresses: HashMap::new(),
            entities: HashMap::new(),
            changed: EventChannel::new(),

            reader_id,
            modified: BitSet::new(),
            inserted: BitSet::new(),
            removed: BitSet::new(),

            scratch_set: HashSet::default(),

            _phantom: PhantomData,
        }
    }

    pub fn get_entity(&self, addr: &IpAddr) -> Option<Entity> {
        if let Some(ix) = self.addresses.get(addr) {
            Some(self.sorted[*ix])
        } else {
            None
        }
    }

    pub fn all(&self) -> &[Entity] {
        self.sorted.as_slice()
    }

    /// Get a token for tracking the modification events from the WorldState
    pub fn track(&mut self) -> ReaderId<NodeEvent> {
        self.changed.register_reader()
    }

    /// Get the `EventChannel` for the modification events for reading
    pub fn changed(&self) -> &EventChannel<NodeEvent> {
        &self.changed
    }

    /// Maintain the WorldState, usually only called by `NodeSystem`.
    pub fn maintain(&mut self, data: EntityData<P>)
    where
        P: Component + Node,
        P::Storage: Tracked,
    {
        let EntityData {
            entities, nodes, ..
        } = data;

        // Maintain tracking
        self.modified.clear();
        self.inserted.clear();
        self.removed.clear();

        let events = nodes.channel().read(&mut self.reader_id);
        for event in events {
            match event {
                ComponentEvent::Modified(id) => {
                    self.modified.add(*id);
                }
                ComponentEvent::Inserted(id) => {
                    self.inserted.add(*id);
                }
                ComponentEvent::Removed(id) => {
                    self.removed.add(*id);
                }
            }
        }

        // bump duplicates
        for (entity, _, node) in (&*entities, &self.inserted, &nodes).join() {
            if let Some(ix) = self.addresses.get(&node.addr()) {
                let other_entity = self.sorted[ix.clone()];
                self.removed.add(other_entity.id());
            }
        }

        // process removed components
        self.scratch_set.clear();
        for id in (&self.removed).iter() {
            if let Some(index) = self.entities.get(&id) {
                self.scratch_set.insert(self.sorted[*index]);
            }
        }

        // do removal
        if !self.scratch_set.is_empty() {
            let mut i = 0;
            let mut min_index = std::usize::MAX;
            while i < self.sorted.len() {
                let entity = self.sorted[i];
                let remove = self.scratch_set.contains(&entity);
                if remove {
                    if i < min_index {
                        min_index = i;
                    }
                    self.scratch_set.insert(entity);
                    self.sorted.remove(i);
                    self.entities.remove(&entity.id());
                } else {
                    i += 1;
                }
            }
            for i in min_index..self.sorted.len() {
                self.entities.insert(self.sorted[i].id(), i);
            }
            for entity in &self.scratch_set {
                self.changed.single_write(NodeEvent::Removed(*entity));
            }
        }
        for (_, _, node) in (&*entities, &self.removed, &nodes).join() {
            self.addresses.remove(&node.addr());
        }
        self.scratch_set.clear();

        for (entity, _, node) in (&*entities, &self.inserted, &nodes).join() {
            let insert_index = self.sorted.len();
            self.entities.insert(entity.id(), insert_index);
            self.sorted.push(entity);
            self.scratch_set.insert(entity);

            if let Some(ix) = self.addresses.get(&node.addr()) {
                let other_entity = self.sorted[ix.clone()];
                if !self.removed.contains(other_entity.id()) {
                    self.changed.single_write(NodeEvent::Removed(other_entity));
                }
            }
            self.addresses.insert(node.addr(), insert_index);
        }

        if !self.scratch_set.is_empty() {
            for i in 0..self.sorted.len() {
                let entity = self.sorted[i];
                let notify = self.scratch_set.contains(&entity);
                if notify {
                    self.scratch_set.insert(entity);
                    self.changed.single_write(NodeEvent::Modified(entity));
                }
            }
        }
        self.scratch_set.clear();
    }
}

pub trait Node {
    fn addr(&self) -> IpAddr;
}

#[derive(SystemData)]
pub struct EntityData<'a, P>
where
    P: Component + Node,
    P::Storage: Tracked,
{
    entities: Entities<'a>,
    nodes: ReadStorage<'a, P>,
}

pub struct NodeSystem<P> {
    m: PhantomData<P>,
}

impl<P> NodeSystem<P>
where
    P: Component + Node + Send + Sync + 'static,
    P::Storage: Tracked,
{
    pub fn new(mut world: &mut World) -> Self {
        <Self as System<'_>>::SystemData::setup(&mut world);
        if !world.has_value::<WorldState<P>>() {
            let world_state = {
                let mut storage: WriteStorage<P> = SystemData::fetch(&world);
                WorldState::<P>::new(storage.register_reader())
            };
            world.insert(world_state);
        }
        NodeSystem { m: PhantomData }
    }
}

impl<'a, P> System<'a> for NodeSystem<P>
where
    P: Component + Node + Send + Sync + 'static,
    P::Storage: Tracked,
{
    type SystemData = (EntityData<'a, P>, WriteExpect<'a, WorldState<P>>);

    fn run(&mut self, (data, mut world_state): Self::SystemData) {
        world_state.maintain(data);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::net::{IpAddr, Ipv6Addr};
    use super::Node as NNode;
    use specs::prelude::{DenseVecStorage, FlaggedStorage};
    use specs::world::Builder;
    use specs::{WorldExt, RunNow};

    struct Node {
        addr: IpAddr,
        name: String,
    }

    impl Component for Node {
        type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
    }

    impl Node {
        fn name(&self) -> String {
            self.name.clone()
        }
    }

    impl NNode for Node {
        fn addr(&self) -> IpAddr {
            self.addr
        }
    }

    fn delete_removals(world: &mut World, reader_id: &mut ReaderId<NodeEvent>) {
        let mut remove = vec![];
        for event in world.fetch::<WorldState<Node>>().changed().read(reader_id) {
            if let NodeEvent::Removed(entity) = *event {
                remove.push(entity);
            }
        }
        for entity in remove {
            if let Err(_) = world.delete_entity(entity) {
                println!("Failed removed entity");
            }
        }
    }

    #[test]
    fn test_make_world() {
        let mut world = World::new();
        world.register::<Node>();
        let mut system = NodeSystem::<Node>::new(&mut world);
        let _reader_id = world.write_resource::<WorldState<Node>>().track();
        let ip1 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        let ip2 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 2));
        let ip3 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 3));
        let ip4 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 4));

        let e1 = world.create_entity().with(Node { addr: ip1, name: "first".to_string() }).build();
        let e2 = world.create_entity().with(Node { addr: ip2, name: "second".to_string() }).build();
        let e3 = world.create_entity().with(Node { addr: ip3, name: "third".to_string() }).build();
        let e4 = world.create_entity().with(Node { addr: ip4, name: "fourth".to_string() }).build();

        system.run_now(&mut world);
        world.maintain();
        assert_eq!(world.is_alive(e1), true);
        assert_eq!(world.is_alive(e2), true);

        let _ = world.delete_entity(e1);
        system.run_now(&mut world);
        world.maintain();

        assert_eq!(world.is_alive(e1), false);
        assert_eq!(world.is_alive(e2), true);

        let _ = world.delete_entity(e3);
        system.run_now(&mut world);
        world.maintain();
        assert_eq!(world.is_alive(e3), false);
        assert_eq!(world.is_alive(e4), true);

        let ip5 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 5));
        let e5 = world.create_entity().with(Node { addr: ip5, name: "fifth".to_string() }).build();
        system.run_now(&mut world);
        world.maintain();

        assert_eq!(3, world.read_resource::<WorldState<Node>>().all().len());
        {
            let nodes = world.read_storage::<Node>();
            let node = nodes
                .get(e5)
                .map(|node| node.addr())
                .unwrap();
            assert_eq!(node, ip5);
        }
    }

    #[test]
    fn test_duplicate_insert() {
        let mut world = World::new();
        world.register::<Node>();
        let mut system = NodeSystem::<Node>::new(&mut world);
        let mut reader_id = world.write_resource::<WorldState<Node>>().track();
        let ip1 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        let _e1 = world.create_entity().with(Node { addr: ip1, name: "first".to_string() }).build();
        let _e2 = world.create_entity().with(Node { addr: ip1, name: "second".to_string() }).build();
        system.run_now(&mut world);
        delete_removals(&mut world, &mut reader_id);
        world.maintain();
        assert_eq!(2, world.read_resource::<WorldState<Node>>().all().len());

        system.run_now(&mut world);
        world.maintain();
        assert_eq!(1, world.read_resource::<WorldState<Node>>().all().len());
        let entity = world.read_resource::<WorldState<Node>>().all()[0];

        let node_name = world.read_storage::<Node>()
            .get(entity)
            .map(|node| node.name())
            .unwrap();
        assert_eq!(node_name, "second".to_string());
    }

    #[test]
    fn test_replacement() {
        let mut world = World::new();
        world.register::<Node>();
        let mut system = NodeSystem::<Node>::new(&mut world);
        let mut reader_id = world.write_resource::<WorldState<Node>>().track();

        let ip1 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        let _e1 = world.create_entity().with(Node { addr: ip1, name: "first".to_string() }).build();
        system.run_now(&mut world);
        world.maintain();

        let e2 = world.create_entity().with(Node { addr: ip1, name: "second".to_string() }).build();
        system.run_now(&mut world);
        delete_removals(&mut world, &mut reader_id);
        world.maintain();
        assert_eq!(1, world.read_resource::<WorldState<Node>>().all().len());

        let entity = world.fetch::<WorldState<Node>>().get_entity(&ip1).unwrap();
        assert_eq!(entity, e2);
    }
}
