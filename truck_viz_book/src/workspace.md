# Workspace Layout

We split the project into two crates under a single workspace so geometry generation and the viewer stay decoupled.

1. Created the workspace directory `truck_viz/` with two sub-crates: `truck_brep_example_code/` (geometry generation) and `truck_viewer/` (Bevy viewer).
2. Kept generated assets out of git by writing to `truck_brep_example_code/output/` and ignoring it in `.gitignore`.
3. Linked the viewer to those assets via a symlink: `ln -s ../truck_brep_example_code/output truck_viewer/assets` (or an equivalent directory junction on Windows).
4. Documented the flow in `README.md` so new clones can set up the symlink before running anything else.
