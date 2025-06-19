// Drag functionality is temporarily disabled while migrating to Bevy 0.15
// This will be reimplemented using the built-in picking system in Bevy 0.15

use bevy::prelude::*;

#[derive(Component)]
pub struct DraggableMarker;

// Empty system for now - will be reimplemented later
pub fn drag_system() {
    // TODO: Reimplement drag functionality using Bevy 0.15 picking system
}