use bevy::asset::AssetServer;
use bevy::ecs::{
    bundle::Bundle, component::Component, entity::Entity, system::{Commands, Query, Res}
};
use bevy::hierarchy::BuildChildren;
use bevy::math::{Quat, Vec3};
use bevy::render::prelude::SpatialBundle;
use bevy::scene::Scene;
use bevy::transform::components::Transform;
use bevy_rapier3d::{
    dynamics::{GenericJoint, GenericJointBuilder, ImpulseJoint, JointAxesMask, RigidBody, Sleeping},
    geometry::{Collider, ColliderMassProperties, CollisionGroups},
};

use crate::drag::DraggableBundle;

const CHASSIS_RADIUS: f32 = 0.175;
const CHASSIS_HEIGHT: f32 = 0.340;
const CHASSIS_HEIGHT_OFFSET: f32 = 0.009;
const CHASSIS_MASS: f32 = 1.0;
const WHEEL_RADIUS: f32 = 0.036;
const WHEEL_WIDTH: f32 = 0.018;
const WHEEL_OFFSET_X: f32 = 0.0;
const WHEEL_OFFSET_Z: f32 = 0.1185;
const WHEEL_MASS: f32 = 0.1;

pub fn velocity_control(
    mut motors: Query<(&Wheel, &mut ImpulseJoint, &mut Sleeping)>,
) {
    for (wheel, mut joint, mut sleeping) in motors.iter_mut() {
        sleeping.sleeping = false;
        match wheel {
            Wheel::Left => joint.data.as_revolute_mut()
                .unwrap()
                .set_motor_velocity(5.0, 100.0),
            Wheel::Right => joint.data.as_revolute_mut()
                .unwrap()
                .set_motor_velocity(5.0, 100.0),
        };
    }
}

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
}

impl Default for ChassisPhysicsBundle {
    fn default() -> ChassisPhysicsBundle {
        ChassisPhysicsBundle {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cylinder(0.5 * CHASSIS_HEIGHT, CHASSIS_RADIUS),
            collision_groups: CollisionGroups::new(
                crate::CHASSIS_GROUP,
                crate::STATIC_GROUP | crate::CHASSIS_GROUP
            ),
            collider_mass_properties: ColliderMassProperties::Mass(CHASSIS_MASS),
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
    sleeping: Sleeping
}

impl WheelPhysicsBundle {
    fn new(chassis: Entity, joint: impl Into<GenericJoint>) -> WheelPhysicsBundle {
        WheelPhysicsBundle {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cylinder(WHEEL_WIDTH * 0.5, WHEEL_RADIUS),
            collision_groups: CollisionGroups::new(
                crate::CHASSIS_INTERNAL_GROUP,
                crate::STATIC_GROUP
            ),
            collider_mass_properties: ColliderMassProperties::Mass(WHEEL_MASS),
            joint: ImpulseJoint::new(chassis, joint.into()),
            sleeping: Default::default(),
        }
    }
}

pub fn spawn(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    transform: &Transform,
) {
    commands.spawn(SpatialBundle::default())
        .with_children(|commands| {
            /* spawn the chassis */
            let chassis_transform = *transform
                * Transform::from_xyz(0.0, 0.5 * CHASSIS_HEIGHT + CHASSIS_HEIGHT_OFFSET, 0.0);
            let chassis = commands.spawn_empty()
                .insert(SpatialBundle::from_transform(chassis_transform))
                .insert(ChassisPhysicsBundle::default())
                .insert(DraggableBundle::default())
                .insert(asset_server.load::<Scene>("robots/turtlebot4.glb#Scene0"))
                .id();
            /* spawn the left wheel */
            let left_wheel_transform = *transform *
                Transform::from_xyz(WHEEL_OFFSET_X, WHEEL_RADIUS, -WHEEL_OFFSET_Z)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
            let left_wheel_joint = GenericJointBuilder::new(JointAxesMask::LOCKED_REVOLUTE_AXES)
                .local_axis1(Vec3::NEG_Z)
                .local_axis2(Vec3::Y)
                .local_anchor1(Vec3::new(0.0, -0.5 * CHASSIS_HEIGHT - CHASSIS_HEIGHT_OFFSET + WHEEL_RADIUS, -WHEEL_OFFSET_Z)) // base
                .local_anchor2(Vec3::new(0.0, 0.0, 0.0));
            commands.spawn(Wheel::Left)
                .insert(SpatialBundle::from_transform(left_wheel_transform))
                .insert(WheelPhysicsBundle::new(chassis, left_wheel_joint));
            /* spawn the right wheel */
            let right_wheel_transform = *transform *
                Transform::from_xyz(WHEEL_OFFSET_X, WHEEL_RADIUS, WHEEL_OFFSET_Z)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
            let right_wheel_joint = GenericJointBuilder::new(JointAxesMask::LOCKED_REVOLUTE_AXES)
                .local_axis1(Vec3::Z)
                .local_axis2(Vec3::Y)
                .local_anchor1(Vec3::new(0.0, -0.5 * CHASSIS_HEIGHT - CHASSIS_HEIGHT_OFFSET + WHEEL_RADIUS, WHEEL_OFFSET_Z)) // base
                .local_anchor2(Vec3::new(0.0, 0.0, 0.0));
            commands.spawn(Wheel::Right)
                .insert(SpatialBundle::from_transform(right_wheel_transform))
                .insert(WheelPhysicsBundle::new(chassis, right_wheel_joint));
        });
}