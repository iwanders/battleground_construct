#[allow(clippy::module_inception)]
mod construct_render;
mod effects;
mod instanced_entity;
mod render;

pub use construct_render::ConstructRender;
pub use render::RenderPass;

pub mod util;
