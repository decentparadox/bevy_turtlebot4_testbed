use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy_rapier3d::{
    geometry::{Collider, CollisionGroups, Group},
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use tracing::info;

mod turtlebot4;
mod camera;
mod camera_sensor;
mod drag;

const STATIC_GROUP: Group = Group::GROUP_1;
const CHASSIS_INTERNAL_GROUP: Group = Group::GROUP_2;
const CHASSIS_GROUP: Group = Group::GROUP_3;

#[derive(Component)]
struct Draggable;

/// Component to control camera input behavior
#[derive(Component)]
struct CameraController {
    enabled: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self { enabled: true }
    }
}

// System to toggle camera controls when dragging objects
fn manage_camera_controls(
    dragging_query: Query<Entity, (With<Draggable>, With<camera::DragTarget>)>,
    mut camera_query: Query<&mut CameraController, With<camera::PanOrbitCamera>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let is_dragging = !dragging_query.is_empty();
    
    for mut controller in camera_query.iter_mut() {
        // Disable camera when dragging objects, enable when not
        controller.enabled = !is_dragging;
    }
    
    // Allow manual toggle with C key for debugging
    if input.just_pressed(KeyCode::KeyC) {
        for mut controller in camera_query.iter_mut() {
            controller.enabled = !controller.enabled;
            info!("Camera controls {}", if controller.enabled { "enabled" } else { "disabled" });
        }
    }
}

// Modified accumulate system that respects camera controller
fn accumulate_mouse_events_system(
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut camera::PanOrbitCamera, &CameraController)>,
) {
    // Check if any camera controller allows input
    let camera_enabled = query.iter().any(|(_, controller)| controller.enabled);
    
    if !camera_enabled {
        // Clear events but don't process them
        ev_motion.clear();
        return;
    }
    
    // Rest of the existing logic
    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;
    
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Middle;

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.read() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) {
        for ev in ev_motion.read() {
            pan += ev.delta;
        }
    }
    for ev in ev_scroll.read() {
        scroll += ev.y;
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    for (mut camera, controller) in query.iter_mut() {
        if controller.enabled {
            camera.orbit_button_changed |= orbit_button_changed;
            camera.pan += 2.0 * pan;
            camera.rotation_move += 2.0 * rotation_move;
            camera.scroll += 2.0 * scroll;
        }
    }

    ev_motion.clear();
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
        // Add mesh picking plugin for 3D object interaction
        .add_plugins(MeshPickingPlugin)
        .add_systems(Startup, (setup, camera_sensor::setup_camera_preview_window))
        .add_systems(Update, (
            manage_camera_controls,
            accumulate_mouse_events_system,
            camera::update_camera_system, 
            camera_sensor::display_camera_preview,
            camera_sensor::update_camera_intrinsics,
            camera_sensor::debug_camera_pose,
        ))
        .add_systems(PostStartup, camera_sensor::setup_robot_camera_once)
        .add_systems(Update, render_origin)
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
        .insert(CameraController::default())
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