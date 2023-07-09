use crate::components;
use battleground_unit_control::units::common;
use battleground_unit_control::units::UnitType;
use components::unit::UnitId;
use components::unit_interface::RegisterInterfaceContainer;
use engine::prelude::*;
use serde::{Deserialize, Serialize};

/// Radio config, for both transmitter and receiver.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct RadioConfig {
    /// The minimum selectable channel for the radio, both receive and transmit.
    pub channel_min: usize,
    /// The maximum selectable channel for the radio, both receive and transmit.
    pub channel_max: usize,
}

pub fn add_common_global(register_interface: &RegisterInterfaceContainer) {
    // -----    Global modules
    register_interface.get_mut().add_module(
        "clock",
        common::MODULE_CLOCK,
        components::clock::ClockModule::new(),
    );
    register_interface.get_mut().add_module(
        "objectives",
        common::MODULE_OBJECTIVES,
        components::objectives_module::ObjectivesModule::new(),
    );
}

pub fn add_common_unit(
    world: &mut World,
    register_interface: &RegisterInterfaceContainer,
    unit_entity: EntityId,
    unit_type: UnitType,
) -> UnitId {
    // -----   Unit
    world.add_component(unit_entity, components::health::Health::new());
    world.add_component(unit_entity, components::eternal::Eternal::new());
    register_interface.get_mut().add_module(
        "team",
        common::MODULE_TEAM,
        components::team_module::TeamModule::new(unit_entity),
    );

    let unit_component =
        components::unit::Unit::new(components::id_generator::generate_id(world), unit_type);
    let unit_id = unit_component.id();
    world.add_component(unit_entity, unit_component);

    register_interface.get_mut().add_module(
        "unit",
        common::MODULE_UNIT,
        components::unit::UnitModuleComponent::new(unit_entity),
    );

    unit_id
}

pub fn add_common_diff_drive(
    world: &mut World,
    register_interface: &RegisterInterfaceContainer,
    base_entity: EntityId,
    config: components::differential_drive_base::DifferentialDriveConfig,
    module_id: u32,
) {
    world.add_component(base_entity, components::velocity::Velocity::new());
    world.add_component(
        base_entity,
        components::differential_drive_base::DifferentialDriveBase::from_config(config),
    );
    register_interface.get_mut().add_module(
        "diff_drive",
        module_id,
        components::differential_drive_base::DifferentialDriveBaseModule::new(base_entity),
    );
}

pub fn add_common_tricycle(
    world: &mut World,
    register_interface: &RegisterInterfaceContainer,
    base_entity: EntityId,
    steer_entity: EntityId,
    config: components::tricycle_base::TricycleConfig,
    module_id: u32,
) {
    world.add_component(base_entity, components::velocity::Velocity::new());
    world.add_component(
        base_entity,
        components::tricycle_base::TricycleBase::new(config, steer_entity),
    );
    register_interface.get_mut().add_module(
        "tricycle",
        module_id,
        components::tricycle_base::TricycleBaseModule::new(base_entity),
    );
}

pub fn add_common_body(
    world: &mut World,
    register_interface: &RegisterInterfaceContainer,
    reflectivity: f32,
    body_entity: EntityId,
) {
    // ------ Body

    world.add_component(
        body_entity,
        components::radar_reflector::RadarReflector::new(reflectivity),
    );
    world.add_component(
        body_entity,
        components::capture_marker::CaptureMarker::new(),
    );

    // Lets place drawing and gps in the base as well.
    register_interface.get_mut().add_module(
        "localization",
        common::MODULE_GPS,
        components::gps::GpsModule::new(body_entity),
    );
}

pub fn add_radio_receiver_transmitter(
    world: &mut World,
    register_interface: &RegisterInterfaceContainer,
    body_entity: EntityId,
    radio_config: Option<super::common::RadioConfig>,
) {
    // Radios are also on the body, because the gps is also there.
    let transmitter_config = radio_config
        .map(|v| components::radio_transmitter::RadioTransmitterConfig {
            channel_min: v.channel_min,
            channel_max: v.channel_max,
            ..Default::default()
        })
        .unwrap_or_default();
    world.add_component(
        body_entity,
        components::radio_transmitter::RadioTransmitter::new_with_config(transmitter_config),
    );
    register_interface.get_mut().add_module(
        "radio_transmitter",
        common::MODULE_RADIO_TRANSMITTER,
        components::radio_transmitter::RadioTransmitterModule::new(body_entity),
    );

    let receiver_config = radio_config
        .map(|v| components::radio_receiver::RadioReceiverConfig {
            channel_min: v.channel_min,
            channel_max: v.channel_max,
            ..Default::default()
        })
        .unwrap_or_default();
    world.add_component(
        body_entity,
        components::radio_receiver::RadioReceiver::new_with_config(receiver_config),
    );
    register_interface.get_mut().add_module(
        "radio_receiver",
        common::MODULE_RADIO_RECEIVER,
        components::radio_receiver::RadioReceiverModule::new(body_entity),
    );
}

