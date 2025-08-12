use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::sdf_loader::*;
use std::collections::HashMap;

/// Component to mark entities loaded from SDF
#[derive(Component)]
pub struct SdfEntity {
    pub model_name: String,
    pub link_name: String,
}

/// Component for SDF models
#[derive(Component)]
pub struct SdfModelComponent {
    pub name: String,
    pub is_static: bool,
}

/// Resource to track loaded SDF worlds
#[derive(Resource, Default)]
pub struct SdfWorldRegistry {
    pub loaded_worlds: HashMap<String, SdfWorld>,
    pub asset_handles: HashMap<String, Handle<Scene>>,
}

/// Plugin for SDF world loading
pub struct SdfWorldPlugin;

impl Plugin for SdfWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SdfWorldRegistry>()
           .add_systems(Update, (
               process_sdf_load_requests,
               update_sdf_model_physics,
           ));
    }
}

/// Event to request loading an SDF world
#[derive(Event)]
pub struct LoadSdfWorldRequest {
    pub sdf_path: String,
    pub spawn_position: Vec3,
    pub spawn_rotation: Quat,
}

/// Component to mark entities that need physics setup from SDF
#[derive(Component)]
pub struct SdfPhysicsSetup {
    pub collisions: Vec<SdfCollision>,
    pub is_static: bool,
    pub mass: f32,
}

/// Load and spawn an SDF world into the Bevy scene
pub fn load_sdf_world(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    world_registry: &mut ResMut<SdfWorldRegistry>,
    sdf_path: &str,
    spawn_position: Vec3,
    spawn_rotation: Quat,
) -> Result<(), String> {
    // Load and parse SDF file
    let sdf_world = load_sdf(sdf_path)?;
    
    info!("Loading SDF world: {}", sdf_world.name);
    info!("Found {} models in world", sdf_world.models.len());
    
    // Store world in registry
    world_registry.loaded_worlds.insert(sdf_world.name.clone(), sdf_world.clone());
    
    // Spawn each model in the world
    for model in &sdf_world.models {
        spawn_sdf_model(
            commands,
            asset_server,
            meshes,
            materials,
            world_registry,
            model,
            spawn_position,
            spawn_rotation,
        )?;
    }
    
    // Apply world physics settings
    if let Some(physics) = &sdf_world.physics {
        // Note: Bevy Rapier gravity is set globally, not per-world
        // You might want to store physics settings for later use
        info!("SDF Physics - Gravity: {:?}, Max step: {}", physics.gravity, physics.max_step_size);
    }
    
    Ok(())
}

/// Spawn a single SDF model
pub fn spawn_sdf_model(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    world_registry: &mut ResMut<SdfWorldRegistry>,
    model: &SdfModel,
    world_position: Vec3,
    world_rotation: Quat,
) -> Result<(), String> {
    info!("Spawning SDF model: {}", model.name);
    
    // Calculate model transform
    let model_transform = Transform {
        translation: world_position + Vec3::new(
            model.pose.translation[0],
            model.pose.translation[1],
            model.pose.translation[2],
        ),
        rotation: world_rotation * Quat::from_euler(
            EulerRot::XYZ,
            model.pose.rotation[0],
            model.pose.rotation[1],
            model.pose.rotation[2],
        ),
        scale: Vec3::ONE,
    };
    
    // Create model entity
    let model_entity = commands.spawn((
        Transform::from_translation(Vec3::new(
            model.pose.translation[0],
            model.pose.translation[1],
            model.pose.translation[2],
        )) * Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            model.pose.rotation[0],
            model.pose.rotation[1],
            model.pose.rotation[2],
        )),
        GlobalTransform::default(),
        Visibility::default(),
        SdfModelComponent {
            name: model.name.clone(),
            is_static: model.is_static,
        },
        Name::new(format!("SDF_Model_{}", model.name)),
    )).id();
    
    // Spawn links as child entities
    for link in &model.links {
        spawn_sdf_link(
            commands,
            asset_server,
            world_registry,
            link,
            model_entity,
            &model.name,
        )?;
    }
    
    Ok(())
}

