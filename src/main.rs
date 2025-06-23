use bevy::prelude::*;
use bevy::input::mouse::MouseButtonInput;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::{
    geometry::{Collider, CollisionGroups, Group},
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
    dynamics::ExternalImpulse,
};

mod turtlebot4;
mod camera;

const STATIC_GROUP: Group = Group::GROUP_1;
const CHASSIS_INTERNAL_GROUP: Group = Group::GROUP_2;
const CHASSIS_GROUP: Group = Group::GROUP_3;

#[derive(Component)]
struct Draggable;

// Simple keyboard-based control system for dragging the turtlebot
fn keyboard_control_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut draggable_objects: Query<&mut ExternalImpulse, With<Draggable>>,
) {
    for mut impulse in draggable_objects.iter_mut() {
        let mut force = Vec3::ZERO;
        
        // WASD for movement
        if keyboard.pressed(KeyCode::KeyW) {
            force.z -= 10.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            force.z += 10.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            force.x -= 10.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            force.x += 10.0;
        }
        // Space for up, Shift for down
        if keyboard.pressed(KeyCode::Space) {
            force.y += 10.0;
        }
        if keyboard.pressed(KeyCode::ShiftLeft) {
            force.y -= 10.0;
        }
        
        impulse.impulse = force;
    }
}

pub fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.98, 0.92, 0.84)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 500.0,
            affects_lightmapped_meshes: true,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (camera::update_camera_system, camera::accumulate_mouse_events_system))
        .add_systems(Update, render_origin)
        .add_systems(Update, keyboard_control_system)
        .run();
}

fn render_origin(mut gizmos: Gizmos) {
    gizmos.line(Vec3::ZERO, Vec3::X, Color::srgb(1.0, 0.0, 0.0));
    gizmos.line(Vec3::ZERO, Vec3::Y, Color::srgb(0.0, 1.0, 0.0));
    gizmos.line(Vec3::ZERO, Vec3::Z, Color::srgb(0.0, 0.0, 1.0));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>
) {
    let translation = Vec3::new(1.0, 2.0, 2.0);
    let focus = Vec3::ZERO;
    let transform = Transform::from_translation(translation)
        .looking_at(focus, Vec3::Y);    

    commands
        .spawn((
            Camera3d::default(),
            transform,
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .insert(camera::PanOrbitCamera {
            focus,
            radius: translation.length(),
            ..default()
        })
        .with_children(|commands| {
            commands.spawn((
                DirectionalLight {
                    shadows_enabled: false,
                    illuminance: 1000.0,
                    ..default()
                },
                Transform::from_xyz(-2.5, 2.5, 2.5)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        });

    // Point lights
    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(5.0, 5.0, 0.0),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(-5.0, 5.0, 0.0),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(0.0, 5.0, 5.0),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(0.0, 5.0, -5.0),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));

    // Wall setup (walls)
    let wall_height = 0.075;
    let wall_thickness = 0.075;
    let wall_length = 4.0;
    let wall_color = Color::srgb(0.7, 0.7, 0.7);

    // North wall
    commands
        .spawn(Collider::cuboid((wall_length - wall_thickness) * 0.5, wall_height * 0.5, wall_thickness * 0.5))
        .insert(CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP))
        .insert((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(wall_length - wall_thickness, wall_height, wall_thickness)))),
            MeshMaterial3d(materials.add(wall_color)),
            Transform::from_xyz(-wall_thickness * 0.5, wall_height * 0.5, (-wall_length + wall_thickness) * 0.5),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));

    // East wall
    commands
        .spawn(Collider::cuboid(wall_thickness * 0.5, wall_height * 0.5, (wall_length - wall_thickness) * 0.5))
        .insert(CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP))
        .insert((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(wall_thickness, wall_height, wall_length - wall_thickness)))),
            MeshMaterial3d(materials.add(wall_color)),
            Transform::from_xyz((wall_length - wall_thickness) * 0.5, wall_height * 0.5, -wall_thickness * 0.5),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));

    // South wall
    commands
        .spawn(Collider::cuboid((wall_length - wall_thickness) * 0.5, wall_height * 0.5, wall_thickness * 0.5))
        .insert(CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP))
        .insert((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(wall_length - wall_thickness, wall_height, wall_thickness)))),
            MeshMaterial3d(materials.add(wall_color)),
            Transform::from_xyz(wall_thickness * 0.5, wall_height * 0.5, (wall_length - wall_thickness) * 0.5),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));

    // West wall
    commands
        .spawn(Collider::cuboid(wall_thickness * 0.5, wall_height * 0.5, (wall_length - wall_thickness) * 0.5))
        .insert(CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP))
        .insert((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(wall_thickness, wall_height, wall_length - wall_thickness)))),
            MeshMaterial3d(materials.add(wall_color)),
            Transform::from_xyz((-wall_length + wall_thickness) * 0.5, wall_height * 0.5, wall_thickness * 0.5),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));

    // Floor
    commands
        .spawn(Collider::cuboid(2.0, 0.1, 2.0))
        .insert(CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP))
        .insert((
            Transform::from_xyz(0.0, -0.1, 0.0),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .with_children(|commands| {
            commands.spawn((
                Mesh3d(meshes.add(Plane3d::default().mesh().size(4.0, 4.0))),
                MeshMaterial3d(materials.add(Color::srgba(0.9, 0.9, 0.9, 1.0))),
                Transform::from_xyz(0.0, 0.1, 0.0),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        });

    // Robot
    turtlebot4::spawn(&mut commands, &asset_server, &Transform::from_xyz(0.0, 0.5, 0.0));
}