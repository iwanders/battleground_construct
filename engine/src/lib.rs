use std::any::TypeId;
use std::marker::PhantomData;

mod as_any;
pub use as_any::AsAny;

/// An entity is represented by this id.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntityId(usize);

impl From<EntityId> for usize {
    fn from(v: EntityId) -> usize {
        v.0
    }
}

/// Entities have a component.
pub trait Component: AsAny {}

/// Systems operate on components.
pub trait System {
    /// Update function for a system, technically, systems should probably be free functions and as
    /// such take a non-mutable self, but sometimes changing self can be helpful
    /// (for example toggling logging), so lets allow mutability on self.
    fn update(&mut self, world: &mut World);
}

/// Prelude for importing the necessities.
pub mod prelude {
    pub use super::{Component, EntityId, System, World};
}

use std::cell::Ref;

/// The world contains the entities and components. Components are ordered by type id, then keyed
/// on entity. The world does allow interior mutability, but only on different component types.
/// Performing two mutable iterations over the same component type is a logic error and will panic.
/// Performing a non-mutable borrow and a mutable borrow on the same component type is also a logic
/// error and will panic.
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
/// Component iterator.
pub struct ComponentIterator<'a, T: Component + 'static> {
    entries: Option<
        std::collections::hash_map::Iter<'a, EntityId, std::cell::RefCell<Box<dyn Component>>>,
    >,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: Component + 'static> Iterator for ComponentIterator<'a, T> {
    type Item = (EntityId, Ref<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.entries.as_mut()?.next();
        use std::ops::Deref;

        next.map(|v| {
            (
                *v.0,
                std::cell::Ref::map(v.1.borrow(), |v| {
                    v.deref()
                        .as_any_ref()
                        .downcast_ref::<T>()
                        .expect("Unwrap should succeed")
                }),
            )
        })
    }
}

use std::cell::RefMut;

/// Mutable component iterator.
pub struct ComponentIteratorMut<'a, T: Component + 'static> {
    entries: Option<
        std::collections::hash_map::Iter<'a, EntityId, std::cell::RefCell<Box<dyn Component>>>,
    >,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: Component + 'static> Iterator for ComponentIteratorMut<'a, T> {
    type Item = (EntityId, RefMut<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.entries.as_mut()?.next();
        use std::ops::DerefMut;

        next.map(|v| {
            (
                *v.0,
                std::cell::RefMut::map(v.1.borrow_mut(), |v| {
                    v.deref_mut()
                        .as_any_mut()
                        .downcast_mut::<T>()
                        .expect("Unwrap should succeed")
                }),
            )
        })
    }
}

impl World {
    /// Create a new empty world.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a new entity to the entity list, return its new id.
    pub fn add_entity(&mut self) -> EntityId {
        let new_id = self.make_entity_id();
        self.entities.insert(new_id);
        new_id
    }

    /// Add a component to an entity.
    pub fn add_component<C: Component + 'static>(&mut self, entity: EntityId, component: C) {
        self.add_component_boxed(entity, Box::new(component));
    }

    /// Add a boxed component to an entity.
    pub fn add_component_boxed<C: Component + 'static>(
        &mut self,
        entity: EntityId,
        component: Box<C>,
    ) {
        let mut v = self.components.get_mut(&TypeId::of::<C>());
        if v.is_none() {
            self.components
                .insert(TypeId::of::<C>(), Default::default());
            v = self.components.get_mut(&TypeId::of::<C>());
        }
        let v = v.unwrap();
        v.insert(entity, std::cell::RefCell::new(component));
    }

