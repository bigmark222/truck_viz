# Viewer Setup

`truck_viewer` is a Bevy app that loads GLTF assets from the symlinked `assets/` directory.

1. Pointed Bevy's asset folder at `truck_viewer/assets/` (the symlink to the generator output).
2. Chose a default model to load in `truck_viewer/src/lib.rs` (initially `cube.gltf`).
3. Wired up camera controls and a minimal UI to inspect the loaded mesh.
4. Verified the pipeline by running the viewer after generating assets:
   ```sh
   cd truck_viewer
   cargo run
   ```
5. Tuned rendering and controls iteratively while re-exporting geometry as needed.
