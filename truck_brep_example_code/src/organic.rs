use truck_modeling::*;

/// Generate a smooth, organic shape: a tapered main stem with two side shoots.
pub fn organic() -> Solid {
    // Helper: build a circular wire of given radius centered at `center`.
    fn circle_wire(center: Point3, radius: f64) -> Wire {
        let v = builder::vertex(center + Vector3::new(radius, 0.0, 0.0));
        builder::translated(
            &builder::rsweep(
                &v,
                center,
                Vector3::unit_y(),
                Rad(std::f64::consts::PI * 2.0),
            ),
            Vector3::new(0.0, 0.0, 0.0),
        )
    }

    // Helper: loft a tube through successive stations (center, radius), with caps.
    fn tube(stations: &[(Point3, f64)]) -> Solid {
        assert!(stations.len() >= 2, "need at least two stations");

        let mut faces: Vec<Face> = Vec::new();
        let mut wires = Vec::new();
        for (center, r) in stations {
            wires.push(circle_wire(*center, *r));
        }

        // Loft between consecutive wires.
        for pair in wires.windows(2) {
            let shell =
                builder::try_wire_homotopy(&pair[0], &pair[1]).expect("failed to loft segment");
            faces.extend(shell.into_iter());
        }

        // Caps: bottom faces should point -Y, top faces +Y.
        let bottom_face =
            builder::try_attach_plane(&vec![wires.first().unwrap().clone()]).expect("cap bottom");
        let top_face =
            builder::try_attach_plane(&vec![wires.last().unwrap().clone()]).expect("cap top");
        faces.push(bottom_face.inverse());
        faces.push(top_face);

        let shell: Shell = faces.into();
        Solid::new(vec![shell])
    }

    // Main stem: gently curving up and back.
    let stem_path = [
        (Point3::new(0.0, 0.0, 0.0), 1.0),
        (Point3::new(0.4, 1.8, 0.3), 0.85),
        (Point3::new(-0.2, 3.6, 0.8), 0.7),
        (Point3::new(0.0, 5.5, 1.2), 0.55),
    ];
    let stem = tube(&stem_path);

    // Side shoot A: peels off forward/right.
    let shoot_a_path = [
        (Point3::new(0.0, 3.0, 0.8), 0.4),
        (Point3::new(0.8, 4.5, 1.6), 0.3),
        (Point3::new(1.2, 6.0, 2.0), 0.2),
    ];
    let shoot_a = tube(&shoot_a_path);

    // Side shoot B: peels off left/back.
    let shoot_b_path = [
        (Point3::new(-0.2, 2.8, 0.6), 0.35),
        (Point3::new(-1.0, 4.0, -0.2), 0.25),
        (Point3::new(-1.6, 5.5, -0.5), 0.18),
    ];
    let shoot_b = tube(&shoot_b_path);

    // Merge shells from all parts.
    let mut shells = Vec::new();
    shells.extend(stem.boundaries().clone());
    shells.extend(shoot_a.boundaries().clone());
    shells.extend(shoot_b.boundaries().clone());
    Solid::new(shells)
}
