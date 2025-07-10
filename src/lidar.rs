use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

// RPLIDAR A1M8 specifications 
const LIDAR_RANGE_MIN: f32 = 0.2;     // 0.2 meters minimum range
const LIDAR_RANGE_MAX: f32 = 12.0;    // 12 meters maximum range
const LIDAR_SCAN_RATE: f32 = 10.0;    // 10 Hz scan rate
const LIDAR_RAYS_PER_SCAN: usize = 36; // Reduced for performance (every 10 degrees)
const LIDAR_ANGULAR_RESOLUTION: f32 = 2.0 * PI / LIDAR_RAYS_PER_SCAN as f32; // 10° per ray

/// LIDAR sensor component with obstacle detection
#[derive(Component)]
pub struct LidarSensor {
    /// Timer for scan rate control
    pub scan_timer: Timer,
    /// Position offset from parent entity
    pub offset: Vec3,
    /// Whether to enable visualization
    pub visualize: bool,
    /// Current ray angle
    pub current_angle: f32,
    /// Current ray index in scan
    pub current_ray: usize,
    /// Last scan results: (angle, distance, hit_something)
    pub scan_results: Vec<(f32, f32, bool)>,
    /// Whether to enable logging
    pub enable_logging: bool,
}

impl Default for LidarSensor {
    fn default() -> Self {
        LidarSensor {
            scan_timer: Timer::from_seconds(1.0 / LIDAR_SCAN_RATE, TimerMode::Repeating),
            offset: Vec3::ZERO, // Position set in Transform
            visualize: true,
            current_angle: 0.0,
            current_ray: 0,
            scan_results: Vec::with_capacity(LIDAR_RAYS_PER_SCAN),
            enable_logging: true,
        }
    }
}

/// Plugin for LIDAR functionality
pub struct LidarPlugin;

impl Plugin for LidarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            lidar_scanning_system,
            lidar_visualization_system,
        ));
    }
}

/// System that performs LIDAR scanning by detecting nearby obstacles
pub fn lidar_scanning_system(
    time: Res<Time>,
    mut lidar_query: Query<(&mut LidarSensor, &GlobalTransform, Entity), With<LidarSensor>>,
    obstacle_query: Query<&GlobalTransform, (With<Collider>, Without<LidarSensor>)>,
) {
    for (mut lidar, lidar_transform, lidar_entity) in lidar_query.iter_mut() {
        lidar.scan_timer.tick(time.delta());
        
        if lidar.scan_timer.just_finished() {
            // Start new scan
            lidar.scan_results.clear();
            lidar.current_ray = 0;
            lidar.current_angle = 0.0;
            
            // Get LIDAR world position
            let lidar_pos = lidar_transform.translation();
            
            // Perform 360-degree scan
            for i in 0..LIDAR_RAYS_PER_SCAN {
                let angle = i as f32 * LIDAR_ANGULAR_RESOLUTION;
                
                // Calculate ray direction (starting from +X axis, rotating clockwise in XZ plane)
                let local_direction = Vec3::new(
                    angle.cos(),
                    0.0,
                    -angle.sin(), // Negative for clockwise rotation like real RPLIDAR
                );
                let world_direction = lidar_transform.rotation() * local_direction;
                
                // Find closest obstacle in this direction
                let mut closest_distance = LIDAR_RANGE_MAX;
                let mut found_obstacle = false;
                
                for obstacle_transform in obstacle_query.iter() {
                    let obstacle_pos = obstacle_transform.translation();
                    let to_obstacle = obstacle_pos - lidar_pos;
                    
                    // Skip if obstacle is too close or too far
                    let distance_to_obstacle = to_obstacle.length();
                    if distance_to_obstacle < LIDAR_RANGE_MIN || distance_to_obstacle > LIDAR_RANGE_MAX {
                        continue;
                    }
                    
                    // Check if obstacle is in the direction of our ray (within a cone)
                    let to_obstacle_normalized = to_obstacle.normalize();
                    let dot_product = world_direction.dot(to_obstacle_normalized);
                    
                    // Angular tolerance (roughly 5 degrees on each side)
                    let angular_tolerance: f32 = 0.087; // ~5 degrees in radians
                    let min_dot = angular_tolerance.cos();
                    
                    if dot_product > min_dot {
                        // Obstacle is in this ray's direction
                        if distance_to_obstacle < closest_distance {
                            closest_distance = distance_to_obstacle;
                            found_obstacle = true;
                        }
                    }
                }
                
                // Store result
                lidar.scan_results.push((angle, closest_distance, found_obstacle));
                
                // Log obstacle detection
                if found_obstacle && lidar.enable_logging {
                    let angle_degrees = angle * 180.0 / PI;
                    info!(
                        "LIDAR: Obstacle detected at {:.1}° - Distance: {:.2}m",
                        angle_degrees, closest_distance
                    );
                }
            }
            
            // Update current values for visualization
            if !lidar.scan_results.is_empty() {
                let (angle, _, _) = lidar.scan_results[lidar.current_ray];
                lidar.current_angle = angle;
            }
            
            debug!("LIDAR scan completed: {} rays", lidar.scan_results.len());
            
            // Log summary of obstacles found
            let obstacles_detected = lidar.scan_results.iter().filter(|(_, _, hit)| *hit).count();
            if lidar.enable_logging && obstacles_detected > 0 {
                info!("LIDAR scan summary: {} obstacles detected out of {} rays", 
                      obstacles_detected, LIDAR_RAYS_PER_SCAN);
            }
        }
        
        // Update current ray for visualization (rotate through scan results)
        if !lidar.scan_results.is_empty() {
            lidar.current_ray = (lidar.current_ray + 1) % lidar.scan_results.len();
            let (angle, _, _) = lidar.scan_results[lidar.current_ray];
            lidar.current_angle = angle;
        }
    }
}

