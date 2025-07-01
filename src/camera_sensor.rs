use bevy::prelude::*;
use bevy::render::camera::{Camera, RenderTarget};
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::window::{Window, WindowPosition, WindowRef};

/// Camera intrinsic parameters based on pinhole camera model
/// Following OpenCV convention: camera matrix K = [[fx, 0, cx], [0, fy, cy], [0, 0, 1]]
#[derive(Component, Clone, Debug)]
pub struct CameraIntrinsics {
    /// Focal length in x direction (pixels)
    pub fx: f32,
    /// Focal length in y direction (pixels)  
    pub fy: f32,
    /// Principal point x coordinate (pixels)
    pub cx: f32,
    /// Principal point y coordinate (pixels)
    pub cy: f32,
    /// Image width (pixels)
    pub width: u32,
    /// Image height (pixels)
    pub height: u32,
}

impl Default for CameraIntrinsics {
    fn default() -> Self {
        Self {
            fx: 500.0,
            fy: 500.0,
            cx: 320.0,
            cy: 240.0,
            width: 640,
            height: 480,
        }
    }
}

impl CameraIntrinsics {
    /// Convert intrinsic parameters to Bevy's perspective projection
    pub fn to_perspective_projection(&self) -> Projection {
        // Calculate field of view from focal length
        // fov_y = 2 * atan(height / (2 * fy))
        let fov_y = 2.0 * (self.height as f32 / (2.0 * self.fy)).atan();
        
        Projection::Perspective(PerspectiveProjection {
            fov: fov_y,
            aspect_ratio: self.width as f32 / self.height as f32,
            near: 0.1,
            far: 100.0,
        })
    }
    
    /// Get the camera matrix as Mat3 (K matrix in OpenCV notation)
    pub fn camera_matrix(&self) -> Mat3 {
        Mat3::from_cols(
            Vec3::new(self.fx, 0.0, 0.0),
            Vec3::new(0.0, self.fy, 0.0),
            Vec3::new(self.cx, self.cy, 1.0),
        )
    }
}

/// Marker component for robot camera sensor
#[derive(Component)]
pub struct RobotCameraSensor;

/// Resource to track the camera preview window and render target
#[derive(Resource)]
pub struct CameraPreviewWindow {
    pub window_entity: Entity,
    pub image: Handle<Image>,
}

/// System to create the camera preview window
pub fn setup_camera_preview_window(mut commands: Commands) {
    // Create secondary window for camera preview
    let window_entity = commands.spawn(Window {
        resolution: (640.0, 480.0).into(),
        title: "Robot Camera View".into(),
        position: WindowPosition::Automatic,
        ..default()
    }).id();
    
    commands.insert_resource(CameraPreviewWindow {
        window_entity,
        image: Handle::default(),
    });
}