    /// Return a list of all entities that have a particular component.
    pub fn component_entities<C: Component + 'static>(&self) -> Vec<EntityId> {
        let v = self.components.get(&TypeId::of::<C>());
        if v.is_none() {
            return vec![];
        }
        let v = v.unwrap();
        v.keys().copied().collect::<_>()
    }

    /// Iterate over all (entity, component) of a particular component type.
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

    /// Remove the specified component from the provided entities. Returns the vector of returned
    /// components. Returned vector is guaranteed to be the same size as entities input.
    pub fn remove_components<C: Component + 'static>(
        &mut self,
        entities: &[EntityId],
    ) -> Vec<Option<Box<C>>> {
        let v = self.components.get_mut(&TypeId::of::<C>());
        if v.is_none() {
            return vec![];
        }
        let v = v.unwrap();
        let mut res = vec![];
        for entity in entities.iter() {
            res.push(v.remove(entity).map(|old_entry| {
                std::cell::RefCell::into_inner(old_entry)
                    .as_any_box()
                    .downcast::<C>()
                    .unwrap()
            }));
        }
        res
    }

    /// Remove a specific component from an entity, returning it if it was present.
    pub fn remove_component<C: Component + 'static>(&mut self, entity: EntityId) -> Option<Box<C>> {
        let v = self.components.get_mut(&TypeId::of::<C>())?;
        v.remove(&entity).map(|v| {
            std::cell::RefCell::into_inner(v)
                .as_any_box()
                .downcast::<C>()
                .unwrap()
        })
    }

    /// Move a component from one entity to another.
    pub fn move_component<C: Component + 'static>(&mut self, src: EntityId, dst: EntityId) {
        let v = self.remove_component::<C>(src);
        if let Some(v) = v {
            self.add_component_boxed(dst, v);
        }
    }

    /// Remove an entity.
    pub fn remove_entity(&mut self, entity: EntityId) {
        if self.entities.remove(&entity) {
            for (_t, comps) in self.components.iter_mut() {
                comps.remove(&entity);
            }
        }
    }

    /// Remove multiple entities.
    pub fn remove_entities(&mut self, entities: &[EntityId]) {
        for entity in entities.iter() {
            self.remove_entity(*entity)
        }
    }

    /// Return the current entity count. This includes any entities that ended up having no
    /// components attached to them, but weren't removed.
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Iterate over a specific component type in mutable fashion.
    /// Method is not const, to allow different component types to be accessed in mutable fashion
    /// at the same time. But it will panic if we're doing a double borrow.
    pub fn component_iter_mut<'a, C: Component + 'static>(&'a self) -> ComponentIteratorMut<'a, C> {
        let v = self.components.get(&TypeId::of::<C>());
        if v.is_none() {
            return ComponentIteratorMut::<'a, C> {
                entries: None,

                phantom: PhantomData,
            };
        }
        ComponentIteratorMut::<'a, C> {
            entries: Some(v.unwrap().iter()),

            phantom: PhantomData,
        }
    }

    /// Obtain a specific component from an entity, None if the entity didn't have the component.
    pub fn component<C: Component + 'static>(&self, entity: EntityId) -> Option<std::cell::Ref<C>> {
        let v = self.components.get(&TypeId::of::<C>())?;

        use std::ops::Deref;
        let v = v.get(&entity);
        v.map(|rc_component| {
            std::cell::Ref::map(rc_component.borrow(), |v| {
                v.deref()
                    .as_any_ref()
                    .downcast_ref::<C>()
                    .expect("Unwrap should succeed")
            })
        })
    }

    /// Mutably obtain a specific component from an entity, None if the entity didn't have the
    /// component.
    pub fn component_mut<C: Component + 'static>(
        &self,
        entity: EntityId,
    ) -> Option<std::cell::RefMut<C>> {
        let v = self.components.get(&TypeId::of::<C>())?;

        use std::ops::DerefMut;
        let v = v.get(&entity);
        v.map(|rc_component| {
            std::cell::RefMut::map(rc_component.borrow_mut(), |v| {
                v.deref_mut()
                    .as_any_mut()
                    .downcast_mut::<C>()
                    .expect("Unwrap should succeed")
            })
        })
    }

    /// Make a new entity id, private function to ensure the entity ids are unique.
    fn make_entity_id(&mut self) -> EntityId {
        self.index += 1;
        EntityId(self.index)
    }
}

/// Systems, a container to hold and organise multiple systems.
#[derive(Default)]
pub struct Systems {
    systems: Vec<Box<dyn System>>,
}
impl Systems {
    /// Create a new empty systems container.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a system to the systems container.
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    /// Run all systems in the order by which they were added on the world.
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
                world.remove_entity(e);
            }
        }
    }

    #[test]
    fn test_things() {
        let mut world = World::new();

        let player_id = world.add_entity();
        world.add_component(player_id, Health(1.0));
        world.add_component(player_id, Regeneration(0.0));

        let monster_id = world.add_entity();
        world.add_component(monster_id, Health(1.0));
        world.add_component(monster_id, Regeneration(0.5));

        // Test the assignment of values in a component by entity id.
        assert_eq!(world.component::<Regeneration>(monster_id).unwrap().0, 0.5);
        world.component_mut::<Regeneration>(monster_id).unwrap().0 = 1.5;
        assert_eq!(world.component::<Regeneration>(monster_id).unwrap().0, 1.5);

        {
            // Check that we can read another component of the same type by id.
            for (entity_health, mut _health) in world.component_iter_mut::<Health>() {
                if entity_health == player_id {
                    let mut _z = world.component_mut::<Health>(monster_id);
                }
            }
        }

        let mut systems = Systems::new();

        systems.add_system(Box::new(HealthPropagator {}));
        systems.add_system(Box::new(DeleteLowHealth {}));
        systems.update(&mut world);

        // Check new states.
        assert_eq!(world.component::<Regeneration>(monster_id).unwrap().0, 1.5);
        assert_eq!(world.component::<Health>(monster_id).unwrap().0, 2.5);

        assert_eq!(world.component::<Regeneration>(player_id).unwrap().0, 0.0);
        assert_eq!(world.component::<Health>(player_id).unwrap().0, 1.0);

        // lets set the monster awesomeness to negative.
        world.component_mut::<Regeneration>(monster_id).unwrap().0 = -1.0;
        systems.update(&mut world);
        systems.update(&mut world);
        systems.update(&mut world);
        systems.update(&mut world);
    }
}
