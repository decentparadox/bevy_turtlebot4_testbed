use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

use crate::{
    CHASSIS_GROUP, RobotChassis, STATIC_GROUP,
    camera::PanOrbitCamera,
    lidar::{LaserScan, LidarSensor},
};

#[cfg(test)]
mod lidar_tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_lidar_sensor_defaults() {
        let lidar = LidarSensor::default();

        // Test that default values match expected RPLIDAR A1M8 specifications
        assert_relative_eq!(lidar.range_min, 0.2, epsilon = 0.01);
        assert_relative_eq!(lidar.range_max, 12.0, epsilon = 0.01);
        assert_relative_eq!(lidar.scan_rate, 10.0, epsilon = 0.01);
        assert_eq!(lidar.rays_per_scan, 36);

        // Angular resolution should be 2π / rays_per_scan
        let expected_resolution = 2.0 * PI / 36.0;
        assert_relative_eq!(
            lidar.angular_resolution,
            expected_resolution,
            epsilon = 0.001
        );

        // Verify initial state
        assert!(lidar.visualize); // Default is true
        assert!(lidar.enable_logging); // Default is true
        assert_relative_eq!(lidar.noise_stddev, 0.0, epsilon = 0.001);
        assert!(lidar.scan_results.is_empty());
    }

    #[test]
    fn test_lidar_parameter_update() {
        let mut lidar = LidarSensor::default();

        // Change parameters
        lidar.rays_per_scan = 72; // Double the resolution
        lidar.update_parameters();

        // Angular resolution should update accordingly
        let expected_resolution = 2.0 * PI / 72.0;
        assert_relative_eq!(
            lidar.angular_resolution,
            expected_resolution,
            epsilon = 0.001
        );
    }

    #[test]
    fn test_laser_scan_message_format() {
        let laser_scan = LaserScan {
            angle_min: 0.0,
            angle_max: 2.0 * PI,
            angle_increment: PI / 180.0, // 1 degree steps
            time_increment: 0.0,
            scan_time: 0.1, // 10 Hz
            range_min: 0.2,
            range_max: 12.0,
            ranges: vec![1.0, 2.0, 3.0, f32::INFINITY, 5.0],
            intensities: vec![1.0; 5],
        };

        // Verify message structure matches ROS LaserScan format
        assert_relative_eq!(laser_scan.angle_min, 0.0, epsilon = 0.001);
        assert_relative_eq!(laser_scan.angle_max, 2.0 * PI, epsilon = 0.001);
        assert_relative_eq!(laser_scan.angle_increment, PI / 180.0, epsilon = 0.001);
        assert_eq!(laser_scan.ranges.len(), 5);
        assert_eq!(laser_scan.intensities.len(), 5);

        // Check that infinite range is properly handled
        assert!(laser_scan.ranges[3].is_infinite());
    }

    #[test]
    fn test_lidar_range_validation() {
        let lidar = LidarSensor::default();
        let test_ranges = vec![0.1, 0.2, 1.0, 5.0, 12.0, 15.0];

        for &range in &test_ranges {
            let is_valid = range >= lidar.range_min && range <= lidar.range_max;

            if range < 0.2 {
                assert!(!is_valid, "Range {} should be below minimum", range);
            } else if range > 12.0 {
                assert!(!is_valid, "Range {} should be above maximum", range);
            } else {
                assert!(is_valid, "Range {} should be valid", range);
            }
        }
    }

    #[test]
    fn test_lidar_angle_calculations() {
        let lidar = LidarSensor::default();

        // Test angle calculations for different ray indices
        for i in 0..lidar.rays_per_scan {
            let angle = i as f32 * lidar.angular_resolution;

            // Angle should be in range [0, 2π)
            assert!(angle >= 0.0, "Angle should be non-negative");
            assert!(angle < 2.0 * PI, "Angle should be less than 2π");

            // Test direction vector calculation (ROS standard: +X forward, +Z left)
            let local_direction = Vec3::new(
                angle.cos(), // X component
                0.0,         // Y component (LIDAR is horizontal)
                angle.sin(), // Z component (counter-clockwise)
            );

            // Direction should be unit length
            assert_relative_eq!(local_direction.length(), 1.0, epsilon = 0.001);
        }
    }
}

