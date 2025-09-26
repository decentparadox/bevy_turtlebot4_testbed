use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::dynamics::TypedJoint;
use crate::robot_drag::{Draggable, DraggableBundle};
use bevy::ecs::event::EventReader;

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
    GripperBase,
    GripperFinger1,
    GripperFinger2,
}

#[derive(Component)]
pub struct SimpleGripper {
    pub is_open: bool,
    pub grip_strength: f32,
}

#[derive(Component)]
pub struct PickupBlock;

#[derive(Component)]
pub struct GrippedObject {
    pub original_parent: Option<Entity>,
}

#[derive(Component)]
pub struct OriginalTransform {
    pub transform: Transform,
}

#[derive(Component)]
pub struct DragState {
    pub is_being_dragged: bool,
    pub was_dragged: bool,
    pub return_timer: f32,
    pub return_duration: f32,
}

impl Default for DragState {
    fn default() -> Self {
        Self {
            is_being_dragged: false,
            was_dragged: false,
            return_timer: 0.0,
            return_duration: 2.0, // 2 seconds to return to original position
        }
    }
}

const MOTOR_STIFFNESS: f32 = 10000.0;
const MOTOR_DAMPING: f32 = 1000.0;

#[allow(unused_variables)]
fn spawn_ur3e_arm(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    robot_transform: Transform,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
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

    let arm_root = commands.spawn((Transform::default(), Visibility::default())).id();

    // Spawn base separately
    let base_transform = robot_transform * Transform::from_xyz(0.0, 0.5 * BASE_HEIGHT, 0.0);
    let base = commands.spawn(ArmLink::Base)
        .insert(Transform::from(base_transform))
        .insert(Visibility::default())
        .insert(RigidBody::Fixed)
        .insert(Collider::cylinder(BASE_HEIGHT * 0.5, BASE_RADIUS))
        .insert(ColliderMassProperties::Mass(BASE_MASS))
        .set_parent(arm_root)
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
        .insert(CollisionGroups::new(Group::GROUP_1, Group::ALL))
        .insert(ImpulseJoint::new(base, TypedJoint::GenericJoint(base_link1_joint.build())))
        .insert(Draggable)
        .insert(DraggableBundle::default())
        .insert(OriginalTransform { transform: link1_transform })
        .insert(DragState::default())
        .set_parent(arm_root)
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
        .insert(CollisionGroups::new(Group::GROUP_1, Group::ALL))
        .insert(ImpulseJoint::new(link1, TypedJoint::GenericJoint(link1_link2_joint.build())))
        .insert(Draggable)
        .insert(DraggableBundle::default())
        .insert(OriginalTransform { transform: link2_transform })
        .insert(DragState::default())
        .set_parent(arm_root)
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
        .insert(CollisionGroups::new(Group::GROUP_1, Group::ALL))
        .insert(ImpulseJoint::new(link2, TypedJoint::GenericJoint(link2_link3_joint.build())))
        .insert(Draggable)
        .insert(DraggableBundle::default())
        .insert(OriginalTransform { transform: link3_transform })
        .insert(DragState::default())
        .set_parent(arm_root)
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
        .insert(CollisionGroups::new(Group::GROUP_1, Group::ALL))
        .insert(ImpulseJoint::new(link3, TypedJoint::GenericJoint(link3_link4_joint.build())))
        .insert(Draggable)
        .insert(DraggableBundle::default())
        .insert(OriginalTransform { transform: link4_transform })
        .insert(DragState::default())
        .set_parent(arm_root)
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
        .insert(CollisionGroups::new(Group::GROUP_1, Group::ALL))
        .insert(ImpulseJoint::new(link4, TypedJoint::GenericJoint(link4_link5_joint.build())))
        .insert(Draggable)
        .insert(DraggableBundle::default())
        .insert(OriginalTransform { transform: link5_transform })
        .insert(DragState::default())
        .set_parent(arm_root)
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
    let link6 = commands.spawn(ArmLink::Link6)
        .insert(link6_transform)
        .insert(Visibility::default())
        .insert(RigidBody::Dynamic)
        .insert(Collider::cylinder(0.5 * LINK6_HEIGHT, LINK6_RADIUS))
        .insert(ColliderMassProperties::Mass(LINK6_MASS))
        .insert(CollisionGroups::new(Group::GROUP_1, Group::ALL))
        .insert(ImpulseJoint::new(link5, TypedJoint::GenericJoint(link5_link6_joint.build())))
        .insert(Draggable)
        .insert(DraggableBundle::default())
        .insert(OriginalTransform { transform: link6_transform })
        .insert(DragState::default())
        .set_parent(arm_root)
        .with_children(|commands| {
            const LINK6_MESH_OFFSET: f32 = -0.5 * LINK6_HEIGHT;
            let transform = Transform::from_xyz(0.0, LINK6_MESH_OFFSET, 0.0)
                .with_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 0.0, 0.0));
            commands.spawn(SceneRoot(asset_server.load::<Scene>("robots/UR3e/link6.glb#Scene0")))
                .insert(transform);
        })
        .id();

    // Spawn the gripper attached to Link6
    spawn_simple_gripper(commands, link6, meshes, materials);

    // Spawn multiple pickup blocks around the robot
    spawn_pickup_blocks(commands, meshes, materials);
}

