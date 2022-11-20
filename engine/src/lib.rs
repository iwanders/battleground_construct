use std::any::TypeId;
use std::marker::PhantomData;

mod as_any;
pub use as_any::AsAny;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct EntityId(usize);

/// Entities have a component.
pub trait Component: AsAny {}

/// Systems operate on components.
pub trait System {
    // I think this should technically be a free function without state, but that's not object safe.
    // So lets just allow mutability for now.
    fn update(&mut self, world: &mut World);
}

pub mod prelude {
    pub use super::{Component, EntityId, System, World};
}

use std::cell::Ref;

#[derive(Default)]
pub struct World {
    index: usize,
    entities: std::collections::HashSet<EntityId>,
    components: std::collections::HashMap<
        std::any::TypeId,
        std::collections::HashMap<EntityId, std::cell::RefCell<Box<dyn Component>>>,
    >,
}

// for vectors; https://stackoverflow.com/a/68737585
pub struct ComponentIterator<'a, T: Component + 'static> {
    entries: Option<std::collections::hash_map::Iter<'a, EntityId, std::cell::RefCell<Box<dyn Component>>>>,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: Component + 'static> Iterator for ComponentIterator<'a, T> {
    type Item = (EntityId, Ref<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.entries.is_none() {
            return None;
        }
        let next = self.entries.as_mut().unwrap().next();
        use std::ops::Deref;

        match next {
            Some(v) => Some((
                v.0.clone(),
                std::cell::Ref::map(v.1.borrow(), |v| {
                    v.deref()
                        .as_any_ref()
                        .downcast_ref::<T>()
                        .expect("Unwrap should succeed")
                }),
            )),
            None => None,
        }
    }
}

use std::cell::RefMut;

pub struct ComponentIteratorMut<'a, T: Component + 'static> {
    entries: Option<std::collections::hash_map::Iter<'a, EntityId, std::cell::RefCell<Box<dyn Component>>>>,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: Component + 'static> Iterator for ComponentIteratorMut<'a, T> {
    type Item = (EntityId, RefMut<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.entries.is_none() {
            return None;
        }

        let next = self.entries.as_mut().unwrap().next();
        use std::ops::DerefMut;

        match next {
            Some(v) => Some((
                v.0.clone(),
                std::cell::RefMut::map(v.1.borrow_mut(), |v| {
                    v.deref_mut()
                        .as_any_mut()
                        .downcast_mut::<T>()
                        .expect("Unwrap should succeed")
                }),
            )),
            None => None,
        }
    }
}
impl World {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_entity(&mut self) -> EntityId {
        let new_id = self.make_id();
        self.entities.insert(new_id.clone());
        new_id
    }

