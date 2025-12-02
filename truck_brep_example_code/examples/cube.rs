fn main() {
    let cube = truck_brep::cube();
    truck_brep::save_obj(&cube, "output/cube.obj").unwrap();
    truck_brep::save_step(&cube, "output/cube.step").unwrap();
    truck_brep::convert_obj_to_gltf("output/cube.obj", "output/cube.gltf").unwrap();
}
