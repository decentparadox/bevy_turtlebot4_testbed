use bevy::prelude::*;
use bevy_rapier3d::dynamics::ExternalImpulse;

#[derive(Component)]
pub struct Draggable {
    pub external_impulse: ExternalImpulse,
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
    mut draggables: Query<(Entity, &mut DragTarget, &GlobalTransform, &mut ExternalImpulse)>,
    time: Res<Time>,
) {
    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = cameras.single() else { return; };

    // Check if mouse is pressed
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_position) = window.cursor_position() {
            if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
                for (entity, mut drag_target, transform, _) in draggables.iter_mut() {
                    // Simple ray intersection check - in a real implementation you'd use proper ray casting
                    let distance_to_object = (transform.translation() - camera_transform.translation()).length();
                    if distance_to_object < 10.0 { // Simple distance check
                        drag_target.is_dragging = true;
                        drag_target.drag_start_pos = transform.translation();
                        drag_target.drag_start_mouse_pos = cursor_position;
                        drag_target.entity = entity;
                        break;
                    }
                }
            }
        }
    }

    // Check if mouse is released
    if mouse_button_input.just_released(MouseButton::Left) {
        for (_, mut drag_target, _, _) in draggables.iter_mut() {
            drag_target.is_dragging = false;
        }
    }

    // Handle dragging
    for (entity, drag_target, transform, mut external_impulse) in draggables.iter_mut() {
        if drag_target.is_dragging {
            if let Some(current_mouse_pos) = window.cursor_position() {
                let mouse_delta = current_mouse_pos - drag_target.drag_start_mouse_pos;

                // Convert mouse movement to world space movement
                let camera_right = camera_transform.right();
                let camera_up = camera_transform.up();

                let movement = (mouse_delta.x * camera_right - mouse_delta.y * camera_up) * 0.01;

                // Apply force to move the object
                external_impulse.impulse = movement * 10.0;
            }
        }
    }
}
