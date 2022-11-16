use std::any::TypeId;
use std::marker::PhantomData;

use std::any::Any;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct EntityId(usize);

/// Things in the world.
pub trait Entity {}


// from https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=f0cee315491dc3c3b6b3f467d6a3b072
pub trait AsAny {
    fn as_any_ref(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    fn as_any_box(self: Box<Self>) -> Box<dyn Any>;
}


/// Entities have a component.
pub trait Component : AsAny {

}

impl<T> AsAny for T
where
    T: Any,
{
    // This cast cannot be written in a default implementation so cannot be
    // moved to the original trait without implementing it for every type.
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_any_box(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}



/// Systems operate on components.
pub trait System {
    fn update(&mut self, world: &mut World);
}

#[derive(Default)]
pub struct World {
    index: usize,
    entities: std::collections::HashMap<EntityId, Box<dyn Entity>>,
    components: std::collections::HashMap<std::any::TypeId, Vec<(EntityId, Box<dyn Component>)>>,
}

pub struct ComponentIterator<'a, T: Component + 'static> {
    entries: &'a Vec<(EntityId, Box<dyn Component>)>,
    index: usize,
    phantom: std::marker::PhantomData<T>,
}

// Implement `Iterator` for `Fibonacci`.
// The `Iterator` trait only requires a method to be defined for the `next` element.
impl<'a, T: Component + 'static> Iterator for ComponentIterator<'a, T> {
    type Item = (EntityId, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.index;
        if current >= self.entries.len() {
            return None;
        }
        let record = &self.entries[current];

        let record = record;
        // println!("type id: {:x?}", (&record.1 as &dyn std::any::Any).type_id());
        // println!("req id:  {:x?}",  TypeId::of::<Box<dyn Component>>());
        // let z = (&record.1.as_ref() as &dyn std::any::Any);
        // let value = z.downcast_ref::<Box<dyn Component>>().unwrap();
        self.index = current + 1;
        use std::ops::Deref;
        Some((record.0.clone(), record.1.deref().as_any_ref().downcast_ref::<T>().unwrap()))
        // return None
        // WIP here.
    }
}

impl World {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_entity(&mut self, entity: Box<dyn Entity>) -> EntityId {
        let new_id = self.make_id();
        self.entities.insert(new_id.clone(), entity);
        new_id
    }

    pub fn add_component<C: Component + 'static>(&mut self, entity: EntityId, component: C) {
        let mut v = self.components.get_mut(&TypeId::of::<C>());
        if v.is_none() {
            self.components.insert(TypeId::of::<C>(), vec![]);
            v = self.components.get_mut(&TypeId::of::<C>());
        }
        v.unwrap().push((entity, Box::new(component)));
    }

    pub fn component_iter<'a, C: Component + 'static>(&'a self) -> ComponentIterator<'a, C> {
        let v = self.components.get(&TypeId::of::<C>());
        if v.is_none() {
            panic!("yikes");
        }
        ComponentIterator::<'a, C> {
            entries: v.unwrap(),
            index: 0,
            phantom: PhantomData,
        }
    }

    fn make_id(&mut self) -> EntityId {
        self.index += 1;
        EntityId(self.index)
    }
}

#[derive(Default)]
pub struct Systems {
    systems: Vec<Box<dyn System>>,
}
impl Systems {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }


    pub fn update(&mut self, world: &mut World) {
        for s in self.systems.iter_mut() {
            s.update(world);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct Health(f32);
    impl Component for Health {}

    #[derive(Debug)]
    struct Awesomeness(f32);
    impl Component for Awesomeness {}

    struct Agent {}
    impl Entity for Agent {}

    struct AwesomenessReporter {}
    impl System for AwesomenessReporter {
        fn update(&mut self, world: &mut World) {
            // get iterator for a type.
            for (entity, awesomeness_component) in world.component_iter::<Awesomeness>() {
                println!("Entity: {entity:?} - component: {awesomeness_component:?}");
            }
        }
    }

    #[test]
    fn test_things() {
        let mut world = World::new();

        let player_id = world.add_entity(Box::new(Agent {}));
        world.add_component(player_id.clone(), Health(1.0));
        world.add_component(player_id.clone(), Awesomeness(0.0));

        let monster_id = world.add_entity(Box::new(Agent {}));
        world.add_component(monster_id.clone(), Health(1.0));
        world.add_component(monster_id.clone(), Awesomeness(0.5));

        let mut systems = Systems::new();
        systems.add_system(Box::new(AwesomenessReporter{}));
        systems.update(&mut world);
    }
}