fn spawn_pickup_blocks(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Create a block mesh (10cm cube) - made bigger for visibility
    let block_mesh = meshes.add(Cuboid::new(0.10, 0.10, 0.10));

    // Create different colored materials for blocks
    let block_materials = [
        materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.2, 0.2), // Red
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 1.0, 0.2), // Green
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 1.0), // Blue
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 0.2), // Yellow
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.2, 1.0), // Magenta
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 1.0, 1.0), // Cyan
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.5, 0.2), // Orange
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.2, 1.0), // Purple
            ..default()
        }),
    ];

    // Spawn 8 blocks in a strategic pattern around the robot
    let positions = [
        Vec3::new(0.3, 0.02, 0.0),   // Front center
        Vec3::new(0.2, 0.02, 0.2),   // Front right
        Vec3::new(0.0, 0.02, 0.3),   // Right center
        Vec3::new(-0.2, 0.02, 0.2),  // Back right
        Vec3::new(-0.3, 0.02, 0.0),  // Back center
        Vec3::new(-0.2, 0.02, -0.2), // Back left
        Vec3::new(0.0, 0.02, -0.3),  // Left center
        Vec3::new(0.2, 0.02, -0.2),  // Front left
    ];

    for (i, position) in positions.iter().enumerate() {
        let material_index = i % block_materials.len();
        commands.spawn((
            Mesh3d(block_mesh.clone()),
            MeshMaterial3d(block_materials[material_index].clone()),
            Transform::from_translation(*position),
            RigidBody::Dynamic,
            Collider::cuboid(0.05, 0.05, 0.05), // 10cm cube
            ColliderMassProperties::Mass(0.5), // Increased mass for bigger blocks
            PickupBlock,
            CollisionGroups::new(Group::GROUP_2, Group::ALL),
        ));
    }
}

fn spawn_simple_gripper(
    commands: &mut Commands,
    parent_entity: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Simple gripper dimensions - made larger and more visible
    const GRIPPER_SIZE: Vec3 = Vec3::new(0.08, 0.06, 0.10);
    const FINGER_LENGTH: f32 = 0.08;
    const FINGER_WIDTH: f32 = 0.012;

    // Create meshes and materials
    let gripper_mesh = meshes.add(Cuboid::new(GRIPPER_SIZE.x, GRIPPER_SIZE.y, GRIPPER_SIZE.z));
    let finger_mesh = meshes.add(Cuboid::new(FINGER_WIDTH, FINGER_LENGTH, FINGER_WIDTH));

    let gripper_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.9), // Light blue - very visible
        metallic: 0.9,
        perceptual_roughness: 0.2,
        emissive: LinearRgba::new(0.05, 0.05, 0.1, 1.0), // Slight blue glow
        ..default()
    });

    let finger_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.9, 0.2), // Bright yellow - highly visible
        metallic: 0.8,
        perceptual_roughness: 0.1,
        emissive: LinearRgba::new(0.2, 0.2, 0.0, 1.0), // Strong yellow glow
        ..default()
    });

    // Spawn gripper as child of Link6
    commands.entity(parent_entity).with_children(|commands| {
        // Gripper base (purely visual, no physics)
        commands.spawn((
            ArmLink::GripperBase,
            SimpleGripper {
                is_open: false,
                grip_strength: 1.0,
            },
            Mesh3d(gripper_mesh),
            MeshMaterial3d(gripper_material),
            Transform::from_xyz(0.0, 0.0, 0.08), // Offset from Link6
            Visibility::default(),
            // Add sensor collider for detection only
            Collider::cuboid(GRIPPER_SIZE.x * 0.6, GRIPPER_SIZE.y * 0.6, GRIPPER_SIZE.z * 0.6),
            Sensor, // This makes it a sensor collider (no physics interactions)
            CollisionGroups::new(Group::GROUP_1, Group::ALL), // Same as Link6
        )).with_children(|commands| {
            // Left finger
            commands.spawn((
                ArmLink::GripperFinger1,
                Mesh3d(finger_mesh.clone()),
                MeshMaterial3d(finger_material.clone()),
                Transform::from_xyz(-0.01, FINGER_LENGTH * 0.5, 0.03),
                Visibility::default(),
            ));

            // Right finger
            commands.spawn((
                ArmLink::GripperFinger2,
                Mesh3d(finger_mesh),
                MeshMaterial3d(finger_material),
                Transform::from_xyz(0.01, FINGER_LENGTH * 0.5, 0.03),
                Visibility::default(),
            ));
        });
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

    // Ground plane - make it much larger to cover the entire workspace
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::default(),
        RigidBody::Fixed,
        Collider::cuboid(50.0, 0.1, 50.0),
        CollisionGroups::new(Group::GROUP_1, Group::ALL),
    ));

    // Spawn robotic arm
    spawn_ur3e_arm(&mut commands, &asset_server, Transform::IDENTITY, &mut meshes, &mut materials);

    // Add some blocks for the gripper to pick up
    spawn_pickup_blocks(&mut commands, &mut meshes, &mut materials);
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

