use truck_modeling::*;

/// Build a torus shell with nested rotational sweeps.
pub fn torus() -> Shell {
    let vertex: Vertex = builder::vertex(Point3::new(0.0, 0.0, 1.0));

    let circle: Wire = builder::rsweep(
        &vertex,
        Point3::new(0.0, 0.5, 1.0), // point on rotation axis
        Vector3::unit_x(),          // axis direction
        Rad(7.0),                   // > 2π ensures closure
    );

    builder::rsweep(&circle, Point3::origin(), Vector3::unit_y(), Rad(7.0))
}