/// System to visualize LIDAR rays using gizmos
pub fn lidar_visualization_system(
    mut gizmos: Gizmos,
    lidar_query: Query<(&LidarSensor, &GlobalTransform)>,
) {
    for (lidar, transform) in lidar_query.iter() {
        if !lidar.visualize {
            continue;
        }
        
        let lidar_pos = transform.translation();
        
        // Draw all rays from the last scan
        for &(angle, distance, hit_something) in &lidar.scan_results {
            // Calculate ray direction
            let local_direction = Vec3::new(
                angle.cos(),
                0.0,
                -angle.sin(),
            );
            let world_direction = transform.rotation() * local_direction;
            
            // Calculate hit point
            let hit_point = lidar_pos + world_direction * distance;
            
            // Color based on hit status and distance
            let color = if hit_something {
                // Hit something - color by distance (close = red, far = yellow)
                let distance_ratio = (distance - LIDAR_RANGE_MIN) / (LIDAR_RANGE_MAX - LIDAR_RANGE_MIN);
                Color::srgb(1.0, distance_ratio.clamp(0.0, 1.0), 0.0)
            } else {
                // No hit - gray
                Color::srgb(0.3, 0.3, 0.3)
            };
            
            // Draw ray line
            gizmos.line(lidar_pos, hit_point, color);
            
            // Draw hit points for obstacles
            if hit_something {
                let hit_cross_size = 0.015;
                gizmos.line(
                    hit_point + Vec3::new(-hit_cross_size, 0.0, 0.0),
                    hit_point + Vec3::new(hit_cross_size, 0.0, 0.0),
                    Color::srgb(1.0, 0.0, 0.0)
                );
                gizmos.line(
                    hit_point + Vec3::new(0.0, 0.0, -hit_cross_size),
                    hit_point + Vec3::new(0.0, 0.0, hit_cross_size),
                    Color::srgb(1.0, 0.0, 0.0)
                );
            }
        }
        
        // Draw LIDAR sensor center (cyan cross)
        let cross_size = 0.05;
        gizmos.line(
            lidar_pos + Vec3::new(-cross_size, 0.0, 0.0),
            lidar_pos + Vec3::new(cross_size, 0.0, 0.0),
            Color::srgb(0.0, 1.0, 1.0)
        );
        gizmos.line(
            lidar_pos + Vec3::new(0.0, 0.0, -cross_size),
            lidar_pos + Vec3::new(0.0, 0.0, cross_size),
            Color::srgb(0.0, 1.0, 1.0)
        );
        
        // Draw current ray direction indicator
        if !lidar.scan_results.is_empty() {
            let local_direction = Vec3::new(
                lidar.current_angle.cos(),
                0.0,
                -lidar.current_angle.sin(),
            );
            let world_direction = transform.rotation() * local_direction;
            let direction_end = lidar_pos + world_direction * 0.4;
            gizmos.line(lidar_pos, direction_end, Color::srgb(1.0, 1.0, 1.0));
        }
    }
}

/// Helper function to spawn a LIDAR sensor on an entity
pub fn spawn_lidar_sensor(commands: &mut Commands, parent_entity: Entity) {
    commands.entity(parent_entity).with_children(|parent| {
        parent.spawn((
            LidarSensor::default(),
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
        ));
    });
} 