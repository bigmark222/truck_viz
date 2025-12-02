fn main() {
    let torus = truck_brep::torus();
    truck_brep::save_obj(&torus, "output/torus.obj").unwrap();
    truck_brep::save_step(&torus, "output/torus.step").unwrap();
    truck_brep::convert_obj_to_gltf("output/torus.obj", "output/torus.gltf").unwrap();
}
