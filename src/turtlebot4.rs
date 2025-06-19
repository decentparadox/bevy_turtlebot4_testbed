use bevy::asset::AssetServer;
use bevy::ecs::{
    component::Component, entity::Entity, system::{Commands, Query, Res}
};
use bevy::hierarchy::BuildChildren;
use bevy::math::{Quat, Vec3};
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

pub fn spawn(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    transform: &Transform,
) {
    commands.spawn(Transform::IDENTITY)
        .with_children(|commands| {
            /* spawn the chassis */
            let chassis_transform = *transform
                * Transform::from_xyz(0.0, 0.5 * CHASSIS_HEIGHT + CHASSIS_HEIGHT_OFFSET, 0.0);
            let chassis = commands.spawn((
                Transform::from_matrix(chassis_transform.compute_matrix()),
                RigidBody::Dynamic,
                Collider::cylinder(0.5 * CHASSIS_HEIGHT, CHASSIS_RADIUS),
                CollisionGroups::new(
                    crate::CHASSIS_GROUP,
                    crate::STATIC_GROUP | crate::CHASSIS_GROUP
                ),
                ColliderMassProperties::Mass(CHASSIS_MASS),
                asset_server.load::<bevy::scene::Scene>("robots/turtlebot4.glb#Scene0"),
            )).id();
            
            /* spawn the left wheel */
            let left_wheel_transform = *transform *
                Transform::from_xyz(WHEEL_OFFSET_X, WHEEL_RADIUS, -WHEEL_OFFSET_Z)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
            let left_wheel_joint = GenericJointBuilder::new(JointAxesMask::LOCKED_REVOLUTE_AXES)
                .local_axis1(Vec3::NEG_Z)
                .local_axis2(Vec3::Y)
                .local_anchor1(Vec3::new(0.0, -0.5 * CHASSIS_HEIGHT - CHASSIS_HEIGHT_OFFSET + WHEEL_RADIUS, -WHEEL_OFFSET_Z)) // base
                .local_anchor2(Vec3::new(0.0, 0.0, 0.0))
                .build();
                
            commands.spawn((
                Wheel::Left,
                Transform::from_matrix(left_wheel_transform.compute_matrix()),
                RigidBody::Dynamic,
                Collider::cylinder(WHEEL_WIDTH * 0.5, WHEEL_RADIUS),
                CollisionGroups::new(
                    crate::CHASSIS_INTERNAL_GROUP,
                    crate::STATIC_GROUP
                ),
                ColliderMassProperties::Mass(WHEEL_MASS),
                ImpulseJoint::new(chassis, left_wheel_joint),
                Sleeping::default(),
            ));
            
            /* spawn the right wheel */
            let right_wheel_transform = *transform *
                Transform::from_xyz(WHEEL_OFFSET_X, WHEEL_RADIUS, WHEEL_OFFSET_Z)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
            let right_wheel_joint = GenericJointBuilder::new(JointAxesMask::LOCKED_REVOLUTE_AXES)
                .local_axis1(Vec3::Z)
                .local_axis2(Vec3::Y)
                .local_anchor1(Vec3::new(0.0, -0.5 * CHASSIS_HEIGHT - CHASSIS_HEIGHT_OFFSET + WHEEL_RADIUS, WHEEL_OFFSET_Z)) // base
                .local_anchor2(Vec3::new(0.0, 0.0, 0.0))
                .build();
                
            commands.spawn((
                Wheel::Right,
                Transform::from_matrix(right_wheel_transform.compute_matrix()),
                RigidBody::Dynamic,
                Collider::cylinder(WHEEL_WIDTH * 0.5, WHEEL_RADIUS),
                CollisionGroups::new(
                    crate::CHASSIS_INTERNAL_GROUP,
                    crate::STATIC_GROUP
                ),
                ColliderMassProperties::Mass(WHEEL_MASS),
                ImpulseJoint::new(chassis, right_wheel_joint),
                Sleeping::default(),
            ));
        });
}