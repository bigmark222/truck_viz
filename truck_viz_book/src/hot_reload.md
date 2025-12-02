# Hot Reload Loop

We leaned on Bevy's asset hot-reload to shorten the edit-test cycle.

1. Enabled polling-based reloads so filesystem events work across platforms: `BEVY_ASSET_WATCHER=poll`.
2. Launched the viewer with hot-reload on:
   ```sh
   cd truck_viewer
   BEVY_ASSET_WATCHER=poll cargo run
   ```
3. Re-exported geometry from `truck_brep_example_code` into `output/`; the symlinked `assets/` folder picked up the changes.
4. Watched the viewer reload GLTF/OBJ outputs without restarting, iterating on geometry and materials quickly.