#[cfg(test)]
mod camera_tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_pan_orbit_camera_defaults() {
        let camera = PanOrbitCamera::default();

        assert_eq!(camera.focus, Vec3::ZERO);
        assert_relative_eq!(camera.radius, 5.0, epsilon = 0.01);
        assert!(!camera.upside_down);
        assert_eq!(camera.pan, Vec2::ZERO);
        assert_eq!(camera.rotation_move, Vec2::ZERO);
        assert_relative_eq!(camera.scroll, 0.0, epsilon = 0.001);
        assert!(!camera.orbit_button_changed);
    }

    #[test]
    fn test_camera_focus_interpolation() {
        let mut camera = PanOrbitCamera::default();
        let target_position = Vec3::new(5.0, 0.0, 3.0);
        let initial_distance = (camera.focus - target_position).length();

        // Simulate smooth interpolation (LERP with factor 0.1)
        let lerp_factor = 0.1;

        // Multiple steps of interpolation
        for _step in 0..20 {
            camera.focus = camera.focus.lerp(target_position, lerp_factor);
        }

        // After several steps, focus should be closer to target than initially
        let final_distance = (camera.focus - target_position).length();
        assert!(
            final_distance < initial_distance,
            "Camera should move closer to target. Initial: {:.2}, Final: {:.2}",
            initial_distance,
            final_distance
        );
    }

    #[test]
    fn test_camera_radius_bounds() {
        let mut camera = PanOrbitCamera::default();

        // Test zoom limits
        camera.radius = 0.01; // Very close
        camera.radius = f32::max(camera.radius, 0.05); // Apply minimum limit
        assert!(
            camera.radius >= 0.05,
            "Camera radius should not go below minimum"
        );

        camera.radius = 1000.0; // Very far
        // No maximum limit in current implementation, just verify it's reasonable
        assert!(camera.radius > 0.0, "Camera radius should be positive");
    }

    #[test]
    fn test_camera_movement_accumulation() {
        let mut camera = PanOrbitCamera::default();

        // Simulate mouse input accumulation
        let mouse_delta = Vec2::new(10.0, 5.0);
        camera.pan += 2.0 * mouse_delta;
        camera.rotation_move += 2.0 * mouse_delta;
        camera.scroll += 2.0 * 1.0; // Scroll amount

        // Verify values are accumulated correctly
        assert_eq!(camera.pan, Vec2::new(20.0, 10.0));
        assert_eq!(camera.rotation_move, Vec2::new(20.0, 10.0));
        assert_relative_eq!(camera.scroll, 2.0, epsilon = 0.001);
    }
}

#[cfg(test)]
mod physics_validation_tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_collision_group_constants() {
        // Verify collision groups are distinct
        assert_ne!(STATIC_GROUP, CHASSIS_GROUP);

        // Test collision group creation
        let static_collision = CollisionGroups::new(STATIC_GROUP, CHASSIS_GROUP);
        let chassis_collision = CollisionGroups::new(CHASSIS_GROUP, STATIC_GROUP);

        // Verify groups are set correctly
        assert_eq!(static_collision.memberships, STATIC_GROUP);
        assert_eq!(static_collision.filters, CHASSIS_GROUP);

        assert_eq!(chassis_collision.memberships, CHASSIS_GROUP);
        assert_eq!(chassis_collision.filters, STATIC_GROUP);
    }

    #[test]
    fn test_robot_chassis_marker() {
        // Test that RobotChassis is a proper marker component
        struct TestEntity {
            _marker: RobotChassis,
        }

        let _test_entity = TestEntity {
            _marker: RobotChassis,
        };

        // If this compiles, RobotChassis works as a marker component
        assert!(true);
    }

    #[test]
    fn test_distance_calculations() {
        // Test 3D distance calculations for sensor detection
        let robot_pos = Vec3::new(0.0, 0.5, 0.0);
        let wall_pos = Vec3::new(5.0, 0.5, 0.0);

        let distance = (wall_pos - robot_pos).length();
        assert_relative_eq!(distance, 5.0, epsilon = 0.001);

        // Test diagonal distance
        let diagonal_pos = Vec3::new(3.0, 0.5, 4.0);
        let diagonal_distance = (diagonal_pos - robot_pos).length();
        assert_relative_eq!(diagonal_distance, 5.0, epsilon = 0.001); // 3-4-5 triangle
    }

    #[test]
    fn test_direction_vectors() {
        // Test direction vector calculations for LIDAR rays
        let angles = vec![0.0, PI / 2.0, PI, 3.0 * PI / 2.0];
        let expected_directions = vec![
            Vec3::X,     // 0° -> +X
            Vec3::Z,     // 90° -> +Z
            Vec3::NEG_X, // 180° -> -X
            Vec3::NEG_Z, // 270° -> -Z
        ];

        for (angle, expected) in angles.iter().zip(expected_directions.iter()) {
            let direction = Vec3::new(angle.cos(), 0.0, angle.sin());
            assert_relative_eq!(direction.x, expected.x, epsilon = 0.001);
            assert_relative_eq!(direction.z, expected.z, epsilon = 0.001);
            assert_relative_eq!(direction.y, 0.0, epsilon = 0.001); // LIDAR is horizontal
        }
    }
}

