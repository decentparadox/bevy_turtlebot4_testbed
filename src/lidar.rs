use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;
use bevy::reflect::Reflect;
use bevy_inspector_egui::prelude::*;

// RPLIDAR A1M8 specifications 
const LIDAR_RANGE_MIN: f32 = 0.2;     // 0.2 meters minimum range
const LIDAR_RANGE_MAX: f32 = 12.0;    // 12 meters maximum range
const LIDAR_SCAN_RATE: f32 = 10.0;    // 10 Hz scan rate
const LIDAR_RAYS_PER_SCAN: usize = 36; // Reduced for performance (every 10 degrees)
const LIDAR_ANGULAR_RESOLUTION: f32 = 2.0 * PI / LIDAR_RAYS_PER_SCAN as f32; // 10° per ray

/// LIDAR sensor component with obstacle detection
#[derive(Component, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct LidarSensor {
    /// Minimum range (meters)
    #[inspector(min = 0.01, max = 1.0, speed = 0.01)]
    pub range_min: f32,
    /// Maximum range (meters)
    #[inspector(min = 1.0, max = 50.0, speed = 0.1)]
    pub range_max: f32,
    /// Scan rate (Hz)
    #[inspector(min = 0.1, max = 100.0, speed = 0.1)]
    pub scan_rate: f32,
    /// Number of rays per scan
    #[inspector(min = 4, max = 720, speed = 1.0)]
    pub rays_per_scan: usize,
    /// Angular resolution (radians) - READ ONLY
    #[inspector(min = 0.0, max = 10.0)]
    pub angular_resolution: f32,
    /// Timer for scan rate control
    #[reflect(ignore)]
    pub scan_timer: Timer,
    /// Position offset from parent entity
    pub offset: Vec3,
    /// Whether to enable visualization
    pub visualize: bool,
    /// Current ray angle
    #[reflect(ignore)]
    pub current_angle: f32,
    /// Current ray index in scan
    #[reflect(ignore)]
    pub current_ray: usize,
    /// Last scan results: (angle, distance, hit_something)
    #[reflect(ignore)]
    pub scan_results: Vec<(f32, f32, bool)>,
    /// Whether to enable logging
    pub enable_logging: bool,
}

impl Default for LidarSensor {
    fn default() -> Self {
        LidarSensor {
            range_min: LIDAR_RANGE_MIN,
            range_max: LIDAR_RANGE_MAX,
            scan_rate: LIDAR_SCAN_RATE,
            rays_per_scan: LIDAR_RAYS_PER_SCAN,
            angular_resolution: LIDAR_ANGULAR_RESOLUTION,
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

impl LidarSensor {
    /// Update internal parameters when inspector values change
    pub fn update_parameters(&mut self) {
        // Recalculate angular resolution
        self.angular_resolution = 2.0 * PI / self.rays_per_scan as f32;
        
        // Update timer with new scan rate
        self.scan_timer = Timer::from_seconds(1.0 / self.scan_rate, TimerMode::Repeating);
        
        // Resize scan results if rays_per_scan changed
        if self.scan_results.capacity() != self.rays_per_scan {
            self.scan_results = Vec::with_capacity(self.rays_per_scan);
        }
    }
}

/// Plugin for LIDAR functionality
pub struct LidarPlugin;

impl Plugin for LidarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            lidar_parameter_update_system,
            lidar_scanning_system,
            lidar_visualization_system,
        ))
        .register_type::<LidarSensor>()
        .register_type::<Vec3>()
        .register_type::<Timer>();
    }
}