    pub fn add_component<C: Component + 'static>(&mut self, entity: &EntityId, component: C) {
        let mut v = self.components.get_mut(&TypeId::of::<C>());
        if v.is_none() {
            self.components
                .insert(TypeId::of::<C>(), Default::default());
            v = self.components.get_mut(&TypeId::of::<C>());
        }
        let v = v.unwrap();
        v.insert(entity.clone(), std::cell::RefCell::new(Box::new(component)));
    }

    pub fn component_entities<C: Component + 'static>(&self) -> Vec<EntityId> {
        let v = self.components.get(&TypeId::of::<C>());
        if v.is_none() {
            return vec![]
        }
        let v = v.unwrap();
        v.keys().map(|x|{x.clone()}).collect::<_>()
    }

    pub fn component_iter<'a, C: Component + 'static>(&'a self) -> ComponentIterator<'a, C> {
        let v = self.components.get(&TypeId::of::<C>());
        if v.is_none() {
            return ComponentIterator::<'a, C> {
                entries: None,
                phantom: PhantomData,
            };
        }
        ComponentIterator::<'a, C> {
            entries: Some(v.unwrap().iter()),
            phantom: PhantomData,
        }
    }

    pub fn remove_entity(&mut self, entity: &EntityId) {
        if self.entities.remove(entity) {
            for (_t, comps) in self.components.iter_mut() {
                comps.remove(entity);
            }
        }
    }

    /// Method is not const, to allow different component types to be accessed in mutable fashion
    /// at the same time. But it will panic if we're doing a double borrow.
    pub fn component_iter_mut<'a, C: Component + 'static>(&'a self) -> ComponentIteratorMut<'a, C> {
        let v = self.components.get(&TypeId::of::<C>());
        if v.is_none() {
            return 
            ComponentIteratorMut::<'a, C> {
                entries: None,

                phantom: PhantomData,
            };
        }
        ComponentIteratorMut::<'a, C> {
            entries: Some(v.unwrap().iter()),

            phantom: PhantomData,
        }
    }

    pub fn component<'a, C: Component + 'static>(
        &'a self,
        entity: &EntityId,
    ) -> Option<std::cell::Ref<'a, C>> {
        let v = self.components.get(&TypeId::of::<C>());
        if v.is_none() {
            return None;
        }
        use std::ops::Deref;
        let v = v.unwrap().get(&entity);
        match v {
            Some(rc_component) => Some(std::cell::Ref::map(rc_component.borrow(), |v| {
                v.deref()
                    .as_any_ref()
                    .downcast_ref::<C>()
                    .expect("Unwrap should succeed")
            })),
            None => None,
        }
    }
    pub fn component_mut<'a, C: Component + 'static>(
        &'a self,
        entity: &EntityId,
    ) -> Option<std::cell::RefMut<'a, C>> {
        let v = self.components.get(&TypeId::of::<C>());
        if v.is_none() {
            return None;
        }

        use std::ops::DerefMut;
        let v = v.unwrap().get(&entity);
        match v {
            Some(rc_component) => Some(std::cell::RefMut::map(rc_component.borrow_mut(), |v| {
                v.deref_mut()
                    .as_any_mut()
                    .downcast_mut::<C>()
                    .expect("Unwrap should succeed")
            })),
            None => None,
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

    #[derive(Debug)]
    struct Health(f32);
    impl Component for Health {}

    #[derive(Debug)]
    struct Regeneration(f32);
    impl Component for Regeneration {}

    // Adds health based on regeneration component.
    struct HealthPropagator {}
    impl System for HealthPropagator {
        fn update(&mut self, world: &mut World) {
            // Mutable iteration inside immutable one!
            for (entity_awesomeness, regeneration_component) in
                world.component_iter::<Regeneration>()
            {
                for (entity_health, mut health) in world.component_iter_mut::<Health>() {
                    if entity_awesomeness == entity_health {
                        println!("Entity: {entity_awesomeness:?} - adding: {regeneration_component:?} to {health:?}");
                        health.0 += regeneration_component.0;
                    }
                }
            }
        }
    }

    // Removes any entity below 0.0 health.
    struct DeleteLowHealth {}
    impl System for DeleteLowHealth {
        fn update(&mut self, world: &mut World) {
            let mut to_delete: std::collections::HashSet<EntityId> = Default::default();
            for (entity, health) in world.component_iter_mut::<Health>() {
                if health.0 <= 0.0 {
                    to_delete.insert(entity);
                }
            }
            for e in to_delete {
                println!("Removing {e:?}");
                world.remove_entity(&e);
            }
        }
    }

    #[test]
    fn test_things() {
        let mut world = World::new();

        let player_id = world.add_entity();
        world.add_component(&player_id, Health(1.0));
        world.add_component(&player_id, Regeneration(0.0));

        let monster_id = world.add_entity();
        world.add_component(&monster_id, Health(1.0));
        world.add_component(&monster_id, Regeneration(0.5));

        // Test the assignment of values in a component by entity id.
        assert_eq!(world.component::<Regeneration>(&monster_id).unwrap().0, 0.5);
        world.component_mut::<Regeneration>(&monster_id).unwrap().0 = 1.5;
        assert_eq!(world.component::<Regeneration>(&monster_id).unwrap().0, 1.5);

        {
            // Check that we can read another component of the same type by id.
            for (entity_health, mut _health) in world.component_iter_mut::<Health>() {
                if entity_health == player_id {
                    let mut _z = world.component_mut::<Health>(&monster_id);
                }
            }
        }

        let mut systems = Systems::new();

        systems.add_system(Box::new(HealthPropagator {}));
        systems.add_system(Box::new(DeleteLowHealth {}));
        systems.update(&mut world);

        // Check new states.
        assert_eq!(world.component::<Regeneration>(&monster_id).unwrap().0, 1.5);
        assert_eq!(world.component::<Health>(&monster_id).unwrap().0, 2.5);

        assert_eq!(world.component::<Regeneration>(&player_id).unwrap().0, 0.0);
        assert_eq!(world.component::<Health>(&player_id).unwrap().0, 1.0);

        // lets set the monster awesomeness to negative.
        world.component_mut::<Regeneration>(&monster_id).unwrap().0 = -1.0;
        systems.update(&mut world);
        systems.update(&mut world);
        systems.update(&mut world);
        systems.update(&mut world);
    }
}