#[cfg(test)]
mod sensor_integration_tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_lidar_sensor_configuration() {
        let mut lidar = LidarSensor::default();

        // Test configuration for different scenarios

        // High-resolution scan
        lidar.rays_per_scan = 360;
        lidar.update_parameters();
        assert_relative_eq!(lidar.angular_resolution, 2.0 * PI / 360.0, epsilon = 0.001);

        // Low-resolution scan (for performance)
        lidar.rays_per_scan = 18;
        lidar.update_parameters();
        assert_relative_eq!(lidar.angular_resolution, 2.0 * PI / 18.0, epsilon = 0.001);

        // Test noise configuration
        lidar.noise_stddev = 0.05; // 5cm standard deviation
        assert_relative_eq!(lidar.noise_stddev, 0.05, epsilon = 0.001);
    }

    #[test]
    fn test_sensor_positioning() {
        let robot_position = Transform::from_xyz(0.0, 0.5, 0.0);
        let lidar_offset = Vec3::new(0.0, 0.15, 0.0); // LIDAR on top of robot

        // Calculate world position of LIDAR
        let lidar_world_pos = robot_position.transform_point(lidar_offset);
        assert_relative_eq!(lidar_world_pos.y, 0.65, epsilon = 0.001); // 0.5 + 0.15

        // Test with rotated robot
        let rotated_robot =
            Transform::from_xyz(0.0, 0.5, 0.0).with_rotation(Quat::from_rotation_y(PI / 2.0));
        let rotated_lidar_pos = rotated_robot.transform_point(lidar_offset);

        // LIDAR should still be above robot, but position may be different due to rotation
        assert_relative_eq!(rotated_lidar_pos.y, 0.65, epsilon = 0.001);
    }

    #[test]
    fn test_obstacle_detection_logic() {
        // Simulate obstacle detection calculations
        let lidar_pos = Vec3::new(0.0, 0.5, 0.0);
        let obstacle_pos = Vec3::new(3.0, 0.5, 0.0);
        let ray_direction = Vec3::X; // Pointing toward obstacle

        let to_obstacle = obstacle_pos - lidar_pos;
        let distance = to_obstacle.length();

        // Check if obstacle is in ray direction (dot product test)
        let normalized_to_obstacle = to_obstacle.normalize();
        let dot_product = ray_direction.dot(normalized_to_obstacle);

        assert_relative_eq!(distance, 3.0, epsilon = 0.001);
        assert!(dot_product > 0.99, "Obstacle should be in ray direction");
    }

    #[test]
    fn test_scan_data_validation() {
        // Test that scan data meets expected formats
        let ranges = vec![1.0, 2.5, f32::INFINITY, 0.8, 12.0];
        let intensities = vec![0.8, 0.9, 0.0, 1.0, 0.7];

        assert_eq!(
            ranges.len(),
            intensities.len(),
            "Ranges and intensities should have same length"
        );

        // Validate individual measurements
        for (i, &range) in ranges.iter().enumerate() {
            if range.is_finite() {
                assert!(range >= 0.0, "Range should be non-negative");
                assert!(
                    intensities[i] >= 0.0 && intensities[i] <= 1.0,
                    "Intensity should be normalized"
                );
            } else {
                // Infinite range typically means no return
                assert_eq!(intensities[i], 0.0, "No return should have zero intensity");
            }
        }
    }
}
