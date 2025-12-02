use std::f64::consts::PI;
use truck_modeling::builder;
use truck_modeling::*;

pub fn cylinder(bottom: f64, height: f64, radius: f64) -> Shell {
    let vertex = builder::vertex(Point3::new(0.0, bottom, radius));
    let circle = builder::rsweep(&vertex, Point3::origin(), Vector3::unit_y(), Rad(7.0));
    let disk = builder::try_attach_plane(&vec![circle]).unwrap();
    let solid = builder::tsweep(&disk, Vector3::new(0.0, height, 0.0));
    solid.into_boundaries().pop().unwrap()
}

pub fn body_shell(bottom: f64, height: f64, width: f64, thickness: f64) -> Shell {
    let v0 = builder::vertex(Point3::new(-width / 2.0, bottom, thickness / 4.0));
    let v1 = builder::vertex(Point3::new(width / 2.0, bottom, thickness / 4.0));
    let transit = Point3::new(0.0, bottom, thickness / 2.0);

    let arc0 = builder::circle_arc(&v0, &v1, transit);
    let arc1 = builder::rotated(&arc0, Point3::origin(), Vector3::unit_y(), Rad(PI));

    let face = builder::homotopy(&arc0, &arc1.inverse());
    let solid = builder::tsweep(&face, Vector3::new(0.0, height, 0.0));
    solid.into_boundaries().pop().unwrap()
}

/// Punch a hole in the ceiling and glue neck faces on top.
pub fn glue_body_neck(body: &mut Shell, neck: Shell) {
    let body_ceiling = body.last_mut().unwrap();
    let wire = neck[0].boundaries()[0].clone();

    // This boundary punch is the ceiling hole for the neck.
    body_ceiling.add_boundary(wire);
    body.extend(neck.into_iter().skip(1));
}

/// Hollow bottle with inner cavity and neck.
pub fn bottle(height: f64, width: f64, thickness: f64) -> Solid {
    let mut body = body_shell(-height / 2.0, height, width, thickness);
    let neck = cylinder(height / 2.0, height / 10.0, thickness / 4.0);
    glue_body_neck(&mut body, neck);

    let eps = height / 50.0;
    let mut inner_body = body_shell(
        -height / 2.0 + eps,
        height - 2.0 * eps,
        width - 2.0 * eps,
        thickness - 2.0 * eps,
    );
    let inner_neck = cylinder(
        height / 2.0 - eps,
        height / 10.0 + eps,
        thickness / 4.0 - eps,
    );
    glue_body_neck(&mut inner_body, inner_neck);

    inner_body.face_iter_mut().for_each(|face| {
        face.invert();
    });
    let inner_ceiling = inner_body.pop().unwrap();
    let wire = inner_ceiling.into_boundaries().pop().unwrap();
    let ceiling = body.last_mut().unwrap();
    ceiling.add_boundary(wire);
    body.extend(inner_body.into_iter());

    Solid::new(vec![body])
}
