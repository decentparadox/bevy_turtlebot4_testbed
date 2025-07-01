use bevy::prelude::*;
use bevy_rapier3d::dynamics::ExternalImpulse;
use crate::camera::DragTarget;

/// System to handle drag start events - adds DragTarget component
pub fn on_drag_start(
    trigger: Trigger<Pointer<DragStart>>,
    mut commands: Commands,
) {
    info!("Drag started on entity: {:?}", trigger.target());
    commands.entity(trigger.target()).insert(DragTarget);
}

/// System to handle drag end events - removes DragTarget component
pub fn on_drag_end(
    trigger: Trigger<Pointer<DragEnd>>,
    mut commands: Commands,
) {
    info!("Drag ended on entity: {:?}", trigger.target());
    commands.entity(trigger.target()).remove::<DragTarget>();
}

/// System to handle drag events - applies forces to draggable entities
pub fn on_drag(
    trigger: Trigger<Pointer<Drag>>,
    mut draggable_objects: Query<&mut ExternalImpulse, With<crate::Draggable>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<crate::Draggable>)>,
) {
    if let Ok(mut impulse) = draggable_objects.get_mut(trigger.target()) {
        if let Ok(camera_transform) = camera_query.single() {
            // Get camera's right and forward vectors (in XZ plane)
            let camera_right = camera_transform.right();
            let camera_forward = camera_transform.forward();
            
            // Project vectors onto XZ plane and normalize
            let camera_right_xz = Vec3::new(camera_right.x, 0.0, camera_right.z).normalize_or_zero();
            let camera_forward_xz = Vec3::new(camera_forward.x, 0.0, camera_forward.z).normalize_or_zero();
            
            // Transform mouse delta relative to camera orientation
            let force_multiplier = 0.05;
            let drag_delta = trigger.event().delta;
            let force = camera_right_xz * drag_delta.x * force_multiplier 
                      - camera_forward_xz * drag_delta.y * force_multiplier; // Negative because forward drag should move forward
            
            impulse.impulse = force;
            info!("Applied drag force: {:?} from delta: {:?}", force, drag_delta);
        }
    } else {
        warn!("Could not find draggable object for entity {:?}", trigger.target());
    }
}

/// System to handle click events - applies upward impulse
pub fn on_click(
    trigger: Trigger<Pointer<Click>>,
    mut draggable_objects: Query<&mut ExternalImpulse, With<crate::Draggable>>,
) {
    info!("Click event on entity: {:?}", trigger.target());
    
    if let Ok(mut impulse) = draggable_objects.get_mut(trigger.target()) {
        impulse.impulse = Vec3::new(0.0, 50.0, 0.0);
        info!("Applied upward impulse to robot");
    }
}

/// System to handle pointer over events
pub fn on_pointer_over(
    trigger: Trigger<Pointer<Over>>,
) {
    info!("Pointer over entity: {:?}", trigger.target());
}

/// System to handle pointer out events  
pub fn on_pointer_out(
    trigger: Trigger<Pointer<Out>>,
) {
    info!("Pointer out of entity: {:?}", trigger.target());
}