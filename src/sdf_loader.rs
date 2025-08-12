use bevy::prelude::*;
use bevy_rapier3d::geometry::{Collider, CollisionGroups, Group};
use std::fs;
use quick_xml::Reader;
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};
use std::io::BufReader;

/// SDF World structure representing a complete simulation world
#[derive(Debug, Clone)]
pub struct SdfWorld {
    pub name: String,
    pub models: Vec<SdfModel>,
    pub lights: Vec<SdfLight>,
    pub physics: Option<SdfPhysics>,
    pub scene: Option<SdfScene>,
}

/// SDF Model structure representing a model in the world
#[derive(Debug, Clone)]
pub struct SdfModel {
    pub name: String,
    pub static_: bool,
    pub pose: SdfPose,
    pub links: Vec<SdfLink>,
    pub joints: Vec<SdfJoint>,
}

/// SDF Link structure representing a link in a model
#[derive(Debug, Clone)]
pub struct SdfLink {
    pub name: String,
    pub pose: SdfPose,
    pub visual: Option<SdfVisual>,
    pub collision: Option<SdfCollision>,
    pub inertial: Option<SdfInertial>,
}

/// SDF Visual structure for visual representation
#[derive(Debug, Clone)]
pub struct SdfVisual {
    pub name: String,
    pub pose: SdfPose,
    pub geometry: SdfGeometry,
    pub material: Option<SdfMaterial>,
}

/// SDF Collision structure for collision detection
#[derive(Debug, Clone)]
pub struct SdfCollision {
    pub name: String,
    pub pose: SdfPose,
    pub geometry: SdfGeometry,
}

/// SDF Geometry types
#[derive(Debug, Clone)]
pub enum SdfGeometry {
    Box { size: Vec3 },
    Sphere { radius: f32 },
    Cylinder { radius: f32, length: f32 },
    Plane { normal: Vec3, size: Vec2 },
    Mesh { uri: String, scale: Option<Vec3> },
}

/// SDF Material structure
#[derive(Debug, Clone)]
pub struct SdfMaterial {
    pub ambient: Option<Color>,
    pub diffuse: Option<Color>,
    pub specular: Option<Color>,
    pub emissive: Option<Color>,
}

/// SDF Pose structure
#[derive(Debug, Clone, Default)]
pub struct SdfPose {
    pub xyz: Vec3,
    pub rpy: Vec3, // roll, pitch, yaw in radians
}

/// SDF Joint structure
#[derive(Debug, Clone)]
pub struct SdfJoint {
    pub name: String,
    pub joint_type: String,
    pub parent: String,
    pub child: String,
    pub pose: SdfPose,
}

/// SDF Light structure
#[derive(Debug, Clone)]
pub struct SdfLight {
    pub name: String,
    pub light_type: String,
    pub pose: SdfPose,
    pub diffuse: Color,
    pub specular: Color,
}

/// SDF Physics structure
#[derive(Debug, Clone)]
pub struct SdfPhysics {
    pub name: String,
    pub max_step_size: f32,
    pub real_time_factor: f32,
    pub real_time_update_rate: f32,
    pub gravity: Vec3,
}

/// SDF Scene structure
#[derive(Debug, Clone)]
pub struct SdfScene {
    pub ambient: Color,
    pub background: Color,
}

/// SDF Inertial structure
#[derive(Debug, Clone)]
pub struct SdfInertial {
    pub mass: f32,
    pub inertia: Vec3,
    pub pose: SdfPose,
}

/// XML parsing context
#[derive(Debug)]
struct XmlContext {
    current_element: String,
    current_model: Option<SdfModel>,
    current_link: Option<SdfLink>,
    current_visual: Option<SdfVisual>,
    current_collision: Option<SdfCollision>,
    current_inertial: Option<SdfInertial>,
    current_geometry: Option<SdfGeometry>,
    current_material: Option<SdfMaterial>,
    current_pose: Option<SdfPose>,
    current_text: String,
}

impl XmlContext {
    fn new() -> Self {
        Self {
            current_element: String::new(),
            current_model: None,
            current_link: None,
            current_visual: None,
            current_collision: None,
            current_inertial: None,
            current_geometry: None,
            current_material: None,
            current_pose: None,
            current_text: String::new(),
        }
    }
}

