use std::{fs, io, path::{Path, PathBuf}};

use serde_json::json;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_stepio::out::{CompleteStepDisplay, StepModel};
use truck_topology::compress::{CompressedShell, CompressedSolid};

/// Default chord tolerance for triangulating curved surfaces.
/// Smaller values produce rounder meshes at the cost of more triangles.
const MIN_TRIANGULATION_TOLERANCE: f64 = 5.0e-6;
const MAX_TRIANGULATION_TOLERANCE: f64 = 1.5e-3;

pub mod cube;
pub use cube::cube;

pub mod torus;
pub use torus::torus;

pub mod cylinder;
pub use cylinder::cylinder;

pub mod bottle;
pub use bottle::bottle;

pub mod organic;
pub use organic::organic;

/// Helper to compress modeling shapes into STEP-compatible data.
pub trait StepCompress {
    type Compressed;
    fn compress_for_step(&self) -> Self::Compressed;
}

impl StepCompress for Shell {
    type Compressed = CompressedShell<Point3, Curve, Surface>;
    fn compress_for_step(&self) -> Self::Compressed {
        self.compress()
    }
}

impl StepCompress for Solid {
    type Compressed = CompressedSolid<Point3, Curve, Surface>;
    fn compress_for_step(&self) -> Self::Compressed {
        self.compress()
    }
}

/// Export any B-rep (Solid or Shell) to STEP.
pub fn save_step<T, P>(brep: &T, path: P) -> io::Result<()>
where
    T: StepCompress,
    for<'a> StepModel<'a, Point3, Curve, Surface>: From<&'a T::Compressed>,
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let compressed = brep.compress_for_step();
    let display = CompleteStepDisplay::new(StepModel::from(&compressed), Default::default());
    fs::write(path, display.to_string())
}

/// Choose a chord tolerance scaled to the model size.
/// Heuristic: ~0.1% of the bounding-box diagonal, clamped to safe bounds.
fn adaptive_triangulation_tolerance(shape: &impl MeshableShape) -> f64 {
    // Use a quick coarse triangulation to estimate span without heavy meshing.
    let coarse = shape.triangulation(0.01).to_polygon();

    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];

    for p in coarse.positions().iter() {
        let coords = [p.x, p.y, p.z];
        for i in 0..3 {
            if coords[i] < min[i] {
                min[i] = coords[i];
            }
            if coords[i] > max[i] {
                max[i] = coords[i];
            }
        }
    }

    let dx = max[0] - min[0];
    let dy = max[1] - min[1];
    let dz = max[2] - min[2];
    let diag = (dx * dx + dy * dy + dz * dz).sqrt();

    let span = if diag.is_finite() && diag > 0.0 { diag } else { 1.0 };
    // Target ~0.025% of span for a touch more smoothness.
    let tol = span * 0.00025;
    tol.clamp(MIN_TRIANGULATION_TOLERANCE, MAX_TRIANGULATION_TOLERANCE)
}

/// Triangulate any B-rep (Solid or Shell) and write an OBJ mesh.
pub fn save_obj(shape: &impl MeshableShape, path: impl AsRef<Path>) -> io::Result<()> {
    let tolerance = adaptive_triangulation_tolerance(shape);
    let mesh = shape.triangulation(tolerance).to_polygon();
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut obj = fs::File::create(path)?;
    obj::write(&mesh, &mut obj).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
}