/// Spawn an SDF link
pub fn spawn_sdf_link(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    world_registry: &mut ResMut<SdfWorldRegistry>,
    link: &SdfLink,
    parent_entity: Entity,
    model_name: &str,
) -> Result<(), String> {
    // Calculate link transform
    let link_transform = Transform {
        translation: Vec3::new(
            link.pose.translation[0],
            link.pose.translation[1],
            link.pose.translation[2],
        ),
        rotation: Quat::from_euler(
            EulerRot::XYZ,
            link.pose.rotation[0],
            link.pose.rotation[1],
            link.pose.rotation[2],
        ),
        scale: Vec3::ONE,
    };
    
    // Determine mass for physics
    let mass = link.inertial.as_ref().map(|i| i.mass).unwrap_or(1.0);
    let is_static = mass <= 0.0; // Static if mass is 0 or negative
    
    // Create link entity
    let link_entity = commands.spawn((
        Transform::from_translation(Vec3::new(
            link.pose.translation[0],
            link.pose.translation[1],
            link.pose.translation[2],
        )) * Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            link.pose.rotation[0],
            link.pose.rotation[1],
            link.pose.rotation[2],
        )),
        GlobalTransform::default(),
        Visibility::default(),
        SdfEntity {
            model_name: model_name.to_string(),
            link_name: link.name.clone(),
        },
        Name::new(format!("SDF_Link_{}_{}", model_name, link.name)),
    )).id();
    
    // Add link as child of model
    commands.entity(parent_entity).add_child(link_entity);
    
    // Setup physics if there are collisions
    if !link.collisions.is_empty() {
        commands.entity(link_entity).insert(SdfPhysicsSetup {
            collisions: link.collisions.clone(),
            is_static,
            mass,
        });
    }
    
    // Spawn visual elements
    for visual in &link.visuals {
        spawn_sdf_visual(
            commands,
            asset_server,
            world_registry,
            visual,
            link_entity,
            model_name,
            &link.name,
        )?;
    }
    
    Ok(())
}

