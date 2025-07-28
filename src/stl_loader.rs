use bevy::prelude::*;

/// Loads an STL file and converts it to a Bevy mesh
pub fn load_stl_mesh(path: &str) -> Result<Mesh, String> {
    let full_path = format!("assets/robots/urdf/{}", path);
    
    // Load the STL file
    let stl_data = std::fs::read(&full_path)
        .map_err(|e| format!("Failed to read STL file {}: {}", full_path, e))?;
    
    // Parse the STL file
    let stl_file = stl::parse(&stl_data)
        .map_err(|e| format!("Failed to parse STL file {}: {}", full_path, e))?;
    
    // Convert to Bevy mesh
    let mesh = stl_to_bevy_mesh(&stl_file);
    
    Ok(mesh)
}

/// Converts an STL file to a Bevy mesh
fn stl_to_bevy_mesh(stl_file: &stl::Stl) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    
    for triangle in &stl_file.triangles {
        add_triangle_to_mesh(triangle, &mut positions, &mut normals, &mut indices);
    }
    
    // Create the mesh
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::mesh::RenderAssetUsages::RENDER_WORLD
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    
    mesh
}

/// Adds a triangle from the STL file to the mesh data
fn add_triangle_to_mesh(
    triangle: &stl::Triangle,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
) {
    let start_index = positions.len() as u32;
    
    // Add the three vertices
    positions.push([triangle.v1[0], triangle.v1[1], triangle.v1[2]]);
    positions.push([triangle.v2[0], triangle.v2[1], triangle.v2[2]]);
    positions.push([triangle.v3[0], triangle.v3[1], triangle.v3[2]]);
    
    // Add the normal (same for all three vertices of the triangle)
    let normal = [triangle.normal[0], triangle.normal[1], triangle.normal[2]];
    normals.push(normal);
    normals.push(normal);
    normals.push(normal);
    
    // Add the triangle indices
    indices.push(start_index);
    indices.push(start_index + 1);
    indices.push(start_index + 2);
}

/// Asset loader for STL files
#[derive(Default)]
pub struct StlAssetLoader;

impl bevy::asset::AssetLoader for StlAssetLoader {
    type Asset = Mesh;
    type Settings = ();
    type Error = String;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a (),
        _load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            
            let stl_file = stl::parse(&bytes)
                .map_err(|e| format!("Failed to parse STL file: {}", e))?;
            
            Ok(stl_to_bevy_mesh(&stl_file))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["stl"]
    }
}