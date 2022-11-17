use std::any::TypeId;
use std::marker::PhantomData;

mod as_any;
pub use as_any::AsAny;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct EntityId(usize);

/// Things in the world.
pub trait Entity {}

/// Entities have a component.
pub trait Component: AsAny {}

/// Systems operate on components.
pub trait System {
    fn update(&mut self, world: &mut World);
}

use std::cell::Ref;

#[derive(Default)]
pub struct World {
    index: usize,
    entities: std::collections::HashMap<EntityId, Box<dyn Entity>>,
    components: std::collections::HashMap<
        std::any::TypeId,
        std::cell::RefCell<Vec<(EntityId, Box<dyn Component>)>>,
    >,
}

// Adapted from https://stackoverflow.com/a/68737585
pub struct ComponentIterator<'a, T: Component + 'static> {
    entries: Option<std::cell::Ref<'a, [(EntityId, Box<dyn Component>)]>>,
    index: usize,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: Component + 'static> Iterator for ComponentIterator<'a, T> {
    type Item = (EntityId, Ref<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.entries.take() {
            Some(borrow) => match *borrow {
                [] => None,
                [_, ..] => {
                    let (head, tail) = Ref::map_split(borrow, |slice| (&slice[0], &slice[1..]));
                    self.entries.replace(tail);
                    Some((
                        head.0.clone(),
                        Ref::map(head, |v| {
                            (*v.1)
                                .as_any_ref()
                                .downcast_ref::<T>()
                                .expect("Unwrap should succeed")
                        }),
                    ))
                }
            },
            None => None,
        }
    }
}
/*
pub struct ComponentIteratorMut<'a, T: Component + 'static> {
    entries: &'a mut Vec<(EntityId, Box<dyn Component>)>,
    index: usize,
    phantom: std::marker::PhantomData<T>,
}


impl<'a, T: Component + 'static> Iterator for ComponentIteratorMut<'a, T> {
    type Item = (EntityId, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.index;
        if current >= self.entries.len() {
            return None;
        }


        let record = self.entries.get_mut(current).unwrap();


        self.index = current + 1;
        use std::ops::Deref;
        use std::ops::DerefMut;

        Some((
            record.0.clone(),
            record.1.deref_mut().as_any_mut().downcast_mut::<T>().unwrap(),
        ))
    }
}*/

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
            self.components
                .insert(TypeId::of::<C>(), std::cell::RefCell::new(vec![]));
            v = self.components.get_mut(&TypeId::of::<C>());
        }
        v.unwrap().get_mut().push((entity, Box::new(component)));
    }

    pub fn component_iter<'a, C: Component + 'static>(&'a self) -> ComponentIterator<'a, C> {
        let v = self.components.get(&TypeId::of::<C>());
        if v.is_none() {
            panic!("yikes");
        }
        ComponentIterator::<'a, C> {
            entries: Some(Ref::map(v.unwrap().borrow(), |v| &v[..])),
            index: 0,
            phantom: PhantomData,
        }
    }

    /*
    pub fn component_iter_mut<'a, C: Component + 'static>(&'a mut self) -> ComponentIteratorMut<'a, C> {
        let v = self.components.get_mut(&TypeId::of::<C>());
        if v.is_none() {
            panic!("yikes");
        }
        ComponentIteratorMut::<'a, C> {
            entries: v.unwrap(),
            index: 0,
            phantom: PhantomData,
        }
    }
    */

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
        systems.add_system(Box::new(AwesomenessReporter {}));
        systems.update(&mut world);
    }
}
