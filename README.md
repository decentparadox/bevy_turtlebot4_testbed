# Bevy TurtleBot4 Testbed

A 3D robotics simulation environment built with Bevy and Rapier physics, demonstrating the difference between visual and physics meshes, and supporting custom arena creation.

## Key Features

### 🎯 Visual vs Physics Meshes
This project demonstrates the important distinction between visual and physics meshes in robotics simulation:

- **Visual Meshes**: High-detail STL files for rendering and appearance
- **Physics Meshes**: Simplified collision shapes (boxes, cylinders, spheres) for performance

### 🌍 Custom Arena Creation
Create custom worlds with walls, obstacles, and complex environments:

- **Simple Arena**: Basic walls and colored obstacles
- **Complex Obstacles**: Detailed visual meshes with simplified physics
- **World Builder**: Modular system for creating custom environments

### 🤖 URDF Robot Support
Load and visualize robots from URDF files with STL visual meshes:

- **STL Loading**: Direct loading of STL files for visual meshes
- **Fallback Geometry**: Automatic fallback to primitive shapes if STL files are missing
- **Physics Integration**: Separate collision geometry for physics simulation

## Architecture

### Visual vs Physics Mesh Implementation

```rust
// Example: Complex obstacle with different visual and physics meshes
commands.spawn((
    // Physics: Simple box collider for performance
    Collider::cuboid(0.4, 0.3, 0.4),
    // ... physics components
))
.with_children(|commands| {
    // Visual: Complex multi-part mesh for appearance
    commands.spawn((
        Mesh3d(complex_mesh_handle),
        MeshMaterial3d(material_handle),
        // ... visual components
    ));
});
```

### Key Differences

| Aspect | Visual Mesh | Physics Mesh |
|--------|-------------|--------------|
| **Purpose** | Rendering and appearance | Collision detection |
| **Complexity** | High detail (STL files) | Simple primitives |
| **Performance** | Less critical | Critical for simulation speed |
| **File Format** | STL, GLB, etc. | Box, cylinder, sphere definitions |

## File Structure

```
src/
├── main.rs              # Main application setup
├── stl_loader.rs        # STL file loading and conversion
├── urdf_loader.rs       # URDF parsing and robot spawning
├── world_builder.rs     # Arena and world creation
├── turtlebot4.rs        # Robot-specific components
├── camera.rs           # Camera controls
├── keyboard_controls.rs # Input handling
├── lidar.rs            # LIDAR sensor simulation
└── robot_drag.rs       # Robot interaction

assets/
└── robots/
    ├── turtlebot4.glb   # Robot model
    └── urdf/
        ├── sample.urdf  # URDF robot description
        ├── test.urdf    # Test URDF
        └── *.STL        # Visual mesh files
```

## URDF vs SDF Format

### URDF (Unified Robot Description Format)
- **Purpose**: Robot-specific descriptions
- **Focus**: Kinematic chains, joints, links
- **Best for**: Individual robot models
- **Limitations**: Single robot, no world description

### SDF (Simulation Description Format)
- **Purpose**: General simulation descriptions
- **Focus**: Entire worlds, multiple objects
- **Best for**: Complete environments
- **Advantages**: Can describe entire scenes

## Usage

### Running the Application
```bash
cargo run
```

### Controls
- **WASD**: Move robot
- **Mouse**: Orbit camera
- **Space**: Toggle LIDAR visualization
- **R**: Reset robot position

### Creating Custom Arenas

```rust
// Spawn a simple arena
world_builder::spawn_simple_arena(commands, &mut meshes, &mut materials);

// Add custom obstacles
world_builder::spawn_complex_obstacle(
    commands, &mut meshes, &mut materials,
    Vec3::new(0.0, 0.0, -2.0),
    "My Obstacle"
);
```

## STL File Support

The system automatically loads STL files referenced in URDF files:

1. **Automatic Detection**: URDF parser finds mesh references
2. **STL Loading**: `stl_loader.rs` converts STL to Bevy meshes
3. **Fallback System**: Uses primitive shapes if STL files are missing
4. **Performance**: Optimized mesh conversion for real-time rendering

### Supported STL Files
- `base_link.STL` - Robot base
- `left_shoulder_link.STL` - Left shoulder
- `left_leg_link.STL` - Left leg
- `left_wheel_link.STL` - Left wheel
- `right_shoulder_link.STL` - Right shoulder
- `right_leg_link.STL` - Right leg
- `right_wheel_link.STL` - Right wheel
- `left_upper_link.STL` - Left upper arm
- `right_upper_link.STL` - Right upper arm
- `front_cover_link.STL` - Front cover
- `rear_cover_link.STL` - Rear cover

## Physics Groups

The simulation uses collision groups for organized physics:

- **STATIC_GROUP**: World objects (walls, obstacles)
- **CHASSIS_GROUP**: Robot chassis
- **CHASSIS_INTERNAL_GROUP**: Internal robot parts (wheels)

## Dependencies

```toml
[dependencies]
bevy = { version = "0.16", features = ["wayland", "dynamic_linking", "jpeg"] }
bevy_rapier3d = { version = "0.30.0", features = ["debug-render-3d"] }
stl = "0.6"  # STL file parsing
quick-xml = "0.31"  # URDF parsing
regex = "1.10"  # Text processing
```

## Development

### Adding New STL Files
1. Place STL files in `assets/robots/urdf/` (STL files are already provided)
2. Reference them in your URDF file
3. The system will automatically load them

### Creating Custom Worlds
1. Use `world_builder.rs` functions
2. Define both visual and physics meshes
3. Use collision groups for organized physics

### Extending Robot Support
1. Create new URDF files
2. Add corresponding STL files
3. Use `urdf_loader::spawn_urdf_scene()` to spawn robots

## Performance Considerations

- **Visual Meshes**: Can be complex without affecting physics performance
- **Physics Meshes**: Keep simple for better simulation speed
- **LOD (Level of Detail)**: Consider distance-based mesh switching
- **Collision Groups**: Use to optimize collision detection

## Future Enhancements

- [ ] SDF format support for complete world descriptions
- [ ] Mesh optimization and LOD systems
- [ ] Material and texture support
- [ ] Advanced physics materials (friction, restitution)
- [ ] Multi-robot support
- [ ] Sensor simulation (cameras, IMU, etc.)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.