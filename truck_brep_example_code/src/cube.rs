use truck_modeling::*;

/// Build a solid cube by chaining translational sweeps.
pub fn cube() -> Solid {
    let vertex: Vertex = builder::vertex(Point3::new(-1.0, 0.0, -1.0));
    let edge: Edge = builder::tsweep(&vertex, 2.0 * Vector3::unit_z());
    let face: Face = builder::tsweep(&edge, 2.0 * Vector3::unit_x());
    builder::tsweep(&face, 2.0 * Vector3::unit_y())
}
