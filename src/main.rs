use bevy::prelude::*;
use bevy::render::camera::{CameraProjection, RenderTarget, SubCameraView};
use bevy::window::{Window, WindowPosition, WindowRef};

use bevy_rapier3d::{
    dynamics::Velocity,
    geometry::Group,
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};

#[derive(Debug, Clone)]
pub struct ObliquePerspectiveProjection {
    pub horizontal_obliqueness: f32,
    pub vertical_obliqueness: f32,
    pub perspective: PerspectiveProjection,
}

/// Implement the [`CameraProjection`] trait for our custom projection:
impl CameraProjection for ObliquePerspectiveProjection {
    fn get_clip_from_view(&self) -> Mat4 {
        let mut mat = self.perspective.get_clip_from_view();
        mat.col_mut(2)[0] = self.horizontal_obliqueness;
        mat.col_mut(2)[1] = self.vertical_obliqueness;
        mat
    }

    fn get_clip_from_view_for_sub(&self, sub_view: &SubCameraView) -> Mat4 {
        let mut mat = self.perspective.get_clip_from_view_for_sub(sub_view);
        mat.col_mut(2)[0] = self.horizontal_obliqueness;
        mat.col_mut(2)[1] = self.vertical_obliqueness;
        mat
    }

    fn update(&mut self, width: f32, height: f32) {
        self.perspective.update(width, height);
    }

    fn far(&self) -> f32 {
        self.perspective.far
    }

    fn get_frustum_corners(&self, z_near: f32, z_far: f32) -> [Vec3A; 8] {
        self.perspective.get_frustum_corners(z_near, z_far)
    }
}

/// Resource to track the custom projection window
#[derive(Resource)]
struct CustomProjectionWindow {
    window_entity: Entity,
}

/// System to create the custom projection window
fn setup_custom_projection_window(mut commands: Commands) {
    // Create secondary window for custom projection
    let window_entity = commands
        .spawn(Window {
            resolution: (800.0, 600.0).into(),
            title: "Robot First-Person View (Oblique Projection)".into(),
            position: WindowPosition::Automatic,
            ..default()
        })
        .id();

    commands.insert_resource(CustomProjectionWindow { window_entity });

    info!("Created secondary window for robot first-person perspective view");
}

/// System to setup the custom projection camera after window is created
fn setup_custom_projection_camera(
    mut commands: Commands,
    custom_window: Res<CustomProjectionWindow>,
) {
    // Spawn camera with custom oblique perspective projection rendering to secondary window
    commands.spawn((
        Camera3d::default(),
        // Use our custom projection:
        Projection::custom(ObliquePerspectiveProjection {
            horizontal_obliqueness: 0.0,
            vertical_obliqueness: 0.0,
            perspective: PerspectiveProjection::default(),
        }),
        Camera {
            target: RenderTarget::Window(WindowRef::Entity(custom_window.window_entity)),
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.3)), // Dark blue background
            order: 2,
            is_active: true, // Make sure camera is active
            ..default()
        },
        // Start camera at robot's initial position (matches robot spawn position)
        Transform::from_xyz(0.0, 0.8, 0.0).looking_at(Vec3::new(0.0, 0.5, -1.0), Vec3::Y),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        ObliqueProjectionController, // Marker component for controlling the projection
    ));

    info!("Robot first-person camera setup complete - follows robot position and rotation");
}

/// Marker component for the oblique projection controller
#[derive(Component)]
pub struct ObliqueProjectionController;

/// Marker component for the main robot chassis
#[derive(Component)]
pub struct RobotChassis;

/// System to update oblique projection based on robot state
#[allow(clippy::type_complexity)]
fn update_projection_from_robot(
    robot_query: Query<(&Transform, Option<&Velocity>), With<RobotChassis>>,
    mut projection_query: Query<
        (&mut Projection, &mut Transform),
        (With<ObliqueProjectionController>, Without<RobotChassis>),
    >,
) {
    if let Ok((robot_transform, robot_velocity)) = robot_query.single() {
        if let Ok((mut projection, mut camera_transform)) = projection_query.single_mut() {
            // Update camera position to follow robot (with slight height offset for better view)
            let camera_height_offset = 0.3; // Adjust this to change camera height above robot
            let camera_forward_offset = 0.1; // Slightly forward from robot center

            // Position camera at robot's position with offsets
            let robot_forward = robot_transform.forward();
            camera_transform.translation = robot_transform.translation
                + Vec3::new(0.0, camera_height_offset, 0.0) // Height offset
                + robot_forward * camera_forward_offset; // Forward offset

            // Make camera look in the same direction as robot
            let target_point = camera_transform.translation + robot_forward * 5.0; // Look ahead
            camera_transform.look_at(target_point, Vec3::Y);

            if let Projection::Custom(custom_projection) = projection.as_mut() {
                if let Some(oblique) =
                    custom_projection.downcast_mut::<ObliquePerspectiveProjection>()
                {
                    // Get robot's forward direction (normalized)
                    let robot_forward = robot_transform.forward().normalize();

                    // Calculate horizontal obliqueness based on robot's rotation around Y axis
                    // Using the robot's forward direction projected onto XZ plane
                    let forward_xz = Vec3::new(robot_forward.x, 0.0, robot_forward.z).normalize();
                    oblique.horizontal_obliqueness = forward_xz.x * 0.3; // Reduced effect for robot view

                    // Calculate vertical obliqueness based on robot's velocity magnitude
                    let velocity_magnitude = if let Some(velocity) = robot_velocity {
                        velocity.linvel.length()
                    } else {
                        0.0
                    };

                    // Use velocity to control vertical obliqueness (reduced for robot view)
                    oblique.vertical_obliqueness = (velocity_magnitude * 0.05).clamp(-0.4, 0.4);

                    // Optional: Add some pitch influence (reduced)
                    let pitch_influence = robot_transform.forward().y * 0.1;
                    oblique.vertical_obliqueness += pitch_influence;

                    // Clamp values to reasonable ranges (smaller range for robot view)
                    oblique.horizontal_obliqueness =
                        oblique.horizontal_obliqueness.clamp(-0.5, 0.5);
                    oblique.vertical_obliqueness = oblique.vertical_obliqueness.clamp(-0.5, 0.5);
                }
            }
        }
    }
}

