# Battleground Viewer

## Control & keyboard shortcuts

- Rotate camera by holding the left mouse button.
- Pan the camera (orthogonal to view direction) by holding the right mouse button.
- Move the camera forwards or backwards with the scroll wheel.
- Select units with the middle mouse button to see what their draw module is drawing. Shift + click allows adding or removing from the selection.
- Space bar pauses the simulation.
- Reload the scenario with 'r', this is helpful if your units destruct because of an accidental panic during development.
- Control + q quits the viewer (non-web only).

## Panels

The top bar always displays what entities are selected (if any). The buttons on the bar at the top
left allow toggling the gui windows. Windows can be closed / moved around / minimized freely.

### Match
The match window shows the general match progress, the time limit, the progress of each team towards
the game objectives. Which team owns the capture point(s). It also shows for each team how many
units it has destroyed. And how many units of each type it currently still has, between parenthesis
is the numbre of estroyed units of that type.

### Time
The time window allows control over the simulation speed. It shows the current elapsed time, the 
calculated realtime factor (this will reduce in order to maintain a goal frame rate of 60 frames per
second in the viewer). It also has a dropdown to control the desired simulation speed.

When playing a recording, this window also allows seeking and scrubbing in the recording. Particles
rendered solely in the viewer may perform odd (non-physically correct) behaviour in face of sudden
time changes, but they should disappear quickly. Initial seeks will require more cpu as the
recording is decompressed.

## In the browser

To run the battleground viewer in the browser, we need to compile it with `wasm-pack`. First install
[wasm-pack](https://github.com/rustwasm/wasm-pack) it, after that, from this directory run:

```
wasm-pack build --release --target web --out-name web
```
This command will compile the viewer and its dependencies for web and creates the `pkg` folder in
this directory.

Next, we need to host the newly created `pkg` directory and the `viewer.html` file from this
directory, one way to do this is with:
```
python3 -m http.server
```

Visit the webserver that was just started at [http://localhost:8000/viewer.html](http://localhost:8000/viewer.html).

Opening this should by default open the scenario `playground`, this is a dummy scenario that is
mainly used during development.

Recordings can be loaded by adding the `url` parameter like so:
[http://localhost:8000/viewer.html?url=recording.bin](http://localhost:8000/viewer.html?url=recording.bin)
, this would try to load recording accessed through `http://localhost:8000/recording.bin`,
`url=foo/recording.bin` would access `http://localhost:8000/foo/recording.bin`, so a relative path
from the host.

The viewer only gets created when the entire recording has been downloaded, so it may look like it
is stuck, but it is just downloading the file before starting the viewer.