/// Spawn an SDF visual element
pub fn spawn_sdf_visual(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    world_registry: &mut ResMut<SdfWorldRegistry>,
    visual: &SdfVisual,
    parent_entity: Entity,
    model_name: &str,
    link_name: &str,
) -> Result<(), String> {
    // Calculate visual transform
    let visual_transform = Transform {
        translation: Vec3::new(
            visual.pose.translation[0],
            visual.pose.translation[1],
            visual.pose.translation[2],
        ),
        rotation: Quat::from_euler(
            EulerRot::XYZ,
            visual.pose.rotation[0],
            visual.pose.rotation[1],
            visual.pose.rotation[2],
        ),
        scale: Vec3::ONE,
    };
    
    // Create visual entity based on geometry type
    match &visual.geometry {
        SdfGeometry::Mesh { uri, scale } => {
            // Load mesh asset
            let mesh_path = resolve_mesh_uri(uri);
            if let Some(path) = mesh_path {
                let scene_handle: Handle<Scene> = asset_server.load(&path);
                world_registry.asset_handles.insert(uri.clone(), scene_handle.clone());
                
                let visual_entity = commands.spawn((
                    scene_handle,
                    visual_transform.with_scale(Vec3::new(scale[0], scale[1], scale[2])),
                    GlobalTransform::default(),
                    Visibility::default(),
                    SdfEntity {
                        model_name: model_name.to_string(),
                        link_name: link_name.to_string(),
                    },
                    Name::new(format!("SDF_Visual_{}_{}_{}", model_name, link_name, visual.name)),
                )).id();
                
                commands.entity(parent_entity).add_child(visual_entity);
            } else {
                warn!("Could not resolve mesh URI: {}", uri);
            }
        },
        
        SdfGeometry::Box { size } => {
            let mesh_handle = asset_server.add(Cuboid::new(size[0], size[1], size[2]).mesh().build());
            let material_handle = create_sdf_material(asset_server, &visual.material);
            
            let visual_entity = commands.spawn((
                mesh_handle,
                material_handle,
                visual_transform,
                GlobalTransform::default(),
                Visibility::default(),
                SdfEntity {
                    model_name: model_name.to_string(),
                    link_name: link_name.to_string(),
                },
                Name::new(format!("SDF_Visual_{}_{}_{}", model_name, link_name, visual.name)),
            )).id();
            
            commands.entity(parent_entity).add_child(visual_entity);
        },
        
        SdfGeometry::Sphere { radius } => {
            let mesh_handle = asset_server.add(Sphere::new(*radius).mesh().ico(5).unwrap().build());
            let material_handle = create_sdf_material(asset_server, &visual.material);
            
            let visual_entity = commands.spawn((
                mesh_handle,
                material_handle,
                visual_transform,
                GlobalTransform::default(),
                Visibility::default(),
                SdfEntity {
                    model_name: model_name.to_string(),
                    link_name: link_name.to_string(),
                },
                Name::new(format!("SDF_Visual_{}_{}_{}", model_name, link_name, visual.name)),
            )).id();
            
            commands.entity(parent_entity).add_child(visual_entity);
        },
        
        SdfGeometry::Cylinder { radius, length } => {
            let mesh_handle = asset_server.add(Cylinder::new(*radius, *length).mesh().build());
            let material_handle = create_sdf_material(asset_server, &visual.material);
            
            let visual_entity = commands.spawn((
                mesh_handle,
                material_handle,
                visual_transform,
                GlobalTransform::default(),
                Visibility::default(),
                SdfEntity {
                    model_name: model_name.to_string(),
                    link_name: link_name.to_string(),
                },
                Name::new(format!("SDF_Visual_{}_{}_{}", model_name, link_name, visual.name)),
            )).id();
            
            commands.entity(parent_entity).add_child(visual_entity);
        },
        
        SdfGeometry::Plane { normal: _, size } => {
            let mesh_handle = asset_server.add(Plane3d::default().mesh().size(size[0], size[1]).build());
            let material_handle = create_sdf_material(asset_server, &visual.material);
            
            let visual_entity = commands.spawn((
                mesh_handle,
                material_handle,
                visual_transform,
                GlobalTransform::default(),
                Visibility::default(),
                SdfEntity {
                    model_name: model_name.to_string(),
                    link_name: link_name.to_string(),
                },
                Name::new(format!("SDF_Visual_{}_{}_{}", model_name, link_name, visual.name)),
            )).id();
            
            commands.entity(parent_entity).add_child(visual_entity);
        },
        
        _ => {
            warn!("Unsupported SDF geometry type for visual: {:?}", visual.geometry);
        }
    }
    
    Ok(())
}

/// Create Bevy material from SDF material
fn create_sdf_material(
    asset_server: &Res<AssetServer>,
    sdf_material: &Option<SdfMaterial>,
) -> Handle<StandardMaterial> {
    if let Some(material) = sdf_material {
        asset_server.add(StandardMaterial {
            base_color: Color::srgba(
                material.diffuse[0],
                material.diffuse[1], 
                material.diffuse[2],
                material.diffuse[3],
            ),
            emissive: LinearRgba::new(
                material.emissive[0],
                material.emissive[1],
                material.emissive[2],
                material.emissive[3],
            ),
            base_color_texture: material.texture.as_ref().map(|tex| {
                asset_server.load(tex.as_str())
            }),
            ..default()
        })
    } else {
        // Default white material
        asset_server.add(StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        })
    }
}

