use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;

#[derive(Debug)]
pub enum StlError {
    Io(std::io::Error),
    Parse(String),
}

impl From<std::io::Error> for StlError {
    fn from(err: std::io::Error) -> Self {
        StlError::Io(err)
    }
}

impl std::fmt::Display for StlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StlError::Io(e) => write!(f, "IO error: {}", e),
            StlError::Parse(s) => write!(f, "STL parsing error: {}", s),
        }
    }
}

impl std::error::Error for StlError {}

#[derive(Asset, TypePath, Debug)]
pub struct StlMesh {
    pub mesh: Mesh,
}

pub struct StlLoader;

impl AssetLoader for StlLoader {
    type Asset = StlMesh;
    type Settings = ();
    type Error = StlError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        
        // Parse STL file
        let mesh = parse_stl_bytes(&bytes)?;
        
        Ok(StlMesh { mesh })
    }

    fn extensions(&self) -> &[&str] {
        &["stl", "STL"]
    }
}

fn parse_stl_bytes(bytes: &[u8]) -> Result<Mesh, StlError> {
    use std::io::Cursor;
    
    let mut cursor = Cursor::new(bytes);
    let stl = stl_io::read_stl(&mut cursor)
        .map_err(|e| StlError::Parse(format!("Failed to parse STL: {}", e)))?;
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    
    // Extract vertices and compute normals
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    
    for (i, triangle) in stl.faces.iter().enumerate() {
        let base_index = (i * 3) as u32;
        
        // Add vertices
        for j in 0..3 {
            let vertex = triangle.vertices[j];
            positions.push([vertex[0], vertex[1], vertex[2]]);
            normals.push([triangle.normal[0], triangle.normal[1], triangle.normal[2]]);
        }
        
        // Add indices
        indices.push(base_index);
        indices.push(base_index + 1);
        indices.push(base_index + 2);
    }
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    
    Ok(mesh)
}

/// Load an STL file directly from a path
pub fn load_stl_file(path: &Path) -> Result<Mesh, StlError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let stl = stl_io::read_stl(&mut reader)
        .map_err(|e| StlError::Parse(format!("Failed to parse STL: {}", e)))?;
    
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    
    // Extract vertices and compute normals
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    
    for (i, triangle) in stl.faces.iter().enumerate() {
        let base_index = (i * 3) as u32;
        
        // Add vertices
        for j in 0..3 {
            let vertex = triangle.vertices[j];
            positions.push([vertex[0], vertex[1], vertex[2]]);
            normals.push([triangle.normal[0], triangle.normal[1], triangle.normal[2]]);
        }
        
        // Add indices
        indices.push(base_index);
        indices.push(base_index + 1);
        indices.push(base_index + 2);
    }
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    
    Ok(mesh)
}