pub fn simple_gripper_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut gripper_query: Query<(&mut SimpleGripper, Entity, &Children, &GlobalTransform), With<SimpleGripper>>,
    mut collision_events: EventReader<CollisionEvent>,
    block_query: Query<(Entity, &Transform), (With<PickupBlock>, Without<GrippedObject>)>,
    gripped_query: Query<(Entity, &mut GrippedObject)>,
) {
    // Handle collision-based picking - automatically pick up blocks when they collide with gripper
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            // Check if one entity is the gripper and the other is a pickup block
            for (gripper, gripper_entity, _, _) in gripper_query.iter() {
                let colliding_block = if *entity1 == gripper_entity {
                    // entity2 might be a block
                    block_query.get(*entity2).ok().map(|(block_entity, _)| block_entity)
                } else if *entity2 == gripper_entity {
                    // entity1 might be a block
                    block_query.get(*entity1).ok().map(|(block_entity, _)| block_entity)
                } else {
                    None
                };

                if let Some(block_entity) = colliding_block {
                    if !gripper.is_open {
                        // Pick up the block automatically when gripper is closed
                        pick_up_block(&mut commands, gripper_entity, block_entity);
                        break; // Only pick up one block per collision event
                    }
                }
            }
        }
    }

    // Handle manual gripper control and block release
    for (mut gripper, gripper_entity, _children, gripper_global_transform) in gripper_query.iter_mut() {
        let was_open = gripper.is_open;

        // G key to manually toggle gripper open/close
        if keyboard_input.just_pressed(KeyCode::KeyG) {
            gripper.is_open = !gripper.is_open;

            // When gripper closes, try to pick up nearby blocks
            if was_open && !gripper.is_open {
                // Use the gripper's world position as the pickup point
                let gripper_position = gripper_global_transform.translation();
                if let Some(nearest_block) = find_nearest_block_in_range(&Transform::from_translation(gripper_position), &block_query, 0.15) {
                    pick_up_block(&mut commands, gripper_entity, nearest_block);
                }
            }
        }

        // P key to manually pick up nearest block (distance-based fallback)
        if keyboard_input.just_pressed(KeyCode::KeyP) {
            let gripper_position = gripper_global_transform.translation();
            if let Some(nearest_block) = find_nearest_block_in_range(&Transform::from_translation(gripper_position), &block_query, 0.15) {
                pick_up_block(&mut commands, gripper_entity, nearest_block);
                gripper.is_open = false; // Close gripper after picking up
            }
        }

        // R key to release gripped blocks and open gripper
        if keyboard_input.just_pressed(KeyCode::KeyR) && !gripped_query.is_empty() {
            gripper.is_open = true; // Open gripper when releasing
            release_gripped_blocks(&mut commands, &gripped_query);
        }
    }
}

fn find_nearest_block_in_range(
    gripper_transform: &Transform,
    block_query: &Query<(Entity, &Transform), (With<PickupBlock>, Without<GrippedObject>)>,
    max_distance: f32,
) -> Option<Entity> {
    let mut nearest_block: Option<Entity> = None;
    let mut nearest_distance = max_distance;

    for (block_entity, block_transform) in block_query.iter() {
        let distance = gripper_transform.translation.distance(block_transform.translation);
        if distance <= nearest_distance {
            nearest_distance = distance;
            nearest_block = Some(block_entity);
        }
    }

    nearest_block
}

fn pick_up_block(commands: &mut Commands, _gripper_entity: Entity, block_entity: Entity) {
    // Add GrippedObject component to mark this block as gripped
    commands.entity(block_entity).insert(GrippedObject {
        original_parent: None,
    });

    // Remove physics from the block while it's gripped
    commands.entity(block_entity).remove::<RigidBody>();
    commands.entity(block_entity).remove::<Collider>();
}