mod camera;
mod keyboard_controls;
mod lidar;
mod robot_drag;
mod stl_loader;
mod turtlebot4;
mod urdf_loader;
mod world_builder;

#[cfg(test)]
mod tests;

pub const STATIC_GROUP: Group = Group::GROUP_1;
pub const CHASSIS_INTERNAL_GROUP: Group = Group::GROUP_2;
pub const CHASSIS_GROUP: Group = Group::GROUP_3;

fn print_urdf_info() {
    match urdf_loader::load_urdf("assets/robots/urdf/sample.urdf") {
        Ok(robot) => {
            println!("URDF loaded: robot name = {}", robot.name);
            println!("Links: {:?}", robot.links);
            println!("Joints: {:?}", robot.joints);
            println!("Visuals: {:?}", robot.visuals);
        },
        Err(e) => println!("Failed to load URDF: {}", e),
    }
}

fn spawn_urdf_scene_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok(robot) = urdf_loader::load_urdf("assets/robots/urdf/sample.urdf") {
        urdf_loader::spawn_urdf_scene(&mut commands, &mut meshes, &mut materials, &robot);
    }
}

pub fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.98, 0.92, 0.84)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 500.0,
            affects_lightmapped_meshes: true,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(lidar::LidarPlugin)
        .add_plugins(robot_drag::RobotDragPlugin)
        .init_asset_loader::<stl_loader::StlAssetLoader>()
        .add_systems(Startup, (setup, setup_custom_projection_window))
        .add_systems(Update, robot_drag::make_robot_draggable)
        .add_systems(
            Update,
            (
                camera::update_camera_system,
                camera::accumulate_mouse_events_system,
                camera::update_camera_focus_on_robot,
                keyboard_controls::control_robot_movement,
                update_projection_from_robot,
                keyboard_controls::display_robot_controls_info,
                keyboard_controls::manual_adjust_oblique_projection,
                keyboard_controls::toggle_lidar_visualization,
                render_origin,
            ),
        )
        .add_systems(PostStartup, setup_custom_projection_camera)
        .add_systems(Startup, (print_urdf_info, spawn_urdf_scene_system))
        .run();
}

fn render_origin(mut gizmos: Gizmos) {
    gizmos.line(Vec3::ZERO, Vec3::X, Color::srgb(1.0, 0.0, 0.0));
    gizmos.line(Vec3::ZERO, Vec3::Y, Color::srgb(0.0, 1.0, 0.0));
    gizmos.line(Vec3::ZERO, Vec3::Z, Color::srgb(0.0, 0.0, 1.0));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let translation = Vec3::new(1.0, 2.0, 2.0);
    let focus = Vec3::ZERO;
    let transform = Transform::from_translation(translation).looking_at(focus, Vec3::Y);

    commands
        .spawn((
            Camera3d::default(),
            transform,
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Camera {
                order: 1,
                is_active: true,
                ..default()
            },
        ))
        .insert(camera::PanOrbitCamera {
            focus,
            radius: translation.length(),
            ..default()
        })
        .with_children(|commands| {
            commands.spawn((
                DirectionalLight {
                    shadows_enabled: false,
                    illuminance: 1000.0,
                    ..default()
                },
                Transform::from_xyz(-2.5, 2.5, 2.5).looking_at(Vec3::ZERO, Vec3::Y),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        });

    // Point lights
    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(5.0, 5.0, 0.0),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(-5.0, 5.0, 0.0),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(0.0, 5.0, 5.0),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(0.0, 5.0, -5.0),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));

    // Spawn the arena with walls and obstacles
    world_builder::spawn_simple_arena(&mut commands, &mut meshes, &mut materials);
    
    // Add a complex obstacle to demonstrate visual vs physics mesh difference
    world_builder::spawn_complex_obstacle(&mut commands, &mut meshes, &mut materials, 
                                         Vec3::new(0.0, 0.0, -2.0), 
                                         "Complex Obstacle");

    // Robot
    turtlebot4::spawn(
        &mut commands,
        &asset_server,
        &Transform::from_xyz(0.0, 0.5, 0.0),
    );
}
