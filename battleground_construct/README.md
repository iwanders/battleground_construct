# Battleground Construct

Some notes about the internals of this crate. See the top level readme for user information.

## Architecture notes

- The viewer is completely standalone. The construct does not depend on the viewer.
- `unit_control_wasm` is an optional dependency.
- Most components are in the [components](./src/components/) directory.
- Some components files have multiple components, like `Pose` and `PreTransform`.
- The unit controller interface module for each component exists in the same file.
- Projectiles and their life-flow have further information in the [systems](./src/systems/) directory.
- Components like `Clock` and `MatchFinished` are assumed to be singletons.
- Recording is done by the [recording](./src/components/recording.rs) component. This also handles playback.
- Game setup, cli handling for viewer and construct and parsing of scenarios is done in the [config](./src/config/) module.
- Default systems and creation of singleton components happens in [default.rs](./src/config/default.rs).
- The [display](./src/display/) module holds components that are mostly for visualisation.
- Units are specified in the [units](./src/units/) module.
- The [util](./src/util/) module holds math/matrix helpers and axis aligned bounding box collision helpers.
- All transforms are 4x4 matrices without a scale component, with top left 3x3 being pure rotation, and 3x1 in the top right being pure translation.
- Velocities - in general - are expressed in local frame.
- `Group`; Groups logical entities together.
- `Unit`; Denotes unit type and unit id, each unit has its own component to retrieve the entity ids that it is made up of.
- `Team`; Denotes team name, comment, color, `TeamMember` denotes membership to a team.
- Destroyed units have their health removed, and all entities save for the `unit_entity` are completely deleted.