fn release_gripped_blocks(commands: &mut Commands, gripped_query: &Query<(Entity, &mut GrippedObject)>) {
    for (gripped_entity, _gripped_object) in gripped_query.iter() {
        // Remove GrippedObject component
        commands.entity(gripped_entity).remove::<GrippedObject>();

        // Re-add physics to the block
        commands.entity(gripped_entity).insert(RigidBody::Dynamic);
        commands.entity(gripped_entity).insert(Collider::cuboid(0.02, 0.02, 0.02)); // 4cm cube
        commands.entity(gripped_entity).insert(ColliderMassProperties::Mass(0.1));
    }
}

fn animate_gripper_fingers(
    children: &Children,
    finger_query: &mut Query<&mut Transform, (With<ArmLink>, Without<Draggable>, Without<GrippedObject>)>,
    is_open: bool,
) {
    for child in children.iter() {
        if let Ok(mut transform) = finger_query.get_mut(child) {
            // More obvious finger animation - move fingers apart when open
            let finger_separation = if is_open { 0.05 } else { 0.01 };

            // Determine if this is left or right finger based on current x position
            if transform.translation.x < 0.0 {
                // Left finger
                transform.translation.x = -finger_separation;
            } else if transform.translation.x > 0.0 {
                // Right finger
                transform.translation.x = finger_separation;
            }
        }
    }
}

pub fn detect_drag_state(
    mut query: Query<(&mut DragState, &Transform, &OriginalTransform), With<Draggable>>,
    _time: Res<Time>,
) {
    const POSITION_THRESHOLD: f32 = 0.001; // Threshold to consider an object "stationary"
    const ROTATION_THRESHOLD: f32 = 0.001; // Threshold for rotation changes

    for (mut drag_state, current_transform, original_transform) in query.iter_mut() {
        // Check if position or rotation has changed significantly from original
        let position_diff = current_transform.translation.distance(original_transform.transform.translation);
        let rotation_diff = current_transform.rotation.angle_between(original_transform.transform.rotation);

        let is_displaced = position_diff > POSITION_THRESHOLD || rotation_diff > ROTATION_THRESHOLD;

        if is_displaced {
            drag_state.was_dragged = true;
            drag_state.is_being_dragged = true;
        } else if drag_state.was_dragged {
            // Object was dragged but is now stationary - start return animation
            drag_state.is_being_dragged = false;
            drag_state.return_timer = 0.0;
        }
    }
}

pub fn return_to_original_position(
    mut query: Query<(&mut Transform, &mut DragState, &OriginalTransform), With<Draggable>>,
    time: Res<Time>,
) {
    for (mut transform, mut drag_state, original_transform) in query.iter_mut() {
        // Force return if object falls below ground level (emergency recovery)
        let fell_below_ground = transform.translation.y < -0.1; // Below ground threshold

        // Only animate return if object was dragged and is not currently being dragged, or if it fell below ground
        if (drag_state.was_dragged && !drag_state.is_being_dragged) || fell_below_ground {
            drag_state.return_timer += time.delta_secs();

            // Calculate interpolation factor (0 to 1 over return_duration seconds)
            let t = (drag_state.return_timer / drag_state.return_duration).min(1.0);

            // Smooth interpolation using ease-out function
            let ease_t = 1.0 - (1.0 - t).powi(3);

            // Interpolate position
            transform.translation = original_transform.transform.translation.lerp(
                transform.translation,
                1.0 - ease_t
            );

            // Interpolate rotation
            transform.rotation = original_transform.transform.rotation.slerp(
                transform.rotation,
                1.0 - ease_t
            );

            // Reset when animation is complete
            if t >= 1.0 {
                // Ensure exact final position
                transform.translation = original_transform.transform.translation;
                transform.rotation = original_transform.transform.rotation;

                drag_state.was_dragged = false;
                drag_state.return_timer = 0.0;
            }
        }
    }
}

pub fn update_gripped_objects(
    gripper_query: Query<&GlobalTransform, With<SimpleGripper>>,
    mut gripped_query: Query<&mut Transform, (With<GrippedObject>, Without<SimpleGripper>)>,
) {
    // Make gripped objects follow the gripper
    for gripper_global_transform in gripper_query.iter() {
        for mut block_transform in gripped_query.iter_mut() {
            // Position block slightly in front of gripper
            let offset = gripper_global_transform.forward() * 0.02 + gripper_global_transform.up() * 0.01;
            block_transform.translation = gripper_global_transform.translation() + offset;
            block_transform.rotation = gripper_global_transform.rotation();
        }
    }
}

pub fn animate_gripper_fingers_system(
    gripper_query: Query<(&SimpleGripper, &Children), With<SimpleGripper>>,
    mut finger_query: Query<&mut Transform, (With<ArmLink>, Without<Draggable>, Without<GrippedObject>)>,
) {
    for (gripper, children) in gripper_query.iter() {
        animate_gripper_fingers(children, &mut finger_query, gripper.is_open);
    }
}
