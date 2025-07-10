use bevy::prelude::*;
use bevy_rapier3d::dynamics::ExternalImpulse;
use bevy::render::camera::Projection;

use crate::{RobotChassis, ObliqueProjectionController, ObliquePerspectiveProjection, camera::PanOrbitCamera, lidar::LidarSensor};

/// System to control robot movement with camera-relative controls
pub fn control_robot_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut robot_query: Query<&mut ExternalImpulse, With<RobotChassis>>,
    camera_query: Query<&Transform, (With<PanOrbitCamera>, Without<RobotChassis>)>,
) {
    if let Ok(mut impulse) = robot_query.single_mut() {
        let mut movement = Vec3::ZERO;
        let mut rotation = Vec3::ZERO;
        let force_multiplier = 0.5;
        let rotation_multiplier = 0.03;
        
        // Get camera transform for relative movement
        if let Ok(camera_transform) = camera_query.single() {
            // Get camera's right and forward vectors (projected onto XZ plane for ground movement)
            let camera_forward = camera_transform.forward();
            let camera_right = camera_transform.right();
            
            // Project vectors onto XZ plane and normalize for ground-based movement
            let camera_forward_xz = Vec3::new(camera_forward.x, 0.0, camera_forward.z).normalize();
            let camera_right_xz = Vec3::new(camera_right.x, 0.0, camera_right.z).normalize();
            
            // Camera-relative movement controls
            if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
                movement += camera_forward_xz * force_multiplier; // Forward relative to camera
            }
            if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
                movement -= camera_forward_xz * force_multiplier; // Backward relative to camera
            }
            if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
                movement -= camera_right_xz * force_multiplier; // Left relative to camera
            }
            if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
                movement += camera_right_xz * force_multiplier; // Right relative to camera
            }
        } else {
            // Fallback to world-relative movement if camera not found
            warn!("Camera not found, using world-relative movement");
            
            if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
                movement.z -= force_multiplier; // Forward in world coordinates
            }
            if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
                movement.z += force_multiplier; // Backward in world coordinates
            }
            if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
                movement.x -= force_multiplier; // Left in world coordinates
            }
            if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
                movement.x += force_multiplier; // Right in world coordinates
            }
        }
        
        // Rotation controls (Q/E keys) - always relative to world Y-axis
        if keyboard.pressed(KeyCode::KeyQ) {
            rotation.y += rotation_multiplier; // Rotate left (counter-clockwise)
        }
        if keyboard.pressed(KeyCode::KeyE) {
            rotation.y -= rotation_multiplier; // Rotate right (clockwise)
        }
        
        // Vertical movement (jump) - always world-relative
        if keyboard.just_pressed(KeyCode::Space) {
            movement.y += 2.0; // Jump
        }
        
        impulse.impulse = movement;
        impulse.torque_impulse = rotation;
    }
}

/// System to manually adjust oblique projection parameters (backup controls)
pub fn manual_adjust_oblique_projection(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut projection_query: Query<&mut Projection, With<ObliqueProjectionController>>,
) {
    if let Ok(mut projection) = projection_query.single_mut() {
        if let Projection::Custom(custom_projection) = projection.as_mut() {
            if let Some(oblique) = custom_projection.downcast_mut::<ObliquePerspectiveProjection>() {
                // Reset to defaults
                if keyboard.just_pressed(KeyCode::KeyR) {
                    oblique.horizontal_obliqueness = 0.0;
                    oblique.vertical_obliqueness = 0.0;
                    info!("Reset oblique projection to default values");
                }
            }
        }
    }
}

/// System to toggle LIDAR visualization and logging
pub fn toggle_lidar_visualization(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut lidar_query: Query<&mut LidarSensor>,
) {
    if keyboard.just_pressed(KeyCode::KeyL) {
        for mut lidar in lidar_query.iter_mut() {
            lidar.visualize = !lidar.visualize;
            info!("LIDAR visualization: {}", if lidar.visualize { "ON" } else { "OFF" });
        }
    }
    
    if keyboard.just_pressed(KeyCode::KeyO) {
        for mut lidar in lidar_query.iter_mut() {
            lidar.enable_logging = !lidar.enable_logging;
            info!("LIDAR obstacle logging: {}", if lidar.enable_logging { "ON" } else { "OFF" });
        }
    }
}

/// System to display robot control information
pub fn display_robot_controls_info(mut ran: Local<bool>) {
    if !*ran {
        *ran = true;
        info!("=== Camera-Relative Robot Movement Controls ===");
        info!("• WASD or Arrow Keys: Move robot (relative to camera view)");
        info!("  - W/Up: Forward (camera direction)");
        info!("  - S/Down: Backward (opposite camera direction)");
        info!("  - A/Left: Left (camera left)");
        info!("  - D/Right: Right (camera right)");
        info!("• Q/E Keys: Rotate robot");
        info!("  - Q: Rotate left (counter-clockwise)");
        info!("  - E: Rotate right (clockwise)");
        info!("• Spacebar: Jump");
        info!("• R key: Reset oblique projection to default");
        info!("• L key: Toggle LIDAR visualization");
        info!("• O key: Toggle LIDAR obstacle logging");
        info!("• Secondary window: Real-time robot first-person view");
        info!("  - Shows exactly what the robot is facing");
        info!("  - Camera follows robot position and rotation");
        info!("  - Subtle oblique projection effects based on movement");
        info!("• Main window: Pan-orbit camera (controls robot movement direction)");
        info!("  - Right-click + drag: Orbit around robot");
        info!("  - Middle-click + drag: Pan view");
        info!("  - Scroll: Zoom in/out");
        info!("==========================================");
    }
} 