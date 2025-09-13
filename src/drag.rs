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

    // Handle mouse press - start dragging (simplified - click on any draggable)
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_position) = window.cursor_position() {
            // For simplicity, just start dragging the first draggable entity
            // In a real implementation, you'd do proper ray casting
            if let Some((draggable_entity, _, transform)) = draggables.iter().next() {
                commands.spawn(DragTarget {
                    is_dragging: true,
                    drag_start_pos: transform.translation(),
                    drag_start_mouse_pos: cursor_position,
                    entity: draggable_entity,
                });
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
                    let world_delta = camera_right * mouse_delta.x * 0.01 + camera_up * -mouse_delta.y * 0.01; // Scale factor

                    // Calculate target position in world space
                    let target_position = drag_target.drag_start_pos + world_delta;

                    // Apply impulse to move the object towards the target
                    const DRAG_FORCE_GAIN: f32 = 50.0; // Adjust this for stronger/weaker drag
                    let force_direction = target_position - transform.translation();
                    external_impulse.impulse = force_direction * DRAG_FORCE_GAIN;
                }
            }
        }
    }
}