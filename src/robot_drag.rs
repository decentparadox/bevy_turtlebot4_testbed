use bevy::prelude::*;
use bevy_rapier3d::dynamics::ExternalImpulse;

#[derive(Component)]
pub struct Draggable;

#[derive(Bundle)]
pub struct DraggableBundle {
    pub draggable: Draggable,
    pub external_impulse: ExternalImpulse,
}

impl Default for DraggableBundle {
    fn default() -> Self {
        Self {
            draggable: Draggable,
            external_impulse: ExternalImpulse::default(),
        }
    }
}

#[derive(Component)]
pub struct DragTarget {
    pub is_dragging: bool,
    pub drag_start_pos: Vec3,
    pub drag_start_mouse_pos: Vec2,
    pub entity: Entity,
}

pub fn drag_system(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut draggables: Query<(Entity, &mut ExternalImpulse, &GlobalTransform), With<Draggable>>,
    mut drag_targets: Query<(Entity, &mut DragTarget)>,
    _time: Res<Time>,
) {
    let Ok(window) = windows.single() else { return; };
    let Ok((_camera, camera_transform)) = cameras.single() else { return; };

    // Handle mouse press - start dragging with simple ray casting
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_position) = window.cursor_position() {
            // Simple ray casting to find the closest draggable entity to the cursor
            if let Ok(ray) = cameras.single().unwrap().0.viewport_to_world(camera_transform, cursor_position) {
                let mut closest_entity = None;
                let mut closest_distance = f32::INFINITY;
                
                for (entity, _, transform) in draggables.iter() {
                    let entity_pos = transform.translation();
                    let ray_to_entity = entity_pos - ray.origin;
                    let projection = ray_to_entity.dot(*ray.direction);
                    
                    if projection > 0.0 {
                        let closest_point_on_ray = ray.origin + *ray.direction * projection;
                        let distance = (entity_pos - closest_point_on_ray).length();
                        
                        // Simple radius check (adjust this value based on your link sizes)
                        if distance < 0.5 && projection < closest_distance {
                            closest_distance = projection;
                            closest_entity = Some((entity, transform.translation()));
                        }
                    }
                }
                
                if let Some((draggable_entity, entity_pos)) = closest_entity {
                    commands.spawn(DragTarget {
                        is_dragging: true,
                        drag_start_pos: entity_pos,
                        drag_start_mouse_pos: cursor_position,
                        entity: draggable_entity,
                    });
                }
            }
        }
    }

    // Handle mouse release
    if mouse_button_input.just_released(MouseButton::Left) {
        for (entity, mut drag_target) in drag_targets.iter_mut() {
            if drag_target.is_dragging {
                drag_target.is_dragging = false;
                commands.entity(entity).despawn(); // Remove DragTarget component
            }
        }
    }

    // Handle active dragging
    if let Some(cursor_position) = window.cursor_position() {
        for (_, drag_target) in drag_targets.iter_mut() {
            if drag_target.is_dragging {
                if let Ok((_, mut external_impulse, transform)) = draggables.get_mut(drag_target.entity) {
                    // Calculate mouse movement
                    let mouse_delta = cursor_position - drag_target.drag_start_mouse_pos;

                    // Convert mouse delta to world space movement (simplified)
                    let camera_right = camera_transform.right();
                    let camera_up = camera_transform.up();

                    // Project mouse movement onto the plane defined by the camera's view
                    let world_delta = camera_right * mouse_delta.x * 0.003 + camera_up * -mouse_delta.y * 0.003; // Ultra-smooth control scaling

                    // Calculate target position in world space
                    let target_position = drag_target.drag_start_pos + world_delta;

                    // Apply impulse to move the object towards the target
                    const DRAG_FORCE_GAIN: f32 = 1.5; // Even gentler for highly realistic feel
                    const DAMPING_FACTOR: f32 = 0.7; // Stronger damping to reduce oscillations
                    let force_direction = target_position - transform.translation();
                    
                    // Only apply force if the distance is reasonable
                    if force_direction.length() < 3.0 {
                        // Apply proportional force with distance-based scaling
                        let distance = force_direction.length();
                        let proportional_gain = (distance / 1.5).min(1.0); // Even gentler ramp-up
                        
                        // Use exponential decay for very smooth forces
                        let exponential_gain = 1.0 - (-distance * 2.0).exp();
                        
                        // Clamp the force to prevent extreme values
                        let max_force = 15.0; // Even lower max force
                        let scaled_force = force_direction.normalize_or_zero() * distance * proportional_gain * exponential_gain * DRAG_FORCE_GAIN;
                        let clamped_force = scaled_force.clamp_length_max(max_force);
                        
                        // Apply stronger damping to previous impulse for smoother motion
                        external_impulse.impulse = external_impulse.impulse * DAMPING_FACTOR + clamped_force * 0.3;
                    } else {
                        // Gradually reduce impulse instead of instant reset
                        external_impulse.impulse *= 0.3; // Faster decay when out of range
                    }
                }
            }
        }
    }
}

/// Helper function to add draggable component to entities (for compatibility)
pub fn make_robot_draggable() {
    // This function is kept for compatibility but functionality is handled by DraggableBundle
}

/// Plugin for robot dragging functionality
pub struct RobotDragPlugin;

impl Plugin for RobotDragPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, drag_system);
    }
}