/// System to update LIDAR parameters when they change in the inspector
pub fn lidar_parameter_update_system(
    mut lidar_query: Query<&mut LidarSensor, Changed<LidarSensor>>,
) {
    for mut lidar in lidar_query.iter_mut() {
        lidar.update_parameters();
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
            
            // Perform 360-degree scan using configurable parameters
            for i in 0..lidar.rays_per_scan {
                let angle = i as f32 * lidar.angular_resolution;
                
                // Calculate ray direction (starting from +X axis, rotating clockwise in XZ plane)
                let local_direction = Vec3::new(
                    angle.cos(),
                    0.0,
                    -angle.sin(), // Negative for clockwise rotation like real RPLIDAR
                );
                let world_direction = lidar_transform.rotation() * local_direction;
                
                // Find closest obstacle in this direction
                let mut closest_distance = lidar.range_max;
                let mut found_obstacle = false;
                
                for obstacle_transform in obstacle_query.iter() {
                    let obstacle_pos = obstacle_transform.translation();
                    let to_obstacle = obstacle_pos - lidar_pos;
                    
                    // Skip if obstacle is too close or too far
                    let distance_to_obstacle = to_obstacle.length();
                    if distance_to_obstacle < lidar.range_min || distance_to_obstacle > lidar.range_max {
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
                      obstacles_detected, lidar.rays_per_scan);
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
            
            // Color based on hit status and distance with 5% opacity
            let base_color = if hit_something {
                // Hit something - color by distance (close = red, far = yellow)
                let distance_ratio = (distance - lidar.range_min) / (lidar.range_max - lidar.range_min);
                Color::srgba(1.0, distance_ratio.clamp(0.0, 1.0), 0.0, 0.05)
            } else {
                // No hit - gray with 5% opacity
                Color::srgba(0.3, 0.3, 0.3, 0.05)
            };
            
            // Draw dotted ray line by drawing segments with gaps
            let ray_vector = hit_point - lidar_pos;
            let ray_length = ray_vector.length();
            let ray_direction = ray_vector.normalize();
            
            // Dotted line parameters
            let dash_length = 0.05; // 5cm dashes
            let gap_length = 0.03;  // 3cm gaps
            let segment_length = dash_length + gap_length;
            
            let num_segments = (ray_length / segment_length).floor() as i32;
            
            for i in 0..num_segments {
                let start_distance = i as f32 * segment_length;
                let end_distance = start_distance + dash_length;
                
                if end_distance <= ray_length {
                    let start_point = lidar_pos + ray_direction * start_distance;
                    let end_point = lidar_pos + ray_direction * end_distance;
                    gizmos.line(start_point, end_point, base_color);
                }
            }
            
            // Draw the final segment if there's a remainder
            let remainder_start = num_segments as f32 * segment_length;
            if remainder_start < ray_length {
                let remainder_end = (remainder_start + dash_length).min(ray_length);
                if remainder_end > remainder_start {
                    let start_point = lidar_pos + ray_direction * remainder_start;
                    let end_point = lidar_pos + ray_direction * remainder_end;
                    gizmos.line(start_point, end_point, base_color);
                }
            }
            
            // Draw hit points for obstacles with slightly higher opacity
            if hit_something {
                let hit_cross_size = 0.015;
                let hit_color = Color::srgba(1.0, 0.0, 0.0, 0.3); // 30% opacity for hit markers
                gizmos.line(
                    hit_point + Vec3::new(-hit_cross_size, 0.0, 0.0),
                    hit_point + Vec3::new(hit_cross_size, 0.0, 0.0),
                    hit_color
                );
                gizmos.line(
                    hit_point + Vec3::new(0.0, 0.0, -hit_cross_size),
                    hit_point + Vec3::new(0.0, 0.0, hit_cross_size),
                    hit_color
                );
            }
        }
        
        // Draw LIDAR sensor center (cyan cross) with higher opacity
        let cross_size = 0.05;
        let center_color = Color::srgba(0.0, 1.0, 1.0, 0.8); // 80% opacity for sensor center
        gizmos.line(
            lidar_pos + Vec3::new(-cross_size, 0.0, 0.0),
            lidar_pos + Vec3::new(cross_size, 0.0, 0.0),
            center_color
        );
        gizmos.line(
            lidar_pos + Vec3::new(0.0, 0.0, -cross_size),
            lidar_pos + Vec3::new(0.0, 0.0, cross_size),
            center_color
        );
        
        // Draw current ray direction indicator with medium opacity
        if !lidar.scan_results.is_empty() {
            let local_direction = Vec3::new(
                lidar.current_angle.cos(),
                0.0,
                -lidar.current_angle.sin(),
            );
            let world_direction = transform.rotation() * local_direction;
            let direction_end = lidar_pos + world_direction * 0.4;
            let direction_color = Color::srgba(1.0, 1.0, 1.0, 0.6); // 60% opacity for current ray
            gizmos.line(lidar_pos, direction_end, direction_color);
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