/// Loads an SDF file and returns the world structure
pub fn load_sdf(path: &str) -> Result<SdfWorld, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read SDF file: {}", e))?;
    
    parse_sdf_content(&content)
}

/// Parses SDF XML content
fn parse_sdf_content(content: &str) -> Result<SdfWorld, String> {
    let mut reader = Reader::from_str(content);
    reader.trim_text(true);
    
    let mut context = XmlContext::new();
    let mut world = SdfWorld {
        name: String::new(),
        models: Vec::new(),
        lights: Vec::new(),
        physics: None,
        scene: None,
    };
    
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                let name_bytes = name.as_ref();
                let tag_name = std::str::from_utf8(name_bytes)
                    .map_err(|e| format!("Invalid UTF-8: {}", e))?;
                
                context.current_element = tag_name.to_string();
                context.current_text.clear();
                
                match tag_name {
                    "world" => {
                        if let Some(name) = get_attribute(e, "name") {
                            world.name = name;
                        }
                    }
                    "model" => {
                        context.current_model = Some(SdfModel {
                            name: get_attribute(e, "name").unwrap_or_default(),
                            static_: false,
                            pose: SdfPose::default(),
                            links: Vec::new(),
                            joints: Vec::new(),
                        });
                    }
                    "link" => {
                        context.current_link = Some(SdfLink {
                            name: get_attribute(e, "name").unwrap_or_default(),
                            pose: SdfPose::default(),
                            visual: None,
                            collision: None,
                            inertial: None,
                        });
                    }
                    "visual" => {
                        context.current_visual = Some(SdfVisual {
                            name: get_attribute(e, "name").unwrap_or_default(),
                            pose: SdfPose::default(),
                            geometry: SdfGeometry::Box { size: Vec3::ONE }, // Default
                            material: None,
                        });
                    }
                    "collision" => {
                        context.current_collision = Some(SdfCollision {
                            name: get_attribute(e, "name").unwrap_or_default(),
                            pose: SdfPose::default(),
                            geometry: SdfGeometry::Box { size: Vec3::ONE }, // Default
                        });
                    }
                    "inertial" => {
                        context.current_inertial = Some(SdfInertial {
                            mass: 0.0,
                            inertia: Vec3::ZERO,
                            pose: SdfPose::default(),
                        });
                    }
                    "geometry" => {
                        context.current_geometry = None;
                    }
                    "box" => {
                        // Box geometry will be set when size is parsed
                    }
                    "sphere" => {
                        // Sphere geometry will be set when radius is parsed
                    }
                    "cylinder" => {
                        // Cylinder geometry will be set when radius and length are parsed
                    }
                    "plane" => {
                        // Initialize plane geometry - normal and size will be parsed separately
                        context.current_geometry = Some(SdfGeometry::Plane { 
                            normal: Vec3::new(0.0, 0.0, 1.0), 
                            size: Vec2::new(1.0, 1.0) 
                        });
                    }
                    "material" => {
                        context.current_material = Some(SdfMaterial {
                            ambient: None,
                            diffuse: None,
                            specular: None,
                            emissive: None,
                        });
                    }
                    "pose" => {
                        context.current_pose = Some(SdfPose::default());
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape()
                    .map_err(|e| format!("Failed to unescape text: {}", e))?;
                context.current_text = text.to_string();
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                let name_bytes = name.as_ref();
                let tag_name = std::str::from_utf8(name_bytes)
                    .map_err(|e| format!("Invalid UTF-8: {}", e))?;
                
                match tag_name {
                    "static" => {
                        if let Some(model) = &mut context.current_model {
                            model.static_ = context.current_text.trim() == "true";
                        }
                    }
                    "pose" => {
                        if let Some(pose) = parse_pose(&context.current_text) {
                            if let Some(model) = &mut context.current_model {
                                model.pose = pose;
                            } else if let Some(link) = &mut context.current_link {
                                link.pose = pose;
                            } else if let Some(visual) = &mut context.current_visual {
                                visual.pose = pose;
                            } else if let Some(collision) = &mut context.current_collision {
                                collision.pose = pose;
                            }
                        }
                    }
                    "mass" => {
                        if let Some(inertial) = &mut context.current_inertial {
                            if let Ok(mass) = context.current_text.trim().parse::<f32>() {
                                inertial.mass = mass;
                            }
                        }
                    }
                    "gravity" => {
                        if let Some(gravity) = parse_vec3(&context.current_text) {
                            world.physics = Some(SdfPhysics {
                                name: "default_physics".to_string(),
                                max_step_size: 0.001,
                                real_time_factor: 1.0,
                                real_time_update_rate: 1000.0,
                                gravity,
                            });
                        }
                    }
                    "ambient" => {
                        if let Some(color) = parse_color(&context.current_text) {
                            if let Some(material) = &mut context.current_material {
                                material.ambient = Some(color);
                            } else {
                                world.scene = Some(SdfScene {
                                    ambient: color,
                                    background: Color::srgba(0.7, 0.7, 0.7, 1.0),
                                });
                            }
                        }
                    }
                    "background" => {
                        if let Some(color) = parse_color(&context.current_text) {
                            if let Some(scene) = &mut world.scene {
                                scene.background = color;
                            }
                        }
                    }
                    "diffuse" => {
                        if let Some(color) = parse_color(&context.current_text) {
                            if let Some(material) = &mut context.current_material {
                                material.diffuse = Some(color);
                            }
                        }
                    }
                    "size" => {
                        // Check if this is for a plane (needs Vec2) or box (needs Vec3)
                        if let Some(SdfGeometry::Plane { normal, size: _ }) = &context.current_geometry {
                            if let Some(size) = parse_vec2(&context.current_text) {
                                context.current_geometry = Some(SdfGeometry::Plane { normal: *normal, size });
                            }
                        } else if let Some(size) = parse_vec3(&context.current_text) {
                            context.current_geometry = Some(SdfGeometry::Box { size });
                        }
                    }
                    "radius" => {
                        if let Ok(radius) = context.current_text.trim().parse::<f32>() {
                            context.current_geometry = Some(SdfGeometry::Sphere { radius });
                        }
                    }
                    "length" => {
                        if let Ok(length) = context.current_text.trim().parse::<f32>() {
                            // For cylinders, we need both radius and length
                            if let Some(SdfGeometry::Cylinder { radius, .. }) = context.current_geometry {
                                context.current_geometry = Some(SdfGeometry::Cylinder { radius, length });
                            }
                        }
                    }
                    "normal" => {
                        if let Some(normal) = parse_vec3(&context.current_text) {
                            // Store normal for plane geometry - size will be parsed separately
                            if let Some(SdfGeometry::Plane { normal: _, size }) = &context.current_geometry {
                                context.current_geometry = Some(SdfGeometry::Plane { normal, size: *size });
                            } else {
                                // Create plane with default size, will be updated when size is parsed
                                context.current_geometry = Some(SdfGeometry::Plane { normal, size: Vec2::new(1.0, 1.0) });
                            }
                        }
                    }
                    "geometry" => {
                        if let Some(geometry) = context.current_geometry.take() {
                            if let Some(visual) = &mut context.current_visual {
                                visual.geometry = geometry;
                            } else if let Some(collision) = &mut context.current_collision {
                                collision.geometry = geometry;
                            }
                        }
                    }
                    "material" => {
                        if let Some(material) = context.current_material.take() {
                            if let Some(visual) = &mut context.current_visual {
                                visual.material = Some(material);
                            }
                        }
                    }
                    "visual" => {
                        if let Some(visual) = context.current_visual.take() {
                            if let Some(link) = &mut context.current_link {
                                link.visual = Some(visual);
                            }
                        }
                    }
                    "collision" => {
                        if let Some(collision) = context.current_collision.take() {
                            if let Some(link) = &mut context.current_link {
                                link.collision = Some(collision);
                            }
                        }
                    }
                    "inertial" => {
                        if let Some(inertial) = context.current_inertial.take() {
                            if let Some(link) = &mut context.current_link {
                                link.inertial = Some(inertial);
                            }
                        }
                    }
                    "link" => {
                        if let Some(link) = context.current_link.take() {
                            if let Some(model) = &mut context.current_model {
                                model.links.push(link);
                            }
                        }
                    }
                    "model" => {
                        if let Some(model) = context.current_model.take() {
                            world.models.push(model);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error parsing XML: {}", e)),
            _ => {}
        }
        
        buf.clear();
    }
    
    Ok(world)
}

/// Helper function to get attribute value
fn get_attribute(e: &BytesStart, name: &str) -> Option<String> {
    e.attributes()
        .find(|attr| attr.as_ref().map(|a| a.key.as_ref() == name.as_bytes()).unwrap_or(false))
        .and_then(|attr| attr.ok())
        .and_then(|attr| String::from_utf8(attr.value.to_vec()).ok())
}

/// Parse pose string (x y z roll pitch yaw)
fn parse_pose(text: &str) -> Option<SdfPose> {
    let parts: Vec<f32> = text.split_whitespace()
        .filter_map(|s| s.parse::<f32>().ok())
        .collect();
    
    if parts.len() >= 6 {
        Some(SdfPose {
            xyz: Vec3::new(parts[0], parts[1], parts[2]),
            rpy: Vec3::new(parts[3], parts[4], parts[5]),
        })
    } else {
        None
    }
}

/// Parse Vec3 string (x y z)
fn parse_vec3(text: &str) -> Option<Vec3> {
    let parts: Vec<f32> = text.split_whitespace()
        .filter_map(|s| s.parse::<f32>().ok())
        .collect();
    
    if parts.len() >= 3 {
        Some(Vec3::new(parts[0], parts[1], parts[2]))
    } else {
        None
    }
}

/// Parse Vec2 string (x y)
fn parse_vec2(text: &str) -> Option<Vec2> {
    let parts: Vec<f32> = text.split_whitespace()
        .filter_map(|s| s.parse::<f32>().ok())
        .collect();
    
    if parts.len() >= 2 {
        Some(Vec2::new(parts[0], parts[1]))
    } else {
        None
    }
}

/// Parse color string (r g b a)
fn parse_color(text: &str) -> Option<Color> {
    let parts: Vec<f32> = text.split_whitespace()
        .filter_map(|s| s.parse::<f32>().ok())
        .collect();
    
    if parts.len() >= 4 {
        Some(Color::srgba(parts[0], parts[1], parts[2], parts[3]))
    } else if parts.len() >= 3 {
        Some(Color::srgb(parts[0], parts[1], parts[2]))
    } else {
        None
    }
}

/// Spawns a complete Bevy world from a parsed SDF world
pub fn spawn_sdf_world(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    _asset_server: &Res<AssetServer>,
    world: &SdfWorld,
) {
    // Apply scene settings
    if let Some(scene) = &world.scene {
        commands.insert_resource(ClearColor(scene.background));
        commands.insert_resource(AmbientLight {
            color: scene.ambient,
            brightness: 500.0,
            affects_lightmapped_meshes: true,
        });
    }
    
    // Apply physics settings
    if let Some(physics) = &world.physics {
        println!("Physics settings: gravity={:?}, max_step_size={}", physics.gravity, physics.max_step_size);
    }
    
    // Spawn all models
    for model in &world.models {
        spawn_sdf_model(commands, meshes, materials, model);
    }
    
    // Spawn all lights
    for light in &world.lights {
        spawn_sdf_light(commands, light);
    }
}

/// Spawns a single SDF model as Bevy entities
fn spawn_sdf_model(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    model: &SdfModel,
) {
    println!("Spawning SDF model: {}", model.name);
    
    // Create model transform from pose
    let model_transform = sdf_pose_to_transform(&model.pose);
    
    // Spawn each link in the model
    for link in &model.links {
        spawn_sdf_link(commands, meshes, materials, model, link, model_transform);
    }
}

/// Spawns a single SDF link as a Bevy entity
fn spawn_sdf_link(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    model: &SdfModel,
    link: &SdfLink,
    model_transform: Transform,
) {
    let link_transform = model_transform.mul_transform(sdf_pose_to_transform(&link.pose));
    
    // Create visual mesh and material
    if let Some(visual) = &link.visual {
        let (mesh_handle, material_handle) = create_visual_geometry(
            meshes, materials, &visual.geometry, &visual.material
        );
        
        // Create the entity
        let mut entity_cmd = commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            link_transform,
            Name::new(format!("{}_{}", model.name, link.name)),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));
        
        // Add collision if specified
        if let Some(collision) = &link.collision {
            let collider = create_collider(&collision.geometry);
            entity_cmd.insert(collider);
            
            // Add collision groups based on whether the model is static
            if model.static_ {
                entity_cmd.insert(CollisionGroups::new(
                    STATIC_GROUP,
                    CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP,
                ));
            } else {
                entity_cmd.insert(CollisionGroups::new(
                    CHASSIS_GROUP,
                    STATIC_GROUP | CHASSIS_INTERNAL_GROUP,
                ));
            }
        }
    }
}

/// Creates visual geometry from SDF geometry
fn create_visual_geometry(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    geometry: &SdfGeometry,
    material: &Option<SdfMaterial>,
) -> (Handle<Mesh>, Handle<StandardMaterial>) {
    let mesh_handle = match geometry {
        SdfGeometry::Box { size } => {
            meshes.add(Mesh::from(Cuboid::new(size.x, size.y, size.z)))
        }
        SdfGeometry::Sphere { radius } => {
            meshes.add(Mesh::from(Sphere { radius: *radius, ..Default::default() }))
        }
        SdfGeometry::Cylinder { radius, length } => {
            meshes.add(Mesh::from(Cylinder { radius: *radius, half_height: *length / 2.0, ..Default::default() }))
        }
        SdfGeometry::Plane { normal: _, size } => {
            meshes.add(Plane3d::default().mesh().size(size.x, size.y))
        }
        SdfGeometry::Mesh { uri: _, scale: _ } => {
            // For now, use a simple box as fallback
            println!("Warning: Mesh loading not fully implemented");
            meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)))
        }
    };
    
    let material_handle = if let Some(sdf_material) = material {
        let color = sdf_material.diffuse.unwrap_or(Color::srgb(0.7, 0.7, 0.7));
        materials.add(StandardMaterial {
            base_color: color,
            ..Default::default()
        })
    } else {
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.7, 0.7),
            ..Default::default()
        })
    };
    
    (mesh_handle, material_handle)
}

