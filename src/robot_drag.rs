use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Component to mark the robot as draggable
#[derive(Component)]
pub struct DraggableRobot;

/// Resource to track drag state
#[derive(Resource, Default)]
pub struct DragState {
    pub is_dragging: bool,
    pub drag_entity: Option<Entity>,
    pub drag_offset: Vec3,
    pub initial_position: Vec3,
}

/// Plugin for robot dragging functionality
pub struct RobotDragPlugin;

impl Plugin for RobotDragPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
            .add_systems(Update, (
                handle_robot_drag_input,
                update_dragged_robot,
            ));
    }
}

/// System to handle mouse input for robot dragging
pub fn handle_robot_drag_input(
    mut drag_state: ResMut<DragState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::camera::PanOrbitCamera>>,
    robot_query: Query<(Entity, &GlobalTransform), (With<DraggableRobot>, With<crate::RobotChassis>)>,
) {
    let Ok(window) = window_query.single() else { return; };
    let Ok((camera, camera_transform)) = camera_query.single() else { return; };
    
    // Check for drag start (left mouse button pressed)
    if mouse_input.just_pressed(MouseButton::Left) && !drag_state.is_dragging {
        if let Some(cursor_position) = window.cursor_position() {
            // Convert cursor position to world ray
            if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
                // Check if ray intersects with any robot using spatial query
                for (robot_entity, robot_transform) in robot_query.iter() {
                    let robot_position = robot_transform.translation();
                    
                    // Check if ray hits the robot's bounding sphere (simple collision detection)
                    let robot_radius = 0.2; // Approximate robot radius
                                         let closest_point = closest_point_on_ray_to_point(ray.origin, *ray.direction, robot_position);
                    let distance_to_robot = (closest_point - robot_position).length();
                    
                    if distance_to_robot <= robot_radius {
                        // Calculate the actual hit point on the ray closest to the robot
                        let hit_point = closest_point;
                        
                        drag_state.is_dragging = true;
                        drag_state.drag_entity = Some(robot_entity);
                        drag_state.drag_offset = hit_point - robot_position;
                        drag_state.initial_position = robot_position;
                        
                        break;
                    }
                }
            }
        }
    }
    
    // Check for drag end (left mouse button released or escape pressed)
    if (mouse_input.just_released(MouseButton::Left) || keyboard_input.just_pressed(KeyCode::Escape)) 
        && drag_state.is_dragging {
        drag_state.is_dragging = false;
        drag_state.drag_entity = None;
    }
}

/// System to update robot position while dragging
pub fn update_dragged_robot(
    drag_state: Res<DragState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::camera::PanOrbitCamera>>,
    mut robot_query: Query<&mut Transform, (With<DraggableRobot>, With<crate::RobotChassis>)>,
) {
    if !drag_state.is_dragging {
        return;
    }
    
    let Some(drag_entity) = drag_state.drag_entity else { return; };
    let Ok(window) = window_query.single() else { return; };
    let Ok((camera, camera_transform)) = camera_query.single() else { return; };
    let Ok(mut robot_transform) = robot_query.get_mut(drag_entity) else { return; };
    
    if let Some(cursor_position) = window.cursor_position() {
        // Convert cursor position to world ray
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            // Project ray onto ground plane (Y = 0)
            let ground_y = 0.0;
            
            // Calculate intersection with ground plane
            if ray.direction.y != 0.0 {
                let t = (ground_y - ray.origin.y) / ray.direction.y;
                if t > 0.0 {
                    let world_position = ray.origin + *ray.direction * t;
                    
                    // Update robot position (keeping Y at ground level)
                    let new_position = Vec3::new(
                        world_position.x - drag_state.drag_offset.x,
                        robot_transform.translation.y, // Keep current Y position
                        world_position.z - drag_state.drag_offset.z,
                    );
                    
                    robot_transform.translation = new_position;
                }
            }
        }
    }
}

/// Helper function to find closest point on ray to a given point
fn closest_point_on_ray_to_point(ray_origin: Vec3, ray_direction: Vec3, point: Vec3) -> Vec3 {
    let to_point = point - ray_origin;
    let projection_length = to_point.dot(ray_direction.normalize());
    let projection_length = projection_length.max(0.0); // Clamp to ray (not line)
    ray_origin + ray_direction.normalize() * projection_length
}

/// Helper function to add draggable component to robot
pub fn make_robot_draggable(mut commands: Commands, robot_query: Query<Entity, (With<crate::RobotChassis>, Without<DraggableRobot>)>) {
    for entity in robot_query.iter() {
        commands.entity(entity).insert(DraggableRobot);
    }
} 