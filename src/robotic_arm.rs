use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::dynamics::TypedJoint;
use crate::robot_drag::{Draggable, DraggableBundle};

const STATIC_GROUP: Group = Group::GROUP_1;

#[derive(Component, Debug, PartialEq, Eq)]
pub enum ArmLink {
    Base,
    Link1,
    Link2,
    Link3,
    Link4,
    Link5,
    Link6,
}

const MOTOR_STIFFNESS: f32 = 10000.0;
const MOTOR_DAMPING: f32 = 1000.0;

fn spawn_ur3e_arm(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    robot_transform: Transform,
) {
    const BASE_HEIGHT: f32 = 0.0949500;
    const BASE_MASS: f32 = 0.1;
    const BASE_RADIUS: f32 = 0.0640000;

    const LINK1_OFFSET: f32 = 0.0949500;
    const LINK1_HEIGHT: f32 = 0.1150854;
    const LINK1_MASS: f32 = 0.25;
    const LINK1_RADIUS: f32 = 0.0640000;

    const LINK2_OFFSET: f32 = 0.10489312;
    const LINK2_Z_OFFSET: f32 = 0.119850;
    const LINK2_HEIGHT: f32 = 0.3298858;
    const LINK2_MASS: f32 = 0.25;
    const LINK2_RADIUS: f32 = 0.033;

    const LINK3_OFFSET: f32 = 0.357900;
    const LINK3_Z_OFFSET: f32 = 0.027000;
    const LINK3_HEIGHT: f32 = 0.28300617;
    const LINK3_MASS: f32 = 0.1;
    const LINK3_RADIUS: f32 = 0.027000;

    const LINK4_OFFSET: f32 = 0.56205387;
    const LINK4_Z_OFFSET: f32 = 0.131050;
    const LINK4_HEIGHT: f32 = 0.08964613;
    const LINK4_MASS: f32 = 0.1;
    const LINK4_RADIUS: f32 = 0.031500;

    const LINK5_OFFSET: f32 = 0.651700;
    const LINK5_Z_OFFSET: f32 = 0.131050;
    const LINK5_HEIGHT: f32 = 0.07455617;
    const LINK5_MASS: f32 = 0.1;
    const LINK5_RADIUS: f32 = 0.031500;

    const LINK6_OFFSET: f32 = 0.693950;
    const LINK6_Z_OFFSET: f32 = 0.198650;
    const LINK6_HEIGHT: f32 = 0.049000;
    const LINK6_MASS: f32 = 0.1;
    const LINK6_RADIUS: f32 = 0.031500;

    commands.spawn((Transform::default(), Visibility::default())).with_children(|commands| {
        // Base
        let base_transform = robot_transform * Transform::from_xyz(0.0, 0.5 * BASE_HEIGHT, 0.0);
        let base = commands.spawn(ArmLink::Base)
            .insert(Transform::from(base_transform))
            .insert(Visibility::default())
            .insert(RigidBody::Fixed)
            .insert(Collider::cylinder(BASE_HEIGHT * 0.5, BASE_RADIUS))
            .insert(ColliderMassProperties::Mass(BASE_MASS))
            .with_children(|commands| {
                let transform = Transform::from_translation(Vec3::new(0.0, -0.5 * BASE_HEIGHT, 0.0))
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
                commands.spawn(SceneRoot(asset_server.load::<Scene>("robots/UR3e/base.glb#Scene0")))
                    .insert(transform);
            })
            .id();

        // Link 1
        let base_link1_joint = GenericJointBuilder::new(JointAxesMask::LOCKED_REVOLUTE_AXES)
            .local_axis1(Vec3::Y)
            .local_axis2(Vec3::Y)
            .local_anchor1(Vec3::new(0.0, 0.5 * BASE_HEIGHT, 0.0))
            .local_anchor2(Vec3::new(0.0, -0.5 * LINK1_HEIGHT, 0.0))
            .motor_position(JointAxis::AngX, 0.0, MOTOR_STIFFNESS, MOTOR_DAMPING);

        let link1_transform = robot_transform * Transform::from_xyz(0.0, 0.5 * LINK1_HEIGHT + LINK1_OFFSET, 0.0);
        let link1 = commands.spawn(ArmLink::Link1)
            .insert(link1_transform)
            .insert(Visibility::default())
            .insert(RigidBody::Dynamic)
            .insert(Collider::cylinder(0.5 * LINK1_HEIGHT, LINK1_RADIUS))
            .insert(ColliderMassProperties::Mass(LINK1_MASS))
            .insert(ImpulseJoint::new(base, TypedJoint::GenericJoint(base_link1_joint.build())))
            .insert(Draggable)
            .with_children(|commands| {
                let transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
                    .with_rotation(Quat::from_euler(EulerRot::XYZ,
                        std::f32::consts::FRAC_PI_2,
                        0.0,
                        std::f32::consts::FRAC_PI_2));
                commands.spawn(SceneRoot(asset_server.load::<Scene>("robots/UR3e/link1.glb#Scene0")))
                    .insert(transform);
            })
            .id();

        // Link 2
        let link1_link2_joint = GenericJointBuilder::new(JointAxesMask::LOCKED_REVOLUTE_AXES)
            .local_axis1(Vec3::Z)
            .local_axis2(Vec3::Z)
            .local_anchor1(Vec3::new(0.0, -0.0006427, -0.062950))
            .local_anchor2(Vec3::new(0.0, -0.1179571, 0.056900))
            .motor_position(JointAxis::AngX, 0.0, MOTOR_STIFFNESS, MOTOR_DAMPING);

        let link2_transform = robot_transform * Transform::from_xyz(0.0, 0.5 * LINK2_HEIGHT + LINK2_OFFSET, -LINK2_Z_OFFSET);
        let link2 = commands.spawn(ArmLink::Link2)
            .insert(link2_transform)
            .insert(Visibility::default())
            .insert(RigidBody::Dynamic)
            .insert(Collider::cylinder(0.5 * LINK2_HEIGHT, LINK2_RADIUS))
            .insert(ColliderMassProperties::Mass(LINK2_MASS))
            .insert(ImpulseJoint::new(link1, TypedJoint::GenericJoint(link1_link2_joint.build())))
            .insert(Draggable)
            .with_children(|commands| {
                const LINK2_MESH_OFFSET: f32 = -0.1179571;
                let transform = Transform::from_xyz(0.0, LINK2_MESH_OFFSET, 0.0)
                    .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, -std::f32::consts::FRAC_PI_2));
                commands.spawn(SceneRoot(asset_server.load::<Scene>("robots/UR3e/link2.glb#Scene0")))
                    .insert(transform);
            })
            .id();

        // Link 3
        let link2_link3_joint = GenericJointBuilder::new(JointAxesMask::LOCKED_REVOLUTE_AXES)
            .local_axis1(Vec3::Z)
            .local_axis2(Vec3::Z)
            .local_anchor1(Vec3::new(0.0, 0.1255929, 0.046300))
            .local_anchor2(Vec3::new(0.0, -0.10400308, -0.046550))
            .motor_position(JointAxis::AngX, 0.0, MOTOR_STIFFNESS, MOTOR_DAMPING);

        let link3_transform = robot_transform * Transform::from_xyz(0.0, 0.5 * LINK3_HEIGHT + LINK3_OFFSET, -LINK3_Z_OFFSET);
        let link3 = commands.spawn(ArmLink::Link3)
            .insert(link3_transform)
            .insert(Visibility::default())
            .insert(RigidBody::Dynamic)
            .insert(Collider::cylinder(0.5 * LINK3_HEIGHT, LINK3_RADIUS))
            .insert(ColliderMassProperties::Mass(LINK3_MASS))
            .insert(ImpulseJoint::new(link2, TypedJoint::GenericJoint(link2_link3_joint.build())))
            .insert(Draggable)
            .with_children(|commands| {
                const LINK3_MESH_OFFSET: f32 = 0.109600;
                let transform = Transform::from_xyz(0.0, LINK3_MESH_OFFSET, 0.0)
                    .with_rotation(Quat::from_euler(EulerRot::ZYX,
                        -std::f32::consts::FRAC_PI_2,
                        0.0,
                        std::f32::consts::PI));
                commands.spawn(SceneRoot(asset_server.load::<Scene>("robots/UR3e/link3.glb#Scene0")))
                    .insert(transform);
            })
            .id();

        // Link 4
        let link3_link4_joint = GenericJointBuilder::new(JointAxesMask::LOCKED_REVOLUTE_AXES)
            .local_axis1(Vec3::Z)
            .local_axis2(Vec3::Z)
            .local_anchor1(Vec3::new(0.0, 0.10919692, -0.043100))
            .local_anchor2(Vec3::new(0.0, 0.00172307, 0.060950))
            .motor_position(JointAxis::AngX, 0.0, MOTOR_STIFFNESS, MOTOR_DAMPING);

        let link4_transform = robot_transform * Transform::from_xyz(0.0, 0.5 * LINK4_HEIGHT + LINK4_OFFSET, -LINK4_Z_OFFSET);
        let link4 = commands.spawn(ArmLink::Link4)
            .insert(link4_transform)
            .insert(Visibility::default())
            .insert(RigidBody::Dynamic)
            .insert(Collider::cylinder(0.5 * LINK4_HEIGHT, LINK4_RADIUS))
            .insert(ColliderMassProperties::Mass(LINK4_MASS))
            .insert(ImpulseJoint::new(link3, TypedJoint::GenericJoint(link3_link4_joint.build())))
            .insert(Draggable)
            .with_children(|commands| {
                const LINK4_MESH_OFFSET: f32 = 0.00172307;
                let transform = Transform::from_xyz(0.0, LINK4_MESH_OFFSET, 0.0)
                    .with_rotation(Quat::from_euler(EulerRot::ZYX,
                        0.0,
                        -std::f32::consts::FRAC_PI_2,
                        -std::f32::consts::FRAC_PI_2));
                commands.spawn(SceneRoot(asset_server.load::<Scene>("robots/UR3e/link4.glb#Scene0")))
                    .insert(transform);
            })
            .id();

        // Link 5
        let link4_link5_joint = GenericJointBuilder::new(JointAxesMask::LOCKED_REVOLUTE_AXES)
            .local_axis1(Vec3::Y)
            .local_axis2(Vec3::Y)
            .local_anchor1(Vec3::new(0.0, 0.04482307, 0.0))
            .local_anchor2(Vec3::new(0.0, -0.03725884, 0.0))
            .motor_position(JointAxis::AngX, 0.0, MOTOR_STIFFNESS, MOTOR_DAMPING);

        let link5_transform = robot_transform * Transform::from_xyz(0.0, 0.5 * LINK5_HEIGHT + LINK5_OFFSET, -LINK5_Z_OFFSET);
        let link5 = commands.spawn(ArmLink::Link5)
            .insert(link5_transform)
            .insert(Visibility::default())
            .insert(RigidBody::Dynamic)
            .insert(Collider::cylinder(0.5 * LINK5_HEIGHT, LINK5_RADIUS))
            .insert(ColliderMassProperties::Mass(LINK5_MASS))
            .insert(ImpulseJoint::new(link4, TypedJoint::GenericJoint(link4_link5_joint.build())))
            .insert(Draggable)
            .with_children(|commands| {
                const LINK5_MESH_OFFSET: f32 = 0.00499116;
                let transform = Transform::from_xyz(0.0, LINK5_MESH_OFFSET, 0.0)
                    .with_rotation(Quat::from_euler(EulerRot::ZYX,
                        -std::f32::consts::FRAC_PI_2,
                        0.0,
                        std::f32::consts::PI));
                commands.spawn(SceneRoot(asset_server.load::<Scene>("robots/UR3e/link5.glb#Scene0")))
                    .insert(transform);
            })
            .id();

        // Link 6 (End Effector)
        let link5_link6_joint = GenericJointBuilder::new(JointAxesMask::LOCKED_REVOLUTE_AXES)
            .local_axis1(Vec3::Z)
            .local_axis2(Vec3::Y)
            .local_anchor1(Vec3::new(0.0, 0.00499116, -0.043100))
            .local_anchor2(Vec3::new(0.0, 0.024500, 0.0))
            .motor_position(JointAxis::AngX, 0.0, MOTOR_STIFFNESS, MOTOR_DAMPING);

        let link6_transform = robot_transform * Transform::from_xyz(0.0, LINK6_OFFSET, -LINK6_Z_OFFSET)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
        let _link6 = commands.spawn(ArmLink::Link6)
            .insert(link6_transform)
            .insert(Visibility::default())
            .insert(RigidBody::Dynamic)
            .insert(Collider::cylinder(0.5 * LINK6_HEIGHT, LINK6_RADIUS))
            .insert(ColliderMassProperties::Mass(LINK6_MASS))
            .insert(ImpulseJoint::new(link5, TypedJoint::GenericJoint(link5_link6_joint.build())))
            .insert(Draggable)
            .insert(DraggableBundle::default())
            .with_children(|commands| {
                const LINK6_MESH_OFFSET: f32 = -0.5 * LINK6_HEIGHT;
                let transform = Transform::from_xyz(0.0, LINK6_MESH_OFFSET, 0.0)
                    .with_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 0.0, 0.0));
                commands.spawn(SceneRoot(asset_server.load::<Scene>("robots/UR3e/link6.glb#Scene0")))
                    .insert(transform);
            })
            .id();
    });
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    let camera_translation = Vec3::new(2.0, 2.0, 2.0);
    let camera_focus = Vec3::ZERO;
    commands.spawn(Camera3d::default())
        .insert(Transform::from_translation(camera_translation).looking_at(camera_focus, Vec3::Y))
        .insert(crate::camera::PanOrbitCamera {
            focus: camera_focus,
            radius: camera_translation.length(),
            ..default()
        });

    // Light
    commands.spawn(DirectionalLight {
        illuminance: 10000.0,
        shadows_enabled: true,
        ..default()
    })
    .insert(Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::default(),
        RigidBody::Fixed,
        Collider::cuboid(5.0, 0.01, 5.0),
    ));

    // Spawn robotic arm
    spawn_ur3e_arm(&mut commands, &asset_server, Transform::IDENTITY);
}

