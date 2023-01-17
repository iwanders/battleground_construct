# Battleground Construct

![banner](./media/banner.png)

In this simulation game, it's up to you to write a controller for your team's units; you write this
controller / 'ai' in [Rust](https://www.rust-lang.org/), compile it into a `.wasm` file which you
can load into the battleground viewer or headless simulator. The file is self-contained, so you can
exchange the compiled `.wasm` file with your friends and see who writes the best controller!

## Getting Started

It should be easy to get started, but hard to master. The quick start steps are the following:

1. Ensure you can compile `wasm32-unknown-unknown` with Rust, install this target with:
```
rustup target install wasm32-unknown-unknown
```
2. Clone this repo with;
```
git clone https://github.com/iwanders/battleground_construct
cd battleground_construct
```
3. Build the example controller in [unit_control_example](unit_control_example);
```
cd unit_control_example
cargo build --release
```
This will create the `unit_control_example.wasm` file in the target directory that contains the controller.
4. Change directories to the [battleground_viewer](battleground_viewer) directory and run it using
  the first tutorial as scenario and specify it should load this `.wasm` file for the `player` team with:
```
cd ../battleground_viewer
cargo run --release --features unit_control_wasm -- scenario tutorial_01_driving_forward --team player:../target/wasm32-unknown-unknown/release/unit_control_example.wasm
```
5. The battleground viewer should start and you should see a red tank unit rotate in place.
6. In the example's code, search for `MODULE_TANK_DIFF_DRIVE` and make your tank drive forwards.
  Recompile the controller by running `cargo build --release` from its directory.
  Restart the viewer with the same command as before, it does not have to recompile so this should
  be quick.
7. The tank should drive into the capture area for the point and fireworks should indicate your
   victory.
8. One can also make changes while keeping the viewer running; each time we recompile the `.wasm`
   file, the viewer automatically reloads it without requiring a restart.
9. Further tutorial files are available, in the `battleground_viewer` directory, run 
   `cargo r --release -- scenario list` for a list of tutorials.


## Unit Control

Controlling your unit is hard - period - this is intentional. The goal of this simulation game was
to provide a gamified way for roboticists to learn Rust, as well as to provide the main
author of this project with a reason to learn about [entity component systems](https://en.wikipedia.org/wiki/Entity_component_system)
, if you are looking for something relaxing you can play on the couch, move along. If you are up for
a challenge, read on!

Your unit's controller should implement [`UnitControl`](battleground_unit_control/src/unit_control.rs), 
it's update method gets called periodically.

The controller interacts with your unit through the [`Interface`](battleground_unit_control/src/interface.rs).
The interface provides access to the `modules`, each module can hold multiple `register`s, each
`register` is either an `i32`, `f32` or `Bytes` register. You can read and write registers, some
registers are read only.

The modules are things like:
- [gps](battleground_unit_control/src/modules/gps.rs): provides your unit's position in the world.
- [revolute](battleground_unit_control/src/modules/revolute.rs): joint controller to measure and
  control rotational joints, like the tank turret's yaw or the barrel pitch.
- [cannon](battleground_unit_control/src/modules/cannon.rs): to control firing of the cannon.
- [draw](battleground_unit_control/src/modules/draw.rs): draw lines in the world, these
  lines are shown whenever the unit is selected in the viewer. This is _very_ helpful for debugging.
- [radar](battleground_unit_control/src/modules/radar.rs): to detect other units.
- ... and more , run `cargo doc` and look for the [battleground_unit_control](battleground_unit_control) crate.

It's up to you to write abstractions for these module registers if you feel that is necessary to
control your unit well.

A few more things of note;
- If your controller panics or raises an `Err`, your unit will self destruct and you get a backtrace
  in the console.
- You can shoot (and hit) your own units, you have a radio
[receiver](battleground_unit_control/src/modules/radio_receiver.rs)
and
[transmitter](battleground_unit_control/src/modules/radio_transmitter.rs) to talk between your units
to prevent that.
- Electronic warfare is authorized (and unless prevented in the configuration) the
  units can transmit and receive on the same radio channels as other teams.
- The [radar](battleground_unit_control/src/modules/radar.rs) sees both friendly and unfriendly units.
- Relevant dimensions for units can be accessed through the [battleground_unit_control's units](battleground_unit_control/src/units) module.


## License
License is `BSD-3-Clause`.