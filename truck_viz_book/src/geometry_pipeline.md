# Geometry Generation Pipeline

`truck_brep_example_code` owns the geometry examples and exports them in formats the viewer can load.

1. Chose Truck's BREP primitives and added small example programs (e.g., `bottle`, `cube`) under `examples/`.
2. Pointed all exporters to the shared output folder: `truck_brep_example_code/output/`.
3. Ran examples to produce meshes and intermediate files:
   ```sh
   cargo run --example bottle   # bottle.gltf/obj/bin/step
   cargo run --example cube     # cube.gltf/obj/bin/step
   ```
4. Kept outputs gitignored so new runs stay local and reproducible.
5. Made sure file names stay stable so the viewer can point to a known GLTF (defaults to `cube.gltf`).
