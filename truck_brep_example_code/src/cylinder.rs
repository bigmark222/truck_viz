use truck_modeling::*;

pub fn cylinder() -> Solid {
    // Base on the print bed (z = 0), standing upright along +Z.
    let vertex: Vertex = builder::vertex(Point3::new(0.0, 0.0, 0.0));
    let circle: Wire = builder::rsweep(
        &vertex,
        Point3::new(0.0, 1.0, 0.0),
        Vector3::unit_z(),
        Rad(7.0),
    );
    let disk: Face = builder::try_attach_plane(&vec![circle]).expect("cannot attach plane");
    builder::tsweep(&disk, 10.0 * Vector3::unit_z())
}