/// Resolve mesh URI to asset path
fn resolve_mesh_uri(uri: &str) -> Option<String> {
    // Handle different URI schemes
    if uri.starts_with("file://") {
        Some(uri[7..].to_string()) // Remove 'file://' prefix
    } else if uri.starts_with("model://") {
        // Convert Gazebo model:// to relative path
        // model://model_name/meshes/file.dae -> assets/models/model_name/meshes/file.dae
        let parts: Vec<&str> = uri.split('/').collect();
        if parts.len() >= 3 {
            let model_name = parts[2];
            let path_parts = &parts[3..];
            Some(format!("models/{}/{}", model_name, path_parts.join("/")))
        } else {
            None
        }
    } else if uri.starts_with("package://") {
        // Convert ROS package:// to relative path
        let parts: Vec<&str> = uri.split('/').collect();
        if parts.len() >= 3 {
            let package_name = parts[2];
            let path_parts = &parts[3..];
            Some(format!("packages/{}/{}", package_name, path_parts.join("/")))
        } else {
            None
        }
    } else if uri.starts_with("assets/") {
        // Already an asset path
        Some(uri.to_string())
    } else {
        // Assume it's a relative path and prepend assets/
        Some(format!("assets/{}", uri))
    }
}

/// System to process SDF load requests
fn process_sdf_load_requests(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut world_registry: ResMut<SdfWorldRegistry>,
    mut load_requests: EventReader<LoadSdfWorldRequest>,
) {
    for request in load_requests.read() {
        info!("Processing SDF load request: {}", request.sdf_path);
        
        match load_sdf_world(
            &mut commands,
            &asset_server,
            &mut world_registry,
            &request.sdf_path,
            request.spawn_position,
            request.spawn_rotation,
        ) {
            Ok(()) => {
                info!("Successfully loaded SDF world: {}", request.sdf_path);
            },
            Err(e) => {
                error!("Failed to load SDF world {}: {}", request.sdf_path, e);
            }
        }
    }
}

/// System to setup physics for SDF entities
fn update_sdf_model_physics(
    mut commands: Commands,
    query: Query<(Entity, &SdfPhysicsSetup), Added<SdfPhysicsSetup>>,
) {
    for (entity, physics_setup) in query.iter() {
        // Create colliders from SDF collision geometries
        let mut colliders = Vec::new();
        
        for collision in &physics_setup.collisions {
            match &collision.geometry {
                SdfGeometry::Box { size } => {
                    colliders.push(Collider::cuboid(size[0] / 2.0, size[1] / 2.0, size[2] / 2.0));
                },
                SdfGeometry::Sphere { radius } => {
                    colliders.push(Collider::ball(*radius));
                },
                SdfGeometry::Cylinder { radius, length } => {
                    colliders.push(Collider::cylinder(*length / 2.0, *radius));
                },
                SdfGeometry::Mesh { uri, scale: _ } => {
                    // For mesh colliders, you might want to load the mesh and create a trimesh collider
                    // This is more complex and depends on your mesh loading setup
                    warn!("Mesh colliders not yet implemented for SDF: {}", uri);
                },
                _ => {
                    warn!("Unsupported collision geometry: {:?}", collision.geometry);
                }
            }
        }
        
        // Add the first collider (compound colliders would require more work)
        if let Some(collider) = colliders.into_iter().next() {
            if physics_setup.is_static {
                commands.entity(entity).insert((
                    RigidBody::Fixed,
                    collider,
                ));
            } else {
                commands.entity(entity).insert((
                    RigidBody::Dynamic,
                    collider,
                    AdditionalMassProperties::Mass(physics_setup.mass),
                ));
            }
        }
        
        // Remove the setup component as it's no longer needed
        commands.entity(entity).remove::<SdfPhysicsSetup>();
    }
}

/// Helper function to spawn SDF world in startup systems
pub fn spawn_sdf_world_at_startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world_registry: ResMut<SdfWorldRegistry>,
    sdf_path: &str,
    position: Vec3,
) {
    match load_sdf_world(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        &mut world_registry,
        sdf_path,
        position,
        Quat::IDENTITY,
    ) {
        Ok(()) => {
            info!("SDF world loaded successfully: {}", sdf_path);
        },
        Err(e) => {
            error!("Failed to load SDF world: {}", e);
        }
    }
}