/// Creates a collider from SDF geometry
fn create_collider(geometry: &SdfGeometry) -> Collider {
    match geometry {
        SdfGeometry::Box { size } => {
            Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0)
        }
        SdfGeometry::Sphere { radius } => {
            Collider::ball(*radius)
        }
        SdfGeometry::Cylinder { radius, length } => {
            Collider::cylinder(*length / 2.0, *radius)
        }
        SdfGeometry::Plane { normal: _, size } => {
            // Create a thin box for the plane
            Collider::cuboid(size.x / 2.0, 0.01, size.y / 2.0)
        }
        SdfGeometry::Mesh { uri: _, scale: _ } => {
            // Fallback to a box collider for meshes
            Collider::cuboid(0.5, 0.5, 0.5)
        }
    }
}

/// Spawns a single SDF light as a Bevy light
fn spawn_sdf_light(commands: &mut Commands, light: &SdfLight) {
    let light_transform = sdf_pose_to_transform(&light.pose);
    
    match light.light_type.as_str() {
        "point" => {
            commands.spawn((
                PointLight::default(),
                light_transform,
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        }
        "directional" => {
            commands.spawn((
                DirectionalLight {
                    shadows_enabled: false,
                    illuminance: 1000.0,
                    ..default()
                },
                light_transform,
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        }
        "spot" => {
            // Bevy doesn't have a built-in spot light, so we'll use a point light
            commands.spawn((
                PointLight::default(),
                light_transform,
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        }
        _ => {
            println!("Warning: Unknown light type: {}", light.light_type);
        }
    }
}

/// Converts SDF pose to Bevy transform
fn sdf_pose_to_transform(pose: &SdfPose) -> Transform {
    let translation = pose.xyz;
    let rotation = Quat::from_euler(EulerRot::XYZ, pose.rpy.x, pose.rpy.y, pose.rpy.z);
    Transform::from_translation(translation).with_rotation(rotation)
}

// Re-export collision groups for consistency
pub const STATIC_GROUP: Group = Group::GROUP_1;
pub const CHASSIS_INTERNAL_GROUP: Group = Group::GROUP_2;
pub const CHASSIS_GROUP: Group = Group::GROUP_3;
