use bevy::prelude::*;
use bevy_rapier3d::dynamics::{RigidBody, ExternalImpulse, ImpulseJoint, RevoluteJointBuilder};
use bevy_rapier3d::geometry::{Collider, ColliderMassProperties, CollisionGroups, Group};
use crate::drag::{Draggable, DragTarget};

const STATIC_GROUP: Group = Group::GROUP_1;
const WHEEL_GROUP: Group = Group::GROUP_2;
const CHASSIS_GROUP: Group = Group::GROUP_3;

const MOTOR_STIFFNESS: f32 = 1000.0;
const MOTOR_DAMPING: f32 = 100.0;

#[derive(Component)]
enum Tag {
    WheelLeft,
    WheelRight,
    Chassis,
    Base,
    Link1,
    Link2,
    Link3,
    Link4,
    Link5,
    Link6,
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _asset_server: Res<AssetServer>
) {
    let translation = Vec3::new(1.0, 2.0, 2.0);
    let focus = Vec3::ZERO;
    let transform = Transform::from_translation(translation)
        .looking_at(focus, Vec3::Y);

    commands
        .spawn((
            Camera3d::default(),
            transform,
            crate::camera::PanOrbitCamera {
                focus,
                radius: translation.length(),
                ..default()
            },
        ))
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

    // lights
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

    // Simple floor
    commands.spawn((
        Collider::cuboid(2.0, 0.1, 2.0),
        CollisionGroups::new(STATIC_GROUP, WHEEL_GROUP | CHASSIS_GROUP),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ))
    .with_children(|commands| {
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(4.0, 4.0))),
            MeshMaterial3d(materials.add(Color::srgba(0.9, 0.9, 0.9, 1.0))),
            Transform::from_xyz(0.0, 0.1, 0.0),
        ));
    });

    // Load the UR3e robotic arm GLB files as a connected kinematic chain with proper joints
    info!("Loading UR3e robotic arm GLB files as connected kinematic chain with joints...");

    let robot_transform = Transform::IDENTITY;

    const BASE_HEIGHT: f32 = 0.0949500;
    const BASE_MASS: f32 = 0.1;
    const BASE_RADIUS: f32 = 0.0640000;

    /* spawn the base */
    let base_transform = robot_transform * Transform::from_xyz(0.0, 0.5 * BASE_HEIGHT, 0.0);
    let base = commands.spawn(Tag::Base)
        .insert((base_transform, Visibility::default()))
        .insert(RigidBody::Fixed)
        .with_children(|commands| {
            let transform = Transform::from_translation(Vec3::new(0.0, -0.5 * BASE_HEIGHT, 0.0))
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
            commands.spawn((transform, Visibility::default()))
                .insert(SceneRoot(_asset_server.load("robots/UR3e/base.glb#Scene0")));
        })
        .id();

    const LINK1_OFFSET: f32 = 0.0949500;
    const LINK1_HEIGHT: f32 = 0.1150854;
    const LINK1_MASS: f32 = 0.25;
    const LINK1_RADIUS: f32 = 0.0640000;

    /* spawn the link1 */
    let base_link1_joint = RevoluteJointBuilder::new(Vec3::Y)
        .local_anchor1(Vec3::new(0.0, 0.5 * BASE_HEIGHT, 0.0))
        .local_anchor2(Vec3::new(0.0, -0.5 * LINK1_HEIGHT, 0.0));

    let link1_transform = robot_transform * Transform::from_xyz(0.0, 0.5 * LINK1_HEIGHT + LINK1_OFFSET, 0.0);
    let link1 = commands.spawn(Tag::Link1)
        .insert((link1_transform, Visibility::default()))
        .insert(RigidBody::Dynamic)
        .insert(Collider::cylinder(0.5 * LINK1_HEIGHT, LINK1_RADIUS))
        .insert(ColliderMassProperties::Mass(LINK1_MASS))
        .insert(Draggable { external_impulse: ExternalImpulse::default() })
        .insert(DragTarget { is_dragging: false, drag_start_pos: Vec3::ZERO, drag_start_mouse_pos: Vec2::ZERO, entity: Entity::PLACEHOLDER })
        .insert(ImpulseJoint::new(base, base_link1_joint))
        .with_children(|commands| {
            let transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
                .with_rotation(Quat::from_euler(EulerRot::XYZ,
                    std::f32::consts::FRAC_PI_2,
                    0.0,
                    std::f32::consts::FRAC_PI_2));
            commands.spawn((transform, Visibility::default()))
                .insert(SceneRoot(_asset_server.load("robots/UR3e/link1.glb#Scene0")));
        })
        .id();

    // Update the drag target entity reference
    commands.entity(link1)
        .insert(DragTarget { is_dragging: false, drag_start_pos: Vec3::ZERO, drag_start_mouse_pos: Vec2::ZERO, entity: link1 });

    // Link 2
    const LINK2_OFFSET: f32 = 0.10489312;
    const LINK2_Z_OFFSET: f32 = 0.119850;
    const LINK2_HEIGHT: f32 = 0.3298858;
    const LINK2_MASS: f32 = 0.25;
    const LINK2_RADIUS: f32 = 0.033;

    let link1_link2_joint = RevoluteJointBuilder::new(Vec3::Z)
        .local_anchor1(Vec3::new(0.0, -0.0006427, -0.062950))
        .local_anchor2(Vec3::new(0.0, -0.1179571, 0.056900));

    let link2_transform = robot_transform * Transform::from_xyz(0.0, 0.5 * LINK2_HEIGHT + LINK2_OFFSET, -LINK2_Z_OFFSET);
    let link2 = commands.spawn(Tag::Link2)
        .insert((link2_transform, Visibility::default()))
        .insert(RigidBody::Dynamic)
        .insert(Collider::cylinder(0.5 * LINK2_HEIGHT, LINK2_RADIUS))
        .insert(ColliderMassProperties::Mass(LINK2_MASS))
        .insert(Draggable { external_impulse: ExternalImpulse::default() })
        .insert(DragTarget { is_dragging: false, drag_start_pos: Vec3::ZERO, drag_start_mouse_pos: Vec2::ZERO, entity: Entity::PLACEHOLDER })
        .insert(ImpulseJoint::new(link1, link1_link2_joint))
        .with_children(|commands| {
            const LINK2_MESH_OFFSET: f32 = -0.1179571;
            let transform = Transform::from_xyz(0.0, LINK2_MESH_OFFSET, 0.0)
                .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, -std::f32::consts::FRAC_PI_2));
            commands.spawn((transform, Visibility::default()))
                .insert(SceneRoot(_asset_server.load("robots/UR3e/link2.glb#Scene0")));
        })
        .id();

    commands.entity(link2)
        .insert(DragTarget { is_dragging: false, drag_start_pos: Vec3::ZERO, drag_start_mouse_pos: Vec2::ZERO, entity: link2 });

    // Link 3
    const LINK3_OFFSET: f32 = 0.357900;
    const LINK3_Z_OFFSET: f32 = 0.027000;
    const LINK3_HEIGHT: f32 = 0.28300617;
    const LINK3_MASS: f32 = 0.1;
    const LINK3_RADIUS: f32 = 0.027000;

    let link2_link3_joint = RevoluteJointBuilder::new(Vec3::Z)
        .local_anchor1(Vec3::new(0.0, 0.1255929, 0.046300))
        .local_anchor2(Vec3::new(0.0, -0.10400308, -0.046550));

    let link3_transform = robot_transform * Transform::from_xyz(0.0, 0.5 * LINK3_HEIGHT + LINK3_OFFSET, -LINK3_Z_OFFSET);
    let link3 = commands.spawn(Tag::Link3)
        .insert((link3_transform, Visibility::default()))
        .insert(RigidBody::Dynamic)
        .insert(Collider::cylinder(0.5 * LINK3_HEIGHT, LINK3_RADIUS))
        .insert(ColliderMassProperties::Mass(LINK3_MASS))
        .insert(Draggable { external_impulse: ExternalImpulse::default() })
        .insert(DragTarget { is_dragging: false, drag_start_pos: Vec3::ZERO, drag_start_mouse_pos: Vec2::ZERO, entity: Entity::PLACEHOLDER })
        .insert(ImpulseJoint::new(link2, link2_link3_joint))
        .with_children(|commands| {
            const LINK3_MESH_OFFSET: f32 = 0.109600;
            let transform = Transform::from_xyz(0.0, LINK3_MESH_OFFSET, 0.0)
                .with_rotation(Quat::from_euler(EulerRot::ZYX, -std::f32::consts::FRAC_PI_2, 0.0, std::f32::consts::PI));
            commands.spawn((transform, Visibility::default()))
                .insert(SceneRoot(_asset_server.load("robots/UR3e/link3.glb#Scene0")));
        })
        .id();

    commands.entity(link3)
        .insert(DragTarget { is_dragging: false, drag_start_pos: Vec3::ZERO, drag_start_mouse_pos: Vec2::ZERO, entity: link3 });

    // Link 4
    const LINK4_OFFSET: f32 = 0.56205387;
    const LINK4_Z_OFFSET: f32 = 0.131050;
    const LINK4_HEIGHT: f32 = 0.08964613;
    const LINK4_MASS: f32 = 0.1;
    const LINK4_RADIUS: f32 = 0.031500;

    let link3_link4_joint = RevoluteJointBuilder::new(Vec3::Z)
        .local_anchor1(Vec3::new(0.0, 0.10919692, -0.043100))
        .local_anchor2(Vec3::new(0.0, 0.00172307, 0.060950));

    let link4_transform = robot_transform * Transform::from_xyz(0.0, 0.5 * LINK4_HEIGHT + LINK4_OFFSET, -LINK4_Z_OFFSET);
    let link4 = commands.spawn(Tag::Link4)
        .insert((link4_transform, Visibility::default()))
        .insert(RigidBody::Dynamic)
        .insert(Collider::cylinder(0.5 * LINK4_HEIGHT, LINK4_RADIUS))
        .insert(ColliderMassProperties::Mass(LINK4_MASS))
        .insert(Draggable { external_impulse: ExternalImpulse::default() })
        .insert(DragTarget { is_dragging: false, drag_start_pos: Vec3::ZERO, drag_start_mouse_pos: Vec2::ZERO, entity: Entity::PLACEHOLDER })
        .insert(ImpulseJoint::new(link3, link3_link4_joint))
        .with_children(|commands| {
            const LINK4_MESH_OFFSET: f32 = 0.00172307;
            let transform = Transform::from_xyz(0.0, LINK4_MESH_OFFSET, 0.0)
                .with_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, -std::f32::consts::FRAC_PI_2, -std::f32::consts::FRAC_PI_2));
            commands.spawn((transform, Visibility::default()))
                .insert(SceneRoot(_asset_server.load("robots/UR3e/link4.glb#Scene0")));
        })
        .id();

    commands.entity(link4)
        .insert(DragTarget { is_dragging: false, drag_start_pos: Vec3::ZERO, drag_start_mouse_pos: Vec2::ZERO, entity: link4 });

    // Link 5
    const LINK5_OFFSET: f32 = 0.651700;
    const LINK5_Z_OFFSET: f32 = 0.131050;
    const LINK5_HEIGHT: f32 = 0.07455617;
    const LINK5_MASS: f32 = 0.1;
    const LINK5_RADIUS: f32 = 0.031500;

    let link4_link5_joint = RevoluteJointBuilder::new(Vec3::Y)
        .local_anchor1(Vec3::new(0.0, 0.04482307, 0.0))
        .local_anchor2(Vec3::new(0.0, -0.03725884, 0.0));

    let link5_transform = robot_transform * Transform::from_xyz(0.0, 0.5 * LINK5_HEIGHT + LINK5_OFFSET, -LINK5_Z_OFFSET);
    let link5 = commands.spawn(Tag::Link5)
        .insert((link5_transform, Visibility::default()))
        .insert(RigidBody::Dynamic)
        .insert(Collider::cylinder(0.5 * LINK5_HEIGHT, LINK5_RADIUS))
        .insert(ColliderMassProperties::Mass(LINK5_MASS))
        .insert(Draggable { external_impulse: ExternalImpulse::default() })
        .insert(DragTarget { is_dragging: false, drag_start_pos: Vec3::ZERO, drag_start_mouse_pos: Vec2::ZERO, entity: Entity::PLACEHOLDER })
        .insert(ImpulseJoint::new(link4, link4_link5_joint))
        .with_children(|commands| {
            const LINK5_MESH_OFFSET: f32 = 0.00499116;
            let transform = Transform::from_xyz(0.0, LINK5_MESH_OFFSET, 0.0)
                .with_rotation(Quat::from_euler(EulerRot::ZYX, -std::f32::consts::FRAC_PI_2, 0.0, std::f32::consts::PI));
            commands.spawn((transform, Visibility::default()))
                .insert(SceneRoot(_asset_server.load("robots/UR3e/link5.glb#Scene0")));
        })
        .id();

    commands.entity(link5)
        .insert(DragTarget { is_dragging: false, drag_start_pos: Vec3::ZERO, drag_start_mouse_pos: Vec2::ZERO, entity: link5 });

    // Link 6 (End Effector)
    const LINK6_OFFSET: f32 = 0.693950;
    const LINK6_Z_OFFSET: f32 = 0.198650;
    const LINK6_HEIGHT: f32 = 0.049000;
    const LINK6_MASS: f32 = 0.1;
    const LINK6_RADIUS: f32 = 0.031500;

    let link5_link6_joint = RevoluteJointBuilder::new(Vec3::Z)
        .local_anchor1(Vec3::new(0.0, 0.00499116, -0.043100))
        .local_anchor2(Vec3::new(0.0, 0.024500, 0.0));

    let link6_transform = robot_transform * Transform::from_xyz(0.0, LINK6_OFFSET, -LINK6_Z_OFFSET)
        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    let link6 = commands.spawn(Tag::Link6)
        .insert((link6_transform, Visibility::default()))
        .insert(RigidBody::Dynamic)
        .insert(Collider::cylinder(0.5 * LINK6_HEIGHT, LINK6_RADIUS))
        .insert(ColliderMassProperties::Mass(LINK6_MASS))
        .insert(Draggable { external_impulse: ExternalImpulse::default() })
        .insert(DragTarget { is_dragging: false, drag_start_pos: Vec3::ZERO, drag_start_mouse_pos: Vec2::ZERO, entity: Entity::PLACEHOLDER })
        .insert(ImpulseJoint::new(link5, link5_link6_joint))
        .with_children(|commands| {
            const LINK6_MESH_OFFSET: f32 = -0.5 * LINK6_HEIGHT;
            let transform = Transform::from_xyz(0.0, LINK6_MESH_OFFSET, 0.0)
                .with_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 0.0, 0.0));
            commands.spawn((transform, Visibility::default()))
                .insert(SceneRoot(_asset_server.load("robots/UR3e/link6.glb#Scene0")));
        })
        .id();

    commands.entity(link6)
        .insert(DragTarget { is_dragging: false, drag_start_pos: Vec3::ZERO, drag_start_mouse_pos: Vec2::ZERO, entity: link6 });

    info!("Robotic arm setup complete - GLB files loaded with full kinematic chain!");
}
