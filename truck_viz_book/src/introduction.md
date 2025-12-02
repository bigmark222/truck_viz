# Introduction

This book documents how we assembled the `truck_viz` workspace. It is written as a build log: terse, chronological, and focused on the exact steps and decisions that got the viewer and geometry pipeline running.

## Why this exists
- Keep the geometry generator (`truck_brep_example_code`) and the Bevy viewer (`truck_viewer`) reproducible for anyone who clones the repo.
- Record the commands and conventions (paths, file names, env vars) so we do not rediscover them every time we add a new example.
- Serve as a tour for new collaborators who want to tweak geometry exports or the viewer without breaking the pipeline.

## What you need
- Rust (stable) with `cargo` installed.
- `mdbook` if you want to build/serve this book locally (`cargo install mdbook`).
- A filesystem that supports symlinks, or the ability to create a directory junction on Windows.

## How to read
1. Start with the workspace overview to understand how the two crates relate.
2. Follow the geometry pipeline chapter to produce sample GLTF/OBJ files.
3. Wire up the viewer and confirm it loads a known asset.
4. Turn on hot-reload to iterate quickly on geometry and materials.

## Build or serve the book
- Build once: `cd truck_viz_book && mdbook build` (outputs to `truck_viz_book/book/`).
- Live preview: `mdbook serve -o` from the same directory to open a local preview while editing.
