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

// Observer system for handling drag events on the turtlebot
fn on_drag_robot(
    drag: Trigger<Pointer<Drag>>,
    mut draggable_objects: Query<&mut ExternalImpulse, With<Draggable>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Draggable>)>,
) {
    if let Ok(mut impulse) = draggable_objects.get_mut(drag.target()) {
        if let Ok(camera_transform) = camera_query.get_single() {
            // Get camera's right and forward vectors (in XZ plane)
            let camera_right = camera_transform.right();
            let camera_forward = camera_transform.forward();
            
            // Project vectors onto XZ plane and normalize
            let camera_right_xz = Vec3::new(camera_right.x, 0.0, camera_right.z).normalize();
            let camera_forward_xz = Vec3::new(camera_forward.x, 0.0, camera_forward.z).normalize();
            
            // Transform mouse delta relative to camera orientation
            let force_multiplier = 0.05;
            let force = camera_right_xz * drag.delta.x * force_multiplier 
                      - camera_forward_xz * drag.delta.y * force_multiplier; // Negative because forward drag should move forward
            
            impulse.impulse = force;
        }
    }
}

// Observer system for handling click events on the turtlebot
fn on_click_robot(
    _click: Trigger<Pointer<Click>>,
    mut draggable_objects: Query<&mut ExternalImpulse, With<Draggable>>,
) {
    // Apply upward impulse when clicked
    for mut impulse in draggable_objects.iter_mut() {
        impulse.impulse = Vec3::new(0.0, 100.0, 0.0);
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
        // Add the MeshPickingPlugin for 3D mesh picking
        .add_plugins(MeshPickingPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (camera::update_camera_system, camera::accumulate_mouse_events_system))
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