/// One-time setup system for robot camera sensor
pub fn setup_robot_camera_once(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    // Look for the robot chassis which has our Draggable marker
    chassis_query: Query<Entity, (With<crate::Draggable>, Without<RobotCameraSensor>)>,
    mut preview_window: ResMut<CameraPreviewWindow>,
) {
    // Find the chassis entity (which has the draggable component)
    if let Ok(chassis_entity) = chassis_query.single() {
        let intrinsics = CameraIntrinsics::default();
        
        // Create render target texture
        let size = Extent3d {
            width: intrinsics.width,
            height: intrinsics.height,
            depth_or_array_layers: 1,
        };
        
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };
        image.resize(size);
        
        let image_handle = images.add(image);
        
        // Add camera sensor to chassis with offset position and rotation
        commands.entity(chassis_entity).with_children(|parent| {
            parent.spawn((
                RobotCameraSensor,
                intrinsics.clone(),
                Camera3d::default(),
                intrinsics.to_perspective_projection(),
                Camera {
                    target: RenderTarget::Image(image_handle.clone().into()),
                    ..default()
                },
                // Camera positioned at front-top of robot, looking forward
                // Adjusted position to be more realistic for a TurtleBot4
                Transform::from_xyz(0.1, 0.15, 0.0)
                    .looking_at(Vec3::new(1.0, 0.0, 0.0), Vec3::Y),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        });
        
        // Update preview window resource with image handle
        preview_window.image = image_handle;
        
        info!("Robot camera sensor setup complete! Camera intrinsics: fx={}, fy={}, cx={}, cy={}", 
              intrinsics.fx, intrinsics.fy, intrinsics.cx, intrinsics.cy);
    } else {
        warn!("Could not find robot chassis to attach camera sensor");
    }
}

/// System to display camera feed in preview window
pub fn display_camera_preview(
    mut commands: Commands,
    preview_window: Res<CameraPreviewWindow>,
    camera_query: Query<&CameraIntrinsics, With<RobotCameraSensor>>,
    existing_preview: Query<Entity, With<CameraPreviewDisplay>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if preview_window.is_changed() && !preview_window.image.is_weak() {
        // Remove existing preview if any
        for entity in existing_preview.iter() {
            commands.entity(entity).despawn();
        }
        
        if camera_query.single().is_ok() {
            // Create a quad to display the camera feed
            commands.spawn((
                CameraPreviewDisplay,
                Mesh3d(meshes.add(Rectangle::new(2.0, 1.5))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(preview_window.image.clone()),
                    unlit: true,
                    ..default()
                })),
                Transform::from_xyz(0.0, 0.0, -1.0),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
            
            // Add camera for preview window
            commands.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.0, 1.0)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                Camera {
                    target: RenderTarget::Window(WindowRef::Entity(preview_window.window_entity)),
                    ..default()
                },
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        }
    }
}

/// Marker component for camera preview display
#[derive(Component)]
pub struct CameraPreviewDisplay;

/// System to update camera parameters during runtime
pub fn update_camera_intrinsics(
    mut camera_query: Query<(&mut CameraIntrinsics, &mut Projection), With<RobotCameraSensor>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if let Ok((mut intrinsics, mut projection)) = camera_query.single_mut() {
        let mut changed = false;
        
        // Allow runtime adjustment of camera parameters using number keys
        if keyboard.pressed(KeyCode::Equal) || keyboard.pressed(KeyCode::NumpadAdd) {
            intrinsics.fx += 10.0;
            intrinsics.fy += 10.0;
            changed = true;
        }
        if keyboard.pressed(KeyCode::Minus) || keyboard.pressed(KeyCode::NumpadSubtract) {
            intrinsics.fx = (intrinsics.fx - 10.0).max(100.0);
            intrinsics.fy = (intrinsics.fy - 10.0).max(100.0);
            changed = true;
        }
        
        // Principal point adjustments
        if keyboard.pressed(KeyCode::ArrowLeft) {
            intrinsics.cx = (intrinsics.cx - 5.0).max(0.0);
            changed = true;
        }
        if keyboard.pressed(KeyCode::ArrowRight) {
            intrinsics.cx = (intrinsics.cx + 5.0).min(intrinsics.width as f32);
            changed = true;
        }
        if keyboard.pressed(KeyCode::ArrowUp) {
            intrinsics.cy = (intrinsics.cy - 5.0).max(0.0);
            changed = true;
        }
        if keyboard.pressed(KeyCode::ArrowDown) {
            intrinsics.cy = (intrinsics.cy + 5.0).min(intrinsics.height as f32);
            changed = true;
        }
        
        // Update projection when intrinsics change
        if changed {
            *projection = intrinsics.to_perspective_projection();
            info!("Camera intrinsics updated: fx={:.1}, fy={:.1}, cx={:.1}, cy={:.1}", 
                  intrinsics.fx, intrinsics.fy, intrinsics.cx, intrinsics.cy);
        }
    }
}

/// System to log camera pose for debugging (extrinsic parameters)
pub fn debug_camera_pose(
    camera_query: Query<&GlobalTransform, (With<RobotCameraSensor>, Changed<GlobalTransform>)>,
) {
    for transform in camera_query.iter() {
        let translation = transform.translation();
        let rotation = transform.to_scale_rotation_translation().1;
        debug!("Robot Camera pose - Position: ({:.2}, {:.2}, {:.2}), Rotation: ({:.2}, {:.2}, {:.2}, {:.2})",
               translation.x, translation.y, translation.z,
               rotation.x, rotation.y, rotation.z, rotation.w);
    }
} 