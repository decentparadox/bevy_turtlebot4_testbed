use bevy::prelude::*;
use bevy_rapier3d::{
    geometry::{Collider, CollisionGroups, Group},
    plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin, TimestepMode},
    render::RapierDebugRenderPlugin};

mod turtlebot4;
mod camera;
mod drag;

const STATIC_GROUP: Group = Group::GROUP_1;
const CHASSIS_INTERNAL_GROUP: Group = Group::GROUP_2;
const CHASSIS_GROUP: Group = Group::GROUP_3;

fn enable_physics(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut rapier: ResMut<RapierConfiguration>
) {
    if keyboard_input.pressed(KeyCode::Space) {
        // Note: these fields were renamed in newer versions
        // rapier.physics_pipeline_active = true;
        // rapier.query_pipeline_active = true;
    }
}

pub fn main() {
    bevy::app::App::new()
        .insert_resource(ClearColor(Color::srgb(0.98, 0.92, 0.84))) // ANTIQUE_WHITE equivalent
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 500.0,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (camera::update_camera_system, camera::accumulate_mouse_events_system))
        .add_systems(Update, render_origin)
        .add_systems(Update, turtlebot4::velocity_control)
        //.add_systems(PreUpdate, enable_physics)
        .run();
}

fn render_origin(mut gizmos: Gizmos) {
    gizmos.line(Vec3::ZERO, Vec3::X, Color::srgb(1.0, 0.0, 0.0)); // RED equivalent
    gizmos.line(Vec3::ZERO, Vec3::Y, Color::srgb(0.0, 1.0, 0.0)); // GREEN equivalent
    gizmos.line(Vec3::ZERO, Vec3::Z, Color::srgb(0.0, 0.0, 1.0)); // BLUE equivalent
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
        .spawn(Camera3d::default())
        .insert(Transform::from_translation(translation).looking_at(focus, Vec3::Y))
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
            ));
        });

    //lights (note ambient light use in the app resources)
    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(5.0, 5.0, 0.0),
    ));
    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(-5.0, 5.0, 0.0),
    ));
    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(0.0, 5.0, 5.0),
    ));
    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(0.0, 5.0, -5.0),
    ));

    // wall parameters
    let wall_height = 0.075;
    let wall_thickness = 0.075;
    let wall_length = 4.0;
    let wall_color = Color::srgb(0.7, 0.7, 0.7);

    // north wall
    commands
        .spawn((
            Collider::cuboid((wall_length - wall_thickness) * 0.5, wall_height * 0.5, wall_thickness * 0.5),
            CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP),
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(wall_length - wall_thickness, wall_height, wall_thickness)))),
            MeshMaterial3d(materials.add(wall_color)),
            Transform::from_xyz(-wall_thickness * 0.5, wall_height * 0.5, (-wall_length + wall_thickness) * 0.5),
        ));

    // east wall
    commands
        .spawn((
            Collider::cuboid(wall_thickness * 0.5, wall_height * 0.5, (wall_length - wall_thickness) * 0.5),
            CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP),
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(wall_thickness, wall_height, wall_length - wall_thickness)))),
            MeshMaterial3d(materials.add(wall_color)),
            Transform::from_xyz((wall_length - wall_thickness) * 0.5, wall_height * 0.5, -wall_thickness * 0.5),
        ));

    // south wall
    commands
        .spawn((
            Collider::cuboid((wall_length - wall_thickness) * 0.5, wall_height * 0.5, wall_thickness * 0.5),
            CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP),
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(wall_length - wall_thickness, wall_height, wall_thickness)))),
            MeshMaterial3d(materials.add(wall_color)),
            Transform::from_xyz(wall_thickness * 0.5, wall_height * 0.5, (wall_length - wall_thickness) * 0.5),
        ));

    // west wall
    commands
        .spawn((
            Collider::cuboid(wall_thickness * 0.5, wall_height * 0.5, (wall_length - wall_thickness) * 0.5),
            CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP),
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(wall_thickness, wall_height, wall_length - wall_thickness)))),
            MeshMaterial3d(materials.add(wall_color)),
            Transform::from_xyz((-wall_length + wall_thickness) * 0.5, wall_height * 0.5, wall_thickness * 0.5),
        ));

    // floor
    commands
        .spawn((
            Collider::cuboid(2.0, 0.1, 2.0),
            CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP),
            Transform::from_xyz(0.0, -0.1, 0.0),
        ))
        .with_children(|commands| {
            commands.spawn((
                Mesh3d(meshes.add(Plane3d::default().mesh().size(4.0, 4.0))),
                MeshMaterial3d(materials.add(Color::srgba(0.9, 0.9, 0.9, 1.0))),
                Transform::from_xyz(0.0, 0.1, 0.0),
            ));
        });

    // robot
    turtlebot4::spawn(&mut commands, &asset_server, &Transform::IDENTITY)
}