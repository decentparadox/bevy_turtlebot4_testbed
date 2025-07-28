use bevy::prelude::*;
use bevy_rapier3d::{
    geometry::{Collider, CollisionGroups},
};

use crate::{STATIC_GROUP, CHASSIS_INTERNAL_GROUP, CHASSIS_GROUP};

/// Component to mark world objects
#[derive(Component)]
pub struct WorldObject;

/// Component to mark obstacles
#[derive(Component)]
pub struct Obstacle;

/// Component to mark walls
#[derive(Component)]
pub struct Wall;

/// Spawns a simple arena with walls and some obstacles
pub fn spawn_simple_arena(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Arena dimensions
    let arena_size = 8.0;
    let wall_height = 0.5;
    let wall_thickness = 0.1;
    
    // Wall material
    let wall_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8),
        ..Default::default()
    });
    
    // Floor
    commands
        .spawn((
            Collider::cuboid(arena_size * 0.5, 0.1, arena_size * 0.5),
            CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP),
            Transform::from_xyz(0.0, -0.1, 0.0),
            WorldObject,
        ))
        .with_children(|commands| {
            // Visual floor (larger than physics floor for better appearance)
            commands.spawn((
                Mesh3d(meshes.add(Plane3d::default().mesh().size(arena_size + 1.0, arena_size + 1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.9, 0.9, 0.9),
                    ..Default::default()
                })),
                Transform::from_xyz(0.0, 0.05, 0.0),
                WorldObject,
            ));
        });
    
    // Spawn walls
    spawn_wall(commands, meshes, materials, &wall_material, 
               Vec3::new(0.0, wall_height * 0.5, -arena_size * 0.5), 
               Vec3::new(arena_size, wall_height, wall_thickness), 
               "North Wall");
    
    spawn_wall(commands, meshes, materials, &wall_material, 
               Vec3::new(0.0, wall_height * 0.5, arena_size * 0.5), 
               Vec3::new(arena_size, wall_height, wall_thickness), 
               "South Wall");
    
    spawn_wall(commands, meshes, materials, &wall_material, 
               Vec3::new(-arena_size * 0.5, wall_height * 0.5, 0.0), 
               Vec3::new(wall_thickness, wall_height, arena_size), 
               "West Wall");
    
    spawn_wall(commands, meshes, materials, &wall_material, 
               Vec3::new(arena_size * 0.5, wall_height * 0.5, 0.0), 
               Vec3::new(wall_thickness, wall_height, arena_size), 
               "East Wall");
    
    // Spawn some obstacles
    spawn_obstacle(commands, meshes, materials, 
                   Vec3::new(2.0, 0.25, 2.0), 
                   Vec3::new(0.5, 0.5, 0.5), 
                   Color::srgb(0.7, 0.3, 0.3),
                   "Red Obstacle");
    
    spawn_obstacle(commands, meshes, materials, 
                   Vec3::new(-1.5, 0.3, -1.0), 
                   Vec3::new(0.6, 0.6, 0.6), 
                   Color::srgb(0.3, 0.7, 0.3),
                   "Green Obstacle");
    
    spawn_obstacle(commands, meshes, materials, 
                   Vec3::new(0.0, 0.2, 3.0), 
                   Vec3::new(1.0, 0.4, 0.3), 
                   Color::srgb(0.3, 0.3, 0.7),
                   "Blue Obstacle");
    
    // Spawn a cylindrical obstacle to demonstrate different shapes
    spawn_cylinder_obstacle(commands, meshes, materials, 
                           Vec3::new(-2.0, 0.25, 1.5), 
                           0.3, 0.5, 
                           Color::srgb(0.8, 0.8, 0.2),
                           "Yellow Cylinder");
}

/// Spawns a wall with both visual and physics meshes
fn spawn_wall(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
    material: &Handle<StandardMaterial>,
    position: Vec3,
    size: Vec3,
    name: &str,
) {
    commands
        .spawn((
            Collider::cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5),
            CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP),
            Transform::from_translation(position),
            Name::new(name.to_string()),
            Wall,
            WorldObject,
        ))
        .with_children(|commands| {
            // Visual mesh (same as physics for walls)
            commands.spawn((
                Mesh3d(meshes.add(Mesh::from(Cuboid::new(size.x, size.y, size.z)))),
                MeshMaterial3d(material.clone()),
                Transform::default(),
                WorldObject,
            ));
        });
}

/// Spawns a box obstacle with both visual and physics meshes
fn spawn_obstacle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    size: Vec3,
    color: Color,
    name: &str,
) {
    commands
        .spawn((
            Collider::cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5),
            CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP),
            Transform::from_translation(position),
            Name::new(name.to_string()),
            Obstacle,
            WorldObject,
        ))
        .with_children(|commands| {
            // Visual mesh (same as physics for simple obstacles)
            commands.spawn((
                Mesh3d(meshes.add(Mesh::from(Cuboid::new(size.x, size.y, size.z)))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    ..Default::default()
                })),
                Transform::default(),
                WorldObject,
            ));
        });
}

/// Spawns a cylindrical obstacle with both visual and physics meshes
fn spawn_cylinder_obstacle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    radius: f32,
    height: f32,
    color: Color,
    name: &str,
) {
    commands
        .spawn((
            Collider::cylinder(height * 0.5, radius),
            CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP),
            Transform::from_translation(position),
            Name::new(name.to_string()),
            Obstacle,
            WorldObject,
        ))
        .with_children(|commands| {
            // Visual mesh (same as physics for simple cylinders)
            commands.spawn((
                Mesh3d(meshes.add(Mesh::from(Cylinder { 
                    radius, 
                    half_height: height * 0.5, 
                    ..Default::default() 
                }))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    ..Default::default()
                })),
                Transform::default(),
                WorldObject,
            ));
        });
}

/// Spawns a complex obstacle with different visual and physics meshes
/// This demonstrates the concept of having detailed visual meshes with simplified physics
pub fn spawn_complex_obstacle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    name: &str,
) {
    // Physics: Simple box collider for performance
    let physics_size = Vec3::new(0.8, 0.6, 0.8);
    
    commands
        .spawn((
            Collider::cuboid(physics_size.x * 0.5, physics_size.y * 0.5, physics_size.z * 0.5),
            CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP),
            Transform::from_translation(position),
            Name::new(name.to_string()),
            Obstacle,
            WorldObject,
        ))
        .with_children(|commands| {
            // Visual: More complex shape made of multiple parts
            let base_material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.6, 0.4, 0.2),
                ..Default::default()
            });
            
            let detail_material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.6, 0.4),
                ..Default::default()
            });
            
            // Base
            commands.spawn((
                Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.8, 0.4, 0.8)))),
                MeshMaterial3d(base_material.clone()),
                Transform::from_xyz(0.0, 0.2, 0.0),
                WorldObject,
            ));
            
            // Top detail
            commands.spawn((
                Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.6, 0.2, 0.6)))),
                MeshMaterial3d(detail_material.clone()),
                Transform::from_xyz(0.0, 0.5, 0.0),
                WorldObject,
            ));
            
            // Side details (smaller boxes)
            for i in 0..4 {
                let angle = i as f32 * std::f32::consts::FRAC_PI_2;
                let x = 0.35 * angle.cos();
                let z = 0.35 * angle.sin();
                
                commands.spawn((
                    Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.1, 0.3, 0.1)))),
                    MeshMaterial3d(detail_material.clone()),
                    Transform::from_xyz(x, 0.35, z),
                    WorldObject,
                ));
            }
        });
}