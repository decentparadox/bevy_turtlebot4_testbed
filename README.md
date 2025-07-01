# Bevy TurtleBot4 Testbed

A robotics simulation testbed built with Bevy 0.16, featuring a TurtleBot4 robot with custom camera sensor implementation and improved interaction systems.

## Features

### 🤖 Robot Camera Sensor
- **Configurable camera intrinsics** - Adjust focal length, principal point, and other camera parameters
- **Real-time camera view** - Secondary window showing the robot's camera perspective
- **Realistic camera positioning** - Camera mounted on the front-top of the TurtleBot4
- **OpenCV-compatible parameters** - Camera matrix follows standard computer vision conventions

### 🎮 Improved Controls
- **Conflict-free interaction** - Camera controls automatically disable when dragging objects
- **3D object manipulation** - Click and drag the robot to apply physics forces
- **Orbit camera** - Right-click and drag to orbit around the scene
- **Pan camera** - Middle-click and drag to pan the view
- **Zoom** - Scroll wheel to zoom in/out

### 🔧 Camera Configuration
The robot camera sensor supports runtime configuration of intrinsic parameters:

#### Focal Length Adjustment
- `+` or `NumPad +` - Increase focal length (zoom in)
- `-` or `NumPad -` - Decrease focal length (zoom out)

#### Principal Point Adjustment
- `Arrow Keys` - Move the principal point (optical center) of the camera

#### Debug Controls
- `C` - Toggle camera controls on/off manually

## Camera Theory

The camera sensor implements a **pinhole camera model** with the following intrinsic parameters:

```
Camera Matrix K = [[fx,  0, cx],
                   [ 0, fy, cy],
                   [ 0,  0,  1]]
```

Where:
- `fx`, `fy` - Focal lengths in pixels (determines zoom/field of view)
- `cx`, `cy` - Principal point coordinates (optical center)

### Extrinsic Parameters
The camera's position and orientation relative to the robot chassis automatically update as the robot moves around the environment. This provides realistic camera motion behavior.

### Real-time Updates
All camera parameters can be adjusted during runtime, allowing for:
- Camera calibration experimentation
- Different lens simulation
- FOV adjustments for different scenarios

## Architecture

### Camera System Integration
- **Automatic conflict resolution** - Camera controls disable when objects are being dragged
- **Observer-based interaction** - Uses Bevy 0.16's built-in picking system with observers
- **Physics integration** - Dragging applies realistic forces through Rapier3D

### Custom Projection
The camera sensor converts intrinsic parameters to Bevy's perspective projection:
```rust
// Field of view calculated from focal length
let fov_y = 2.0 * (height / (2.0 * fy)).atan();
```

## Usage

1. **Run the simulation**:
   ```bash
   cargo run
   ```

2. **Interact with the robot**:
   - Click the robot to apply an upward impulse
   - Drag the robot to apply directional forces
   - Watch the robot camera view in the secondary window

3. **Control the main camera**:
   - Right-click and drag to orbit
   - Middle-click and drag to pan
   - Scroll to zoom

4. **Adjust camera parameters**:
   - Use `+`/`-` to change focal length
   - Use arrow keys to adjust principal point
   - Press `C` to manually toggle camera controls

## Implementation Details

### Camera Sensor Architecture
The robot camera sensor is implemented as a child entity of the robot chassis, ensuring proper transform inheritance. The camera renders to a texture that is displayed in a secondary window.

### Picking System
Uses Bevy 0.16's built-in picking system with mesh picking for 3D object interaction. The system properly handles:
- Multiple input devices
- Event bubbling
- Conflict resolution between different interaction modes

### Physics Integration
Drag forces are applied through Rapier3D's `ExternalImpulse` system, providing realistic physics-based interaction.

## Future Enhancements

- [ ] Camera parameter persistence
- [ ] Multiple camera sensors
- [ ] Image recording/capture
- [ ] Computer vision algorithm integration
- [ ] Robot motion planning integration
- [ ] Sensor noise simulation

## Requirements

- Rust 1.70+
- Bevy 0.16
- Rapier3D 0.30
- Compatible graphics drivers (OpenGL 3.3+ or Vulkan)

## License

This project is licensed under the MIT License.