# Battleground Construct

![banner](./media/banner.png)

In this simulation game, it's up to you to write a controller for your team's units; you write this
controller in [Rust](https://www.rust-lang.org/), compile it into a `.wasm` file which you can load
into the battleground viewer or headless simulator. The file is self-contained, so you can also
exchange controllers with your friends and see who writes the best one!

## Getting Started

It should be fairly easy to get started, but hard to master. The quick start steps are the
following:

1. Ensure you can compile `wasm32-unknown-unknown`, install this target with:
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
cargo b --release
```
4. Run the [battleground_viewer](battleground_viewer) using the first tutorial scenario, using
your controller:
```
cd ../battleground_viewer
cargo r --release -- scenario tutorial_01_driving_forward --team player:../target/wasm32-unknown-unknown/release/unit_control_example.wasm
```
5. The battleground viewer should start and you should see a red tank vehicle rotate in place.
6. In the example, search for `MODULE_TANK_DIFF_DRIVE` and make your tank drive forwards.
   Restart the viewer with the same command, it does not have to recompile so this should be quick.
7. The tank should drive into the capture area for the point and fireworks should indicate your
   victory.
8. One can also make changes to keep the viewer running; each time we recompile the `.wasm` file,
   the viewer automatically reloads it without requiring a restart.
9. Further tutorial files are available, run `cargo r --release -- scenario list` for a list.

