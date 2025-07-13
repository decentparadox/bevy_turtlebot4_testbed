use bevy::asset::AssetServer;
use bevy::ecs::{
    bundle::Bundle,
    component::Component,
    entity::Entity,
    system::{Commands, Res},
};
use bevy::math::{Quat, Vec3};
use bevy::prelude::*;
use bevy::scene::Scene;
use bevy::transform::components::Transform;
use bevy_rapier3d::{
    dynamics::{ExternalImpulse, ImpulseJoint, RevoluteJoint, RigidBody, Sleeping, Velocity},
    geometry::{Collider, ColliderMassProperties, CollisionGroups},
};

const CHASSIS_RADIUS: f32 = 0.175;
const CHASSIS_HEIGHT: f32 = 0.340;
const CHASSIS_HEIGHT_OFFSET: f32 = 0.009;
const CHASSIS_MASS: f32 = 1.0;
const WHEEL_RADIUS: f32 = 0.036;
const WHEEL_WIDTH: f32 = 0.018;
const WHEEL_OFFSET_X: f32 = 0.0;
const WHEEL_OFFSET_Z: f32 = 0.1185;
const WHEEL_MASS: f32 = 0.1;

#[derive(Component)]
pub enum Wheel {
    Left,
    Right,
}

#[derive(Bundle)]
struct ChassisPhysicsBundle {
    rigid_body: RigidBody,
    collider: Collider,
    collision_groups: CollisionGroups,
    collider_mass_properties: ColliderMassProperties,
    velocity: Velocity,
}

impl Default for ChassisPhysicsBundle {
    fn default() -> ChassisPhysicsBundle {
        ChassisPhysicsBundle {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cylinder(0.5 * CHASSIS_HEIGHT, CHASSIS_RADIUS),
            collision_groups: CollisionGroups::new(
                crate::CHASSIS_GROUP,
                crate::STATIC_GROUP | crate::CHASSIS_GROUP,
            ),
            collider_mass_properties: ColliderMassProperties::Mass(CHASSIS_MASS),
            velocity: Velocity::default(),
        }
    }
}

#[derive(Bundle)]
struct WheelPhysicsBundle {
    rigid_body: RigidBody,
    collider: Collider,
    collision_groups: CollisionGroups,
    collider_mass_properties: ColliderMassProperties,
    joint: ImpulseJoint,
    sleeping: Sleeping,
}

impl WheelPhysicsBundle {
    fn new(chassis: Entity, axis: Vec3, _anchor1: Vec3, _anchor2: Vec3) -> WheelPhysicsBundle {
        // Create a RevoluteJoint with the axis and configure anchors via builder pattern
        let revolute_joint = RevoluteJoint::new(axis);

        WheelPhysicsBundle {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cylinder(WHEEL_WIDTH * 0.5, WHEEL_RADIUS),
            collision_groups: CollisionGroups::new(
                crate::CHASSIS_INTERNAL_GROUP,
                crate::STATIC_GROUP,
            ),
            collider_mass_properties: ColliderMassProperties::Mass(WHEEL_MASS),
            joint: ImpulseJoint::new(chassis, revolute_joint),
            sleeping: Default::default(),
        }
    }
}

pub fn spawn(commands: &mut Commands, asset_server: &Res<AssetServer>, transform: &Transform) {
    commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .with_children(|commands| {
            /* spawn the chassis */
            let chassis_transform = *transform
                * Transform::from_xyz(0.0, 0.5 * CHASSIS_HEIGHT + CHASSIS_HEIGHT_OFFSET, 0.0);
            let chassis = commands
                .spawn_empty()
                .insert((
                    chassis_transform,
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .insert(ChassisPhysicsBundle::default())
                .insert(ExternalImpulse::default()) // For applying movement forces
                .insert(crate::RobotChassis) // Marker component for robot control
                .insert(SceneRoot(
                    asset_server.load::<Scene>("robots/turtlebot4.glb#Scene0"),
                ))
                .id();

            /* spawn the LIDAR sensor on top of chassis */
            commands
                .spawn((
                    crate::lidar::LidarSensor::default(),
                    Transform::from_translation(Vec3::new(0.0, 0.15, 0.0)), // Mount on top
                    GlobalTransform::default(),
                    Visibility::default(),
                ))
                .insert(ChildOf(chassis));
            /* spawn the left wheel */
            let left_wheel_transform = *transform
                * Transform::from_xyz(WHEEL_OFFSET_X, WHEEL_RADIUS, -WHEEL_OFFSET_Z)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
            let left_wheel_anchor1 = Vec3::new(
                0.0,
                -0.5 * CHASSIS_HEIGHT - CHASSIS_HEIGHT_OFFSET + WHEEL_RADIUS,
                -WHEEL_OFFSET_Z,
            );
            let left_wheel_anchor2 = Vec3::new(0.0, 0.0, 0.0);
            commands
                .spawn(Wheel::Left)
                .insert((
                    left_wheel_transform,
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .insert(WheelPhysicsBundle::new(
                    chassis,
                    Vec3::Y,
                    left_wheel_anchor1,
                    left_wheel_anchor2,
                ));
            /* spawn the right wheel */
            let right_wheel_transform = *transform
                * Transform::from_xyz(WHEEL_OFFSET_X, WHEEL_RADIUS, WHEEL_OFFSET_Z)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
            let right_wheel_anchor1 = Vec3::new(
                0.0,
                -0.5 * CHASSIS_HEIGHT - CHASSIS_HEIGHT_OFFSET + WHEEL_RADIUS,
                WHEEL_OFFSET_Z,
            );
            let right_wheel_anchor2 = Vec3::new(0.0, 0.0, 0.0);
            commands
                .spawn(Wheel::Right)
                .insert((
                    right_wheel_transform,
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .insert(WheelPhysicsBundle::new(
                    chassis,
                    Vec3::Y,
                    right_wheel_anchor1,
                    right_wheel_anchor2,
                ));
        });
}
