# SDF (Simulation Description Format) Support for Bevy TurtleBot4 Testbed

This project now includes comprehensive SDF support, similar to ROS/Gazebo, allowing you to load complex worlds and models into your Bevy simulation.

## Features

### 1. SDF Parser (`src/sdf_loader.rs`)
- **Complete SDF parsing**: Supports SDF 1.6 format with worlds, models, links, joints, visuals, and collisions
- **Geometry support**: Box, sphere, cylinder, plane, mesh, and heightmap geometries
- **Material support**: Ambient, diffuse, specular, emissive colors and textures
- **Physics support**: Mass, inertia, joint limits, and dynamics
- **Pose handling**: Full 6DOF positioning with translation and rotation

### 2. Mesh Converter (`sdf_mesh_converter.py`)
- **Multi-format support**: Converts COLLADA (.dae), OBJ, STL, PLY, and OFF files to GLTF/GLB
- **URI resolution**: Handles `file://`, `model://`, and `package://` URI schemes
- **Batch processing**: Can process multiple SDF files recursively
- **Scale preservation**: Maintains mesh scaling from SDF definitions
- **Path management**: Updates SDF references to point to converted meshes

### 3. Bevy World Loader (`src/sdf_world_loader.rs`)
- **Complete world loading**: Spawns entire SDF worlds as Bevy entity hierarchies
- **Physics integration**: Automatic Rapier3D collider generation from SDF collision geometries
- **Material conversion**: Converts SDF materials to Bevy PBR materials
- **Asset management**: Efficient handling of mesh assets and scene loading
- **Component system**: Marks entities with SDF-specific components for easy identification

## Quick Start

### 1. Load an SDF World in Bevy

```rust
use bevy::prelude::*;
use your_project::sdf_world_loader::*;

fn setup_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut world_registry: ResMut<SdfWorldRegistry>,
) {
    // Load SDF world at specific position
    spawn_sdf_world_at_startup(
        &mut commands,
        asset_server,
        &mut world_registry,
        "assets/worlds/my_world.sdf",
        Vec3::new(0.0, 0.0, 0.0),
    );
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SdfWorldPlugin)  // Add SDF support
        .add_systems(Startup, setup_world)
        .run();
}
```

### 2. Convert SDF Meshes

Before loading SDF files with mesh references, convert them to Bevy-compatible formats:

#### Using the GUI (Windows):
```bash
# Run the conversion script
convert_sdf_meshes.bat
# or
convert_sdf_meshes.ps1
```

#### Using Python directly:
```bash
# Install dependencies
pip install trimesh[easy] pygltflib lxml

# Convert single SDF file
python sdf_mesh_converter.py assets/worlds/my_world.sdf --output-dir assets/converted --format glb

# Convert all SDF files in directory
python sdf_mesh_converter.py assets/worlds/ --recursive --format gltf
```

### 3. Organize Your Assets

Recommended directory structure:
```
assets/
├── worlds/
│   ├── simple_world.sdf
│   └── complex_world.sdf
├── models/
│   ├── robot_model/
│   │   ├── model.sdf
│   │   └── meshes/
│   │       ├── base.dae
│   │       └── arm.obj
│   └── environment/
├── converted/
│   ├── base.glb
│   └── arm.glb
└── textures/
```

## SDF Format Support

### Supported Elements

#### World
- `<physics>`: Gravity, step size, real-time factors
- `<scene>`: Ambient lighting, background, shadows
- `<model>`: Multiple models per world

#### Model
- `<static>`: Static/dynamic flag
- `<pose>`: 6DOF positioning
- `<link>`: Model links with physics properties
- `<joint>`: Joints between links (revolute, prismatic, etc.)

#### Link
- `<inertial>`: Mass and inertia matrix
- `<visual>`: Visual appearance
- `<collision>`: Collision geometry

#### Geometry Types
- `<box>`: Rectangular prisms
- `<sphere>`: Spheres
- `<cylinder>`: Cylinders
- `<plane>`: Infinite planes
- `<mesh>`: External mesh files (converted to GLTF/GLB)
- `<heightmap>`: Terrain heightmaps

#### Materials
- Ambient, diffuse, specular, emissive colors
- Texture mapping support
- PBR material conversion

### Example SDF File

