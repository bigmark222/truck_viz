fn main() {
    let sphere = truck_brep::sphere();
    truck_brep::save_obj(&sphere, "output/sphere.obj").unwrap();
    truck_brep::save_step(&sphere, "output/sphere.step").unwrap();
    truck_brep::convert_obj_to_gltf("output/sphere.obj", "output/sphere.gltf").unwrap();
}