pub fn add_revolute(
    world: &mut World,
    register_interface: &RegisterInterfaceContainer,
    entity: EntityId,
    name: &str,
    module_id: u32,
    revolute_config: components::revolute::RevoluteConfig,
) {
    register_interface.get_mut().add_module(
        name,
        module_id,
        components::revolute::RevoluteModule::new(entity),
    );
    let revolute = components::revolute::Revolute::from_config(revolute_config);
    world.add_component(entity, revolute);
    world.add_component(entity, components::pose::Pose::new());
    world.add_component(entity, components::velocity::Velocity::new());
}
pub fn add_revolute_pair(world: &mut World, entity: EntityId, pair_entity: EntityId) {
    world.add_component(
        entity,
        components::revolute_pair::RevolutePair::new(pair_entity),
    );
    world.add_component(entity, components::pose::Pose::new());
    world.add_component(entity, components::velocity::Velocity::new());
}

pub fn add_radar(
    world: &mut World,
    register_interface: &RegisterInterfaceContainer,
    radar_entity: EntityId,
    name: &str,
    module_id: u32,
    radar_config: components::radar::RadarConfig,
) {
    register_interface.get_mut().add_module(
        name,
        module_id,
        components::radar::RadarModule::new(radar_entity),
    );
    world.add_component(
        radar_entity,
        components::radar::Radar::new_with_config(radar_config),
    );
}

pub fn add_group_team_unit(
    world: &mut World,
    unit: &dyn super::Unit,
    team: Option<components::team_member::TeamMember>,
) {
    use crate::components::group::Group;
    // Add the group, unit and team membership to each of the component.
    // Unit must be first in the group!
    let mut constructor_group_entities: Vec<EntityId> = vec![unit.unit_entity()];
    constructor_group_entities.append(&mut unit.children());

    let group = Group::from(&constructor_group_entities);
    for e in constructor_group_entities.iter() {
        world.add_component(*e, group.clone());
        world.add_component(*e, components::unit_member::UnitMember::new(unit.unit_id()));
        // This feels a bit like a crux... but it's cheap and easy.
        if let Some(team_member) = team {
            world.add_component(*e, team_member);
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct ComponentBox {
    /// The base entity to which the box is attached at the center.
    pub base: EntityId,
    /// The entity to which the lid is attached, it also has the revolute joint.
    pub lid: EntityId,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct ComponentBoxSpawnConfig {
    pub width: f32,
    pub length: f32,
    pub height: f32,
}

pub fn add_component_box(world: &mut World, config: ComponentBoxSpawnConfig) -> ComponentBox {
    use crate::components::parent::Parent;
    use crate::components::pose::PreTransform;
    use crate::display::primitives::Vec3;
    let base = world.add_entity();

    let component_box = crate::display::component_box::ComponentBox::from_config(config);
    let lid_hinge_offset = component_box.lid_hinge_offset();
    let hitboxes = component_box.hit_collection();
    world.add_component(base, hitboxes);
    world.add_component(base, component_box);

    let lid = world.add_entity();
    let lid_box = crate::display::component_box_lid::ComponentBoxLid::from_config(config);

    let revolute_config = components::revolute::RevoluteConfig {
        axis: Vec3::new(-1.0, 0.0, 0.0),
        velocity_bounds: (-0.75, 0.75),
        acceleration_bounds: Some((-1.0, 1.0)),
        velocity_cmd: 0.1,
        ..Default::default()
    };

    world.add_component(lid, lid_box);
    let revolute = components::revolute::Revolute::from_config(revolute_config);
    world.add_component(lid, revolute);
    world.add_component(lid, components::pose::Pose::new());
    world.add_component(lid, components::velocity::Velocity::new());

    world.add_component(lid, PreTransform::from_translation(lid_hinge_offset));
    world.add_component(lid, Parent::new(base));

    ComponentBox { base, lid }
}