```xml
<?xml version="1.0" ?>
<sdf version="1.6">
  <world name="example_world">
    <physics name="default_physics" default="1" type="ode">
      <gravity>0 0 -9.81</gravity>
      <max_step_size>0.001</max_step_size>
    </physics>
    
    <model name="ground_plane">
      <static>true</static>
      <link name="ground_link">
        <collision name="ground_collision">
          <geometry>
            <plane>
              <normal>0 0 1</normal>
              <size>10 10</size>
            </plane>
          </geometry>
        </collision>
        <visual name="ground_visual">
          <geometry>
            <plane>
              <normal>0 0 1</normal>
              <size>10 10</size>
            </plane>
          </geometry>
          <material>
            <diffuse>0.7 0.7 0.7 1.0</diffuse>
          </material>
        </visual>
      </link>
    </model>
    
    <model name="robot">
      <pose>0 0 0.5 0 0 0</pose>
      <link name="base_link">
        <inertial>
          <mass>10.0</mass>
          <inertia>
            <ixx>1.0</ixx>
            <iyy>1.0</iyy>
            <izz>1.0</izz>
            <ixy>0</ixy>
            <ixz>0</ixz>
            <iyz>0</iyz>
          </inertia>
        </inertial>
        <visual name="base_visual">
          <geometry>
            <mesh>
              <uri>robots/turtlebot4.glb</uri>
              <scale>1 1 1</scale>
            </mesh>
          </geometry>
        </visual>
        <collision name="base_collision">
          <geometry>
            <box>
              <size>0.4 0.4 0.2</size>
            </box>
          </geometry>
        </collision>
      </link>
    </model>
  </world>
</sdf>
```

## Advanced Usage

### Custom Mesh Loading

If you need custom mesh loading behavior, you can extend the `resolve_mesh_uri` function:

```rust
fn custom_resolve_mesh_uri(uri: &str) -> Option<String> {
    match uri {
        uri if uri.starts_with("custom://") => {
            // Handle custom URI scheme
            Some(format!("assets/custom/{}", &uri[9..]))
        },
        _ => resolve_mesh_uri(uri), // Fall back to default
    }
}
```

### Physics Customization

Modify physics properties by accessing SDF entities:

```rust
fn customize_physics(
    mut query: Query<(Entity, &SdfEntity, &mut RigidBody)>,
) {
    for (entity, sdf_entity, mut rigid_body) in query.iter_mut() {
        if sdf_entity.model_name == "special_robot" {
            *rigid_body = RigidBody::KinematicPositionBased;
        }
    }
}
```

### Event-Driven Loading

Use events for dynamic world loading:

```rust
fn request_world_load(
    mut events: EventWriter<LoadSdfWorldRequest>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyL) {
        events.send(LoadSdfWorldRequest {
            sdf_path: "assets/worlds/dynamic_world.sdf".to_string(),
            spawn_position: Vec3::new(10.0, 0.0, 0.0),
            spawn_rotation: Quat::IDENTITY,
        });
    }
}
```

## Troubleshooting

### Common Issues

1. **Mesh not loading**: Ensure mesh files are converted to GLTF/GLB format
2. **Physics not working**: Check that collision geometries are properly defined
3. **Materials not appearing**: Verify material properties and texture paths
4. **Performance issues**: Use LOD meshes for complex models

### Debug Information

Enable debug logging to see detailed SDF parsing information:

```rust
// Add to your Bevy app
.add_plugins(bevy::log::LogPlugin {
    level: bevy::log::Level::DEBUG,
    filter: "wgpu=error,bevy_render=info,bevy_ecs=trace".to_string(),
})
```

## Dependencies

- **Bevy**: 0.16+ (graphics engine)
- **bevy_rapier3d**: 0.30+ (physics)
- **quick-xml**: XML parsing for SDF files
- **regex**: Pattern matching for SDF parsing
- **trimesh**: Python mesh processing (for converter)
- **pygltflib**: GLTF/GLB file handling (for converter)
- **lxml**: Robust XML parsing (for converter)

## License

This SDF implementation follows the same license as the main project.

## Contributing

When adding new SDF features:
1. Update the appropriate parser in `sdf_loader.rs`
2. Add Bevy conversion logic in `sdf_world_loader.rs`
3. Test with example SDF files
4. Update this documentation

For mesh converter improvements:
1. Add new format support to `sdf_mesh_converter.py`
2. Handle new URI schemes in `resolve_mesh_path`
3. Test with various mesh formats
