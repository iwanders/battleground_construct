use crate::components;
use engine::prelude::*;
use engine::Systems;

pub struct Construct {
    pub world: World,
    pub systems: Systems,
}

#[allow(clippy::new_without_default)]
impl Construct {
    pub fn new() -> Self {
        let world = World::new();
        let systems = engine::Systems::new();
        Construct { world, systems }
    }

    pub fn can_update(&self) -> bool {
        self.world
            .component_iter::<components::recording::PlaybackFinishedMarker>()
            .next()
            .is_none()
    }

    pub fn update(&mut self) {
        self.systems.update(&mut self.world);
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn entity_pose(&self, entity: EntityId) -> components::pose::Pose {
        components::pose::world_pose(&self.world, entity)
    }

    /// If it is a recording, return the max time.
    pub fn recording_max_time(&self) -> Option<f32> {
        let (_, recording) = self
            .world
            .component_iter::<components::recording::Recording>()
            .next()?;
        recording.record().borrow().max_time()
    }

    /// Sketchy seek functionality.
    pub fn recording_seek(&mut self, time: f32) {
        // Obtain the recording, then obliterate the world, then perform the seek, effectively
        // reinstantiating it.
        let record_entry = self
            .world
            .component_entities::<components::recording::Recording>();
        if let Some(entity) = record_entry.first() {
            let recording = self
                .world
                .remove_component::<components::recording::Recording>(*entity)
                .unwrap();
            let record = recording.record();
            // Nuke the world...
            self.world = Default::default();
            // Add the recorder back.
            let recorder_entity = self.world.add_entity();
            self.world.add_component_boxed(recorder_entity, recording);

            // Perform the seek.
            {
                record.borrow_mut().seek(time);
            }
            // Apply the state.
            record.borrow().apply_state(&mut self.world);
            self.update(); // one update cycle to spawn vehicles, to ensure this works while paused.
        }
    }

    pub fn elapsed_as_f32(&self) -> f32 {
        let (_entity, clock) = self
            .world
            .component_iter_mut::<crate::components::clock::Clock>()
            .next()
            .expect("should have a clock, are default components added?");
        clock.elapsed_as_f32()
    }

    pub fn is_match_finished(&self) -> bool {
        !self
            .world
            .component_entities::<crate::components::match_finished::MatchFinished>()
            .is_empty()
    }

    // We could have something fancy here... where we generalize this over 'has ray intersect'...
    pub fn select_intersect(
        &mut self,
        pos: &cgmath::Vector3<f32>,
        dir: &cgmath::Vector3<f32>,
    ) -> Vec<EntityId> {
        use crate::components::pose::world_pose;
        use crate::util::box_collision::AxisAlignedBox;
        use crate::util::cgmath::prelude::*;
        let select_box_with_pose = {
            let selectboxes = self
                .world
                .component_iter::<components::select_box::SelectBox>();
            selectboxes
                .map(|(entity, selectbox)| {
                    let pose = world_pose(&self.world, entity);
                    (entity, pose, *selectbox)
                })
                .collect::<Vec<_>>()
        };
        // Make direction long, our is_intersecing works on line segments, not rays.
        let dir = dir * 10000.0; // 10 km ought to be enough for anyone...
        let p1 = pos + dir;

        const PLOT_DEBUG: bool = false;

        if PLOT_DEBUG {
            // Optionally, we can sprinkle all rays into the world here...
            let z = self.world.add_entity();
            let mut l = crate::display::debug_lines::DebugLines::new();
            l.add_line(
                crate::display::primitives::Line {
                    p0: (pos.x, pos.y, pos.z),
                    p1: (p1.x, p1.y, p1.z),
                    width: 0.01,
                },
                crate::display::primitives::Color::RED,
            );
            self.world.add_component(z, l);
        }

        let mut v = vec![];
        for (entity, select_box_pose, select_box) in select_box_with_pose.iter() {
            let pos_in_box_frame =
                (select_box_pose.transform().to_inv_h() * pos.to_h()).to_translation();
            let p1_in_box_frame =
                (select_box_pose.transform().to_inv_h() * p1.to_h()).to_translation();
            let b =
                AxisAlignedBox::new(select_box.length(), select_box.width(), select_box.height());
            let have_intersection = b.is_intersecting(pos_in_box_frame, p1_in_box_frame);
            if have_intersection {
                v.push(*entity);
                if PLOT_DEBUG {
                    let intersections = b.intersections(pos_in_box_frame, p1_in_box_frame).unwrap();
                    // parametrized in one coordinate system is also parametrized in another if the
                    // spaces are uniform. So don't need to transform here.
                    for n in [intersections.0, intersections.1] {
                        let z = self.world.add_entity();
                        let l = crate::display::debug_box::DebugBox::cube(0.1);
                        let diff = p1 - pos;
                        let intersect_pos = pos + diff * n;
                        self.world.add_component(
                            z,
                            components::pose::Pose::from_translation(intersect_pos),
                        );
                        self.world.add_component(z, l);
                    }
                }
            }
        }
        v
    }
}
