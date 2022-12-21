/*
struct ClockConfig {
    pub step: f32,
}

#[derive(Default)]
struct ComponentConfig {
    clock: Option<ClockConfig>,
    radar: Option<crate::components::radar::Radar>,
    controller: Option<ControllerConfig>
    // ...
}
// Lets not over-engineer this from the get-go.
*/

pub mod playground;
pub mod specification;
pub mod loader;
pub mod default;