/// Convert a Wavefront OBJ into a single-mesh glTF file.
/// If `output_path` ends with `.glb`, a binary glTF is produced; otherwise a `.gltf`
/// JSON file is written alongside a `.bin` buffer of the same name.
pub fn convert_obj_to_gltf(obj_path: impl AsRef<Path>, output_path: impl AsRef<Path>) -> io::Result<()> {
    let obj_path = obj_path.as_ref();
    let output_path = output_path.as_ref();

    let (models, _) = tobj::load_obj(
        obj_path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )
    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    if models.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "OBJ contained no geometry",
        ));
    }

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let mut has_normals = true;

    for model in models {
        let mesh = model.mesh;
        if mesh.positions.len() % 3 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "OBJ positions were not 3D coordinates",
            ));
        }

        let vertex_offset = (positions.len() / 3) as u32;
        positions.extend_from_slice(&mesh.positions);

        if mesh.normals.is_empty() {
            has_normals = false;
        } else if has_normals {
            normals.extend_from_slice(&mesh.normals);
        }

        indices.extend(mesh.indices.iter().map(|idx| idx + vertex_offset));
    }

    if positions.is_empty() || indices.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "OBJ did not contain indexed triangles",
        ));
    }

    if !has_normals {
        normals.clear();
    }

    let vertex_count = positions.len() / 3;
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for chunk in positions.chunks_exact(3) {
        for i in 0..3 {
            let value = chunk[i];
            if value < min[i] {
                min[i] = value;
            }
            if value > max[i] {
                max[i] = value;
            }
        }
    }

    let mut bin = Vec::new();
    let position_offset = bin.len();
    for v in positions.iter() {
        bin.extend_from_slice(&v.to_le_bytes());
    }

    let normal_offset = bin.len();
    if !normals.is_empty() {
        for n in normals.iter() {
            bin.extend_from_slice(&n.to_le_bytes());
        }
    }

    let indices_offset = bin.len();
    for i in indices.iter() {
        bin.extend_from_slice(&i.to_le_bytes());
    }

    while bin.len() % 4 != 0 {
        bin.push(0);
    }

    let bin_length = u32::try_from(bin.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Resulting binary buffer is larger than 4 GiB",
        )
    })?;

    let position_bytes = (positions.len() * std::mem::size_of::<f32>()) as u32;
    let normal_bytes = (normals.len() * std::mem::size_of::<f32>()) as u32;
    let indices_bytes = (indices.len() * std::mem::size_of::<u32>()) as u32;

    let mut buffer = serde_json::Map::new();
    buffer.insert("byteLength".into(), json!(bin_length));

    let mut buffer_views = Vec::new();
    let mut accessors = Vec::new();
    let mut attributes = serde_json::Map::new();

    let position_view = buffer_views.len();
    buffer_views.push(json!({
        "buffer": 0,
        "byteOffset": position_offset,
        "byteLength": position_bytes,
        "target": 34962
    }));
    let position_accessor = accessors.len();
    accessors.push(json!({
        "bufferView": position_view,
        "componentType": 5126,
        "count": vertex_count,
        "type": "VEC3",
        "min": min,
        "max": max
    }));
    attributes.insert("POSITION".into(), json!(position_accessor));

    if !normals.is_empty() {
        let normal_view = buffer_views.len();
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": normal_offset,
            "byteLength": normal_bytes,
            "target": 34962
        }));
        let normal_accessor = accessors.len();
        accessors.push(json!({
            "bufferView": normal_view,
            "componentType": 5126,
            "count": vertex_count,
            "type": "VEC3"
        }));
        attributes.insert("NORMAL".into(), json!(normal_accessor));
    }

    let indices_view = buffer_views.len();
    buffer_views.push(json!({
        "buffer": 0,
        "byteOffset": indices_offset,
        "byteLength": indices_bytes,
        "target": 34963
    }));
    let indices_accessor = accessors.len();
    accessors.push(json!({
        "bufferView": indices_view,
        "componentType": 5125,
        "count": indices.len(),
        "type": "SCALAR"
    }));

    let primitive = json!({
        "attributes": attributes,
        "indices": indices_accessor,
        "mode": 4
    });

    let buffers;
    let write_glb = output_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("glb"))
        .unwrap_or(false);
    let bin_path: Option<PathBuf> = if write_glb {
        None
    } else {
        Some(output_path.with_extension("bin"))
    };

    if let Some(bin_path) = bin_path.as_ref() {
        if let Some(parent) = bin_path.parent() {
            fs::create_dir_all(parent)?;
        }
        buffer.insert(
            "uri".into(),
            json!(
                bin_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("buffer.bin")
            ),
        );
    }

    buffers = vec![serde_json::Value::Object(buffer)];

    let root = json!({
        "asset": { "version": "2.0" },
        "buffers": buffers,
        "bufferViews": buffer_views,
        "accessors": accessors,
        "meshes": [{ "primitives": [primitive] }],
        "nodes": [{ "mesh": 0 }],
        "scenes": [{ "nodes": [0] }],
        "scene": 0
    });

    if write_glb {
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut json_bytes = serde_json::to_vec(&root)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        while json_bytes.len() % 4 != 0 {
            json_bytes.push(b' ');
        }

        let json_length = u32::try_from(json_bytes.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "glTF JSON chunk is larger than 4 GiB",
            )
        })?;
        let total_length = 12u64 + 8 + json_length as u64 + 8 + bin_length as u64;
        let total_length = u32::try_from(total_length).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "GLB payload exceeds 4 GiB and cannot be written",
            )
        })?;

        let mut glb = Vec::with_capacity(total_length as usize);
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&total_length.to_le_bytes());
        glb.extend_from_slice(&json_length.to_le_bytes());
        glb.extend_from_slice(&0x4E4F534A_u32.to_le_bytes()); // "JSON"
        glb.extend_from_slice(&json_bytes);
        glb.extend_from_slice(&bin_length.to_le_bytes());
        glb.extend_from_slice(&0x004E4942_u32.to_le_bytes()); // "BIN\0"
        glb.extend_from_slice(&bin);

        fs::write(output_path, glb)
    } else {
        let bin_path = bin_path.expect("bin path is computed when not writing GLB");
        fs::write(&bin_path, &bin)?;

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let gltf = serde_json::to_vec_pretty(&root)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        fs::write(output_path, gltf)
    }
}
