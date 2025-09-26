use bevy::prelude::*;
use crate::sdf_loader::*;

/// Component to mark entities loaded from SDF
#[derive(Component)]
pub struct SdfEntity {
    pub model_name: String,
    pub link_name: String,
}

/// Resource to track loaded SDF worlds
#[derive(Resource, Default)]
pub struct SdfWorldRegistry {
    pub loaded_worlds: Vec<String>,
}

/// Plugin for SDF world loading
pub struct SdfWorldPlugin;

impl Plugin for SdfWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SdfWorldRegistry>();
    }
}

/// Simple function to load and display SDF information
pub fn load_sdf_world_simple(
    sdf_path: &str,
    _position: Vec3,
    world_registry: &mut ResMut<SdfWorldRegistry>,
) -> Result<(), String> {
    // Load and parse SDF file
    let sdf_world = load_sdf(sdf_path)?;
    
    info!("Loading SDF world: {}", sdf_world.name);
    info!("Found {} models in world", sdf_world.models.len());
    
    // Store world in registry
    world_registry.loaded_worlds.push(sdf_world.name.clone());
    
    // Print information about each model
    for model in &sdf_world.models {
        info!("Model: {} (static: {})", model.name, model.static_);
        info!("  Links: {}", model.links.len());
        info!("  Joints: {}", model.joints.len());
        
        for link in &model.links {
            info!("    Link: {}", link.name);
            info!("      Visuals: {}", if link.visual.is_some() { 1 } else { 0 });
            info!("      Collisions: {}", if link.collision.is_some() { 1 } else { 0 });

            if let Some(visual) = &link.visual {
                match &visual.geometry {
                    SdfGeometry::Box { size } => {
                        info!("        Box visual: {}x{}x{}", size[0], size[1], size[2]);
                    },
                    SdfGeometry::Sphere { radius } => {
                        info!("        Sphere visual: radius {}", radius);
                    },
                    SdfGeometry::Cylinder { radius, length } => {
                        info!("        Cylinder visual: radius {}, length {}", radius, length);
                    },
                    SdfGeometry::Mesh { uri, scale } => {
                        if let Some(scale_vec) = scale {
                            info!("        Mesh visual: {} (scale: {}x{}x{})", uri, scale_vec[0], scale_vec[1], scale_vec[2]);
                        } else {
                            info!("        Mesh visual: {} (no scale)", uri);
                        }
                    },
                    SdfGeometry::Plane { normal, size } => {
                        info!("        Plane visual: normal [{}, {}, {}], size {}x{}", 
                              normal[0], normal[1], normal[2], size[0], size[1]);
                    },
                    _ => {
                        info!("        Other geometry type");
                    }
                }
            }
        }
    }
    
    // Apply world physics settings
    if let Some(physics) = &sdf_world.physics {
        info!("SDF Physics - Gravity: {:?}, Max step: {}", physics.gravity, physics.max_step_size);
    }
    
    Ok(())
}

/// Helper function to demonstrate SDF loading in startup systems
pub fn demo_sdf_loading(
    mut world_registry: ResMut<SdfWorldRegistry>,
    sdf_path: &str,
    position: Vec3,
) {
    match load_sdf_world_simple(sdf_path, position, &mut world_registry) {
        Ok(()) => {
            info!("SDF world information loaded successfully: {}", sdf_path);
        },
        Err(e) => {
            error!("Failed to load SDF world: {}", e);
        }
    }
}
