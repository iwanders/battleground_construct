use crate::components;
use crate::display;
use battleground_unit_control::units::common;
use battleground_unit_control::units::UnitType;
use components::unit::UnitId;
use components::unit_interface::RegisterInterfaceContainer;
use engine::prelude::*;
use serde::{Deserialize, Serialize};

/// Radio config, for both transmitter and receiver.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct RadioConfig {
    pub channel_min: usize,
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

pub fn add_common_body(
    world: &mut World,
    register_interface: &RegisterInterfaceContainer,
    body_entity: EntityId,
) {
    // ------ Body

    world.add_component(
        body_entity,
        components::radar_reflector::RadarReflector::new(),
    );
    world.add_component(
        body_entity,
        components::capture_marker::CaptureMarker::new(),
    );

    // Lets place drawing and gps in the base as well.
    world.add_component(body_entity, display::draw_module::DrawComponent::new());
    register_interface.get_mut().add_module(
        "draw",
        common::MODULE_DRAW,
        display::draw_module::DrawModule::new(body_entity),
    );
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
