fn main() {
    let bottle = truck_brep::bottle(2.0, 1.0, 0.6);
    truck_brep::save_obj(&bottle, "output/bottle.obj").unwrap();
    truck_brep::save_step(&bottle, "output/bottle.step").unwrap();
    truck_brep::convert_obj_to_gltf("output/bottle.obj", "output/bottle.gltf").unwrap();
}