// Track current target positions for each joint
#[derive(Resource, Default)]
pub struct JointTargets {
    pub positions: Vec<f32>,
}

pub fn keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut joint_targets: ResMut<JointTargets>,
    mut joint_query: Query<(&mut ImpulseJoint, &ArmLink)>,
) {
    // Initialize joint targets if empty
    if joint_targets.positions.is_empty() {
        joint_targets.positions = vec![0.0; 6]; // 6 joints
    }

    // Only control the first joint for now to prevent unrealistic behavior
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        joint_targets.positions[0] += 0.005; // Smaller increments for stability
        joint_targets.positions[0] = joint_targets.positions[0].clamp(-1.57, 1.57); // ±90 degrees
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        joint_targets.positions[0] -= 0.005;
        joint_targets.positions[0] = joint_targets.positions[0].clamp(-1.57, 1.57); // ±90 degrees
    }

    // Apply motor control only to Link1 for now
    for (mut joint, arm_link) in joint_query.iter_mut() {
        if matches!(arm_link, ArmLink::Link1) {
            if let TypedJoint::GenericJoint(generic_joint) = &mut joint.data {
                // Use lower stiffness and higher damping for stability
                const MOTOR_STIFFNESS: f32 = 5000.0;
                const MOTOR_DAMPING: f32 = 2000.0;
                
                generic_joint.set_motor_position(
                    JointAxis::AngX, 
                    joint_targets.positions[0], 
                    MOTOR_STIFFNESS, 
                    MOTOR_DAMPING
                );
            }
        }
    }
}
