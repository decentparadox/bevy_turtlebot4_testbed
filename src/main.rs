use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
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
        rapier.physics_pipeline_active = true;
        rapier.query_pipeline_active = true;
    }
    // else {
    //     rapier.physics_pipeline_active = false;
    //     rapier.query_pipeline_active = false;
    // }
}

pub fn main() {
    bevy::app::App::new()
        .insert_resource(ClearColor(Color::ANTIQUE_WHITE))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 500.0,
        })
        .insert_resource(RapierConfiguration {
            timestep_mode: TimestepMode::Fixed { dt: 0.05, substeps: 20 },
            // // note that picking will not work if (query) pipeline is not active
            // physics_pipeline_active: false,
            // query_pipeline_active: false,
            // gravity: Vec3::ZERO,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (camera::update_camera_system, camera::accumulate_mouse_events_system))
        .add_systems(Update, render_origin)
        .add_systems(Update, drag::drag_system)
        .add_systems(Update, turtlebot4::velocity_control)
        //.add_systems(PreUpdate, enable_physics)
        .run();
}

fn render_origin(mut gizmos: Gizmos) {
    gizmos.line(Vec3::ZERO, Vec3::X, Color::RED);
    gizmos.line(Vec3::ZERO, Vec3::Y, Color::GREEN);
    gizmos.line(Vec3::ZERO, Vec3::Z, Color::BLUE);
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
        .spawn(Camera3dBundle {
            transform,
            ..default()
        })
        .insert(camera::PanOrbitCamera {
            focus,
            radius: translation.length(),
            ..default()
        })
        .insert(VisibilityBundle::default())
        .with_children(|commands| {
            commands.spawn(DirectionalLightBundle {
                directional_light: DirectionalLight {
                    shadows_enabled: false,
                    illuminance: 1000.0,
                    ..default()
                },
                transform: Transform::from_xyz(-2.5, 2.5, 2.5)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            });
        });

    //lights (note ambient light use in the app resources)
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(5.0, 5.0, 0.0),
        ..default()
    });
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(-5.0, 5.0, 0.0),
        ..default()
    });
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(0.0, 5.0, 5.0),
        ..default()
    });
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(0.0, 5.0, -5.0),
        ..default()
    });

    // wall parameters
    let wall_height = 0.075;
    let wall_thickness = 0.075;
    let wall_length = 4.0;
    let wall_color = Color::rgb(0.7, 0.7, 0.7);

    // north wall
    commands
        .spawn(Collider::cuboid((wall_length - wall_thickness) * 0.5, wall_height * 0.5, wall_thickness * 0.5))
        .insert(CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(wall_length - wall_thickness, wall_height, wall_thickness))),
            material: materials.add(wall_color),
            transform: Transform::from_xyz(-wall_thickness * 0.5, wall_height * 0.5, (-wall_length + wall_thickness) * 0.5),
            ..default()
        });

    // east wall
    commands
        .spawn(Collider::cuboid(wall_thickness * 0.5, wall_height * 0.5, (wall_length - wall_thickness) * 0.5))
        .insert(CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(wall_thickness, wall_height, wall_length - wall_thickness))),
            material: materials.add(wall_color),
            transform: Transform::from_xyz((wall_length - wall_thickness) * 0.5, wall_height * 0.5, -wall_thickness * 0.5),
            ..default()
        });

    // south wall
    commands
        .spawn(Collider::cuboid((wall_length - wall_thickness) * 0.5, wall_height * 0.5, wall_thickness * 0.5))
        .insert(CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(wall_length - wall_thickness, wall_height, wall_thickness))),
            material: materials.add(wall_color),
            transform: Transform::from_xyz(wall_thickness * 0.5, wall_height * 0.5, (wall_length - wall_thickness) * 0.5),
            ..default()
        });

    // west wall
    commands
        .spawn(Collider::cuboid(wall_thickness * 0.5, wall_height * 0.5, (wall_length - wall_thickness) * 0.5))
        .insert(CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(wall_thickness, wall_height, wall_length - wall_thickness))),
            material: materials.add(wall_color),
            transform: Transform::from_xyz((-wall_length + wall_thickness) * 0.5, wall_height * 0.5, wall_thickness * 0.5),
            ..default()
        });

    // floor
    commands
        .spawn(Collider::cuboid(2.0, 0.1, 2.0))
        .insert(CollisionGroups::new(STATIC_GROUP, CHASSIS_INTERNAL_GROUP | CHASSIS_GROUP))
        .insert(SpatialBundle::from_transform(Transform::from_xyz(0.0, -0.1, 0.0)))
        .with_children(|commands| {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Plane3d::default().mesh().size(4.0, 4.0)),
                material: materials.add(Color::rgba(0.9, 0.9, 0.9, 1.0)),
                transform: Transform::from_xyz(0.0, 0.1, 0.0),
                ..default()
            });
        });

    // robot
    turtlebot4::spawn(&mut commands, &asset_server, &Transform::IDENTITY)
}