use std::f64::consts::PI;
use truck_modeling::*;

/// Build a sphere by sweeping a semicircle profile around the Y axis.
pub fn sphere() -> Solid {
    // Semicircle in the YZ plane (radius 1), centered at the origin.
    let v0: Vertex = builder::vertex(Point3::new(0.0, 1.0, 0.0));
    let meridian: Wire = builder::rsweep(&v0, Point3::origin(), Vector3::unit_x(), Rad(PI));

    // Sweep the meridian around the Y axis (> 2π to ensure closure) to form a closed, oriented shell.
    let shell: Shell = builder::cone(&meridian, Vector3::unit_y(), Rad(7.0));
    Solid::new(vec![shell])
}
