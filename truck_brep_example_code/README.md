# truck_brep

Small collection of Truck-based solid modeling helpers that build a few demo shapes (cube, torus, cylinder, bottle) and export them as OBJ or STEP.

## Run an example

```bash
cargo run --example torus
```

This writes `output/torus.obj` and `output/torus.step`. The `output/` directory stays in git via `.gitkeep`, but the generated files are ignored.

## Library quickstart

```rust
use truck_brep::{save_obj, save_step, torus};

fn main() -> std::io::Result<()> {
    let shape = torus();
    save_obj(&shape, "output/torus.obj")?;
    save_step(&shape, "output/torus.step")?;
    Ok(())
}
```
