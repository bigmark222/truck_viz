fn main() {
    let shape = truck_brep::organic();
    truck_brep::save_obj(&shape, "output/organic.obj").unwrap();
    truck_brep::save_step(&shape, "output/organic.step").unwrap();
    truck_brep::convert_obj_to_gltf("output/organic.obj", "output/organic.gltf").unwrap();
}
