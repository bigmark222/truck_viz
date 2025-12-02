fn main() {
    let cylinder = truck_brep::cylinder();
    truck_brep::save_obj(&cylinder, "output/cylinder.obj").unwrap();
    truck_brep::save_step(&cylinder, "output/cylinder.step").unwrap();
    truck_brep::convert_obj_to_gltf("output/cylinder.obj", "output/cylinder.gltf").unwrap();
}
