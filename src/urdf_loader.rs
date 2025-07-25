use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::BufReader;
use bevy::prelude::*;
use bevy_rapier3d::geometry::Collider;
use crate::RobotChassis;
use crate::robot_drag::DraggableRobot;

/// Parsed URDF visual element (minimal for now)
#[derive(Debug, Clone)]
pub enum UrdfGeometry {
    Box { size: [f32; 3] },
    Sphere { radius: f32 },
    Cylinder { radius: f32, length: f32 },
    // Mesh { filename: String }, // Optionally add mesh support
    Unknown,
}

#[derive(Debug, Clone)]
pub struct UrdfVisual {
    pub link_name: String,
    pub geometry: UrdfGeometry,
}

#[derive(Debug, Clone)]
pub struct UrdfCollision {
    pub link_name: String,
    pub geometry: UrdfGeometry,
}

#[derive(Debug, Clone, Default)]
pub struct UrdfOrigin {
    pub xyz: [f32; 3],
    pub rpy: [f32; 3],
}

/// Parsed URDF joint element (minimal for now)
#[derive(Debug, Clone)]
pub struct UrdfJoint {
    pub name: String,
    pub joint_type: String,
    pub parent: String,
    pub child: String,
    pub origin: UrdfOrigin,
}

/// Parsed URDF robot structure
#[derive(Debug)]
pub struct UrdfRobot {
    pub name: String,
    pub links: Vec<String>,
    pub joints: Vec<UrdfJoint>,
    pub visuals: Vec<UrdfVisual>,
    pub collisions: Vec<UrdfCollision>,
}

/// Loads a URDF file and returns the robot name, link names, joints, and visuals.
pub fn load_urdf(path: &str) -> Result<UrdfRobot, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut reader = Reader::from_reader(BufReader::new(file));
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut robot_name = String::new();
    let mut links = Vec::new();
    let mut joints = Vec::new();
    let mut visuals = Vec::new();
    let mut collisions = Vec::new();
    let mut in_robot = false;
    let mut current_link: Option<String> = None;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                match e.name().as_ref() {
                    b"robot" => {
                        in_robot = true;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"name" {
                                robot_name = attr.unescape_value().unwrap_or_default().to_string();
                            }
                        }
                    }
                    b"link" if in_robot => {
                        let mut link_name = None;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"name" {
                                let name = attr.unescape_value().unwrap_or_default().to_string();
                                links.push(name.clone());
                                link_name = Some(name);
                            }
                        }
                        current_link = link_name;
                    }
                    b"visual" if in_robot => {
                        if let Some(ref link_name) = current_link {
                            let geometry = parse_geometry(&mut reader, &mut buf);
                            visuals.push(UrdfVisual {
                                link_name: link_name.clone(),
                                geometry,
                            });
                        }
                    }
                    b"collision" if in_robot => {
                        if let Some(ref link_name) = current_link {
                            let geometry = parse_geometry(&mut reader, &mut buf);
                            collisions.push(UrdfCollision {
                                link_name: link_name.clone(),
                                geometry,
                            });
                        }
                    }
                    b"joint" if in_robot => {
                        let mut name = String::new();
                        let mut joint_type = String::new();
                        let mut parent = String::new();
                        let mut child = String::new();
                        let mut origin = UrdfOrigin::default();
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"name" {
                                name = attr.unescape_value().unwrap_or_default().to_string();
                            }
                            if attr.key.as_ref() == b"type" {
                                joint_type = attr.unescape_value().unwrap_or_default().to_string();
                            }
                        }
                        // Parse parent/child/origin from nested elements
                        let mut joint_buf = Vec::new();
                        loop {
                            match reader.read_event_into(&mut joint_buf) {
                                Ok(Event::Start(ref je)) | Ok(Event::Empty(ref je)) => {
                                    match je.name().as_ref() {
                                        b"parent" => {
                                            for attr in je.attributes().flatten() {
                                                if attr.key.as_ref() == b"link" {
                                                    parent = attr.unescape_value().unwrap_or_default().to_string();
                                                }
                                            }
                                        }
                                        b"child" => {
                                            for attr in je.attributes().flatten() {
                                                if attr.key.as_ref() == b"link" {
                                                    child = attr.unescape_value().unwrap_or_default().to_string();
                                                }
                                            }
                                        }
                                        b"origin" => {
                                            for attr in je.attributes().flatten() {
                                                if attr.key.as_ref() == b"xyz" {
                                                    origin.xyz = parse_xyz(&attr.unescape_value().unwrap_or_default());
                                                }
                                                if attr.key.as_ref() == b"rpy" {
                                                    origin.rpy = parse_xyz(&attr.unescape_value().unwrap_or_default());
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                Ok(Event::End(ref je)) if je.name().as_ref() == b"joint" => break,
                                Ok(Event::Eof) => break,
                                Ok(_) => {},
                                Err(_) => break,
                            }
                            joint_buf.clear();
                        }
                        joints.push(UrdfJoint { name, joint_type, parent, child, origin });
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {},
            Err(e) => return Err(format!("XML error: {}", e)),
        }
        buf.clear();
    }
    if robot_name.is_empty() {
        return Err("No <robot> element with name attribute found".to_string());
    }
    Ok(UrdfRobot { name: robot_name, links, joints, visuals, collisions })
}

fn parse_geometry<R: std::io::BufRead>(reader: &mut Reader<R>, buf: &mut Vec<u8>) -> UrdfGeometry {
    let mut geometry = UrdfGeometry::Unknown;
    let mut depth = 0;
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                match e.name().as_ref() {
                    b"box" => {
                        let mut size = [1.0, 1.0, 1.0];
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"size" {
                                size = parse_xyz(&attr.unescape_value().unwrap_or_default());
                            }
                        }
                        geometry = UrdfGeometry::Box { size };
                    }
                    b"sphere" => {
                        let mut radius = 1.0;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"radius" {
                                radius = attr.unescape_value().unwrap_or_default().parse().unwrap_or(1.0);
                            }
                        }
                        geometry = UrdfGeometry::Sphere { radius };
                    }
                    b"cylinder" => {
                        let mut radius = 1.0;
                        let mut length = 1.0;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"radius" {
                                radius = attr.unescape_value().unwrap_or_default().parse().unwrap_or(1.0);
                            }
                            if attr.key.as_ref() == b"length" {
                                length = attr.unescape_value().unwrap_or_default().parse().unwrap_or(1.0);
                            }
                        }
                        geometry = UrdfGeometry::Cylinder { radius, length };
                    }
                    // Optionally add mesh support here
                    _ => {}
                }
                if !matches!(e.name().as_ref(), b"geometry") {
                    depth += 1;
                }
            }
            Ok(Event::End(ref e)) => {
                if depth == 0 && e.name().as_ref() == b"geometry" {
                    break;
                }
                if depth > 0 {
                    depth -= 1;
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {},
            Err(_) => break,
        }
        buf.clear();
    }
    geometry
}

fn parse_xyz(s: &str) -> [f32; 3] {
    let mut out = [0.0; 3];
    for (i, v) in s.split_whitespace().enumerate().take(3) {
        out[i] = v.parse().unwrap_or(0.0);
    }
    out
}

/// Spawns a simple Bevy scene from a parsed URDF robot.
/// Each link is represented as a colored cube; joints are printed for now.
pub fn spawn_urdf_scene(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    urdf: &UrdfRobot,
) {
    use bevy::math::{Quat, Vec3};
    use std::collections::HashMap;
    // Build parent->children map
    let mut children_map: HashMap<String, Vec<&UrdfJoint>> = HashMap::new();
    let mut joint_map: HashMap<String, &UrdfJoint> = HashMap::new();
    for joint in &urdf.joints {
        children_map.entry(joint.parent.clone()).or_default().push(joint);
        joint_map.insert(joint.child.clone(), joint);
    }
    // Find root links (not a child in any joint)
    let mut all_children: std::collections::HashSet<&String> = urdf.joints.iter().map(|j| &j.child).collect();
    let root_links: Vec<&String> = urdf.links.iter().filter(|l| !all_children.contains(l)).collect();
    // Recursively spawn links
    for root in root_links {
        spawn_link_recursive(
            commands,
            meshes,
            materials,
            urdf,
            root,
            &children_map,
            &joint_map,
            Transform::default(),
        );
    }
    // Print joint connections
    for joint in &urdf.joints {
        println!("Joint '{}' (type: {}) connects parent '{}' to child '{}' at origin {:?}", joint.name, joint.joint_type, joint.parent, joint.child, joint.origin);
    }
}

fn spawn_link_recursive(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    urdf: &UrdfRobot,
    link_name: &str,
    children_map: &std::collections::HashMap<String, Vec<&UrdfJoint>>,
    joint_map: &std::collections::HashMap<String, &UrdfJoint>,
    parent_transform: Transform,
) {
    // Find the first visual for this link, or fallback to a colored box
    let visual = urdf.visuals.iter().find(|v| v.link_name == link_name);
    let (mesh_handle, material_handle) = match &visual.map(|v| &v.geometry) {
        Some(UrdfGeometry::Box { size }) => {
            let mesh = meshes.add(Mesh::from(Cuboid::new(size[0], size[1], size[2])));
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(size[0] / 2.0, size[1] / 2.0, size[2] / 2.0),
                ..Default::default()
            });
            (mesh, mat)
        }
        Some(UrdfGeometry::Sphere { radius }) => {
            let mesh = meshes.add(Mesh::from(Sphere { radius: *radius, ..Default::default() }));
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.5, 1.0),
                ..Default::default()
            });
            (mesh, mat)
        }
        Some(UrdfGeometry::Cylinder { radius, length }) => {
            let mesh = meshes.add(Mesh::from(Cylinder { radius: *radius, half_height: *length / 2.0, ..Default::default() }));
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.5, 0.5),
                ..Default::default()
            });
            (mesh, mat)
        }
        _ => {
            // fallback: colored cube
            let i = urdf.links.iter().position(|l| l == link_name).unwrap_or(0);
            let color = Color::hsl((i as f32) * 360.0 / (urdf.links.len().max(1) as f32), 0.7, 0.5);
            let mesh = meshes.add(Mesh::from(Cuboid::new(0.2, 0.2, 0.2)));
            let mat = materials.add(StandardMaterial {
                base_color: color,
                ..Default::default()
            });
            (mesh, mat)
        }
    };
    // Find the first collision for this link
    let collider = urdf.collisions.iter().find(|c| c.link_name == link_name).map(|c| match &c.geometry {
        UrdfGeometry::Box { size } => Collider::cuboid(size[0] / 2.0, size[1] / 2.0, size[2] / 2.0),
        UrdfGeometry::Sphere { radius } => Collider::ball(*radius),
        UrdfGeometry::Cylinder { radius, length } => Collider::cylinder(*length / 2.0, *radius),
        _ => Collider::cuboid(0.1, 0.1, 0.1),
    });
    let mut entity_cmd = commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        parent_transform,
        Name::new(link_name.to_string()),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        DraggableRobot,
        RobotChassis,
    ));
    if let Some(collider) = collider {
        entity_cmd.insert(collider);
    }
    let entity = entity_cmd.id();
    // Recurse for children
    if let Some(joints) = children_map.get(link_name) {
        for joint in joints {
            let t = joint_to_transform(&joint.origin);
            let child_transform = parent_transform.mul_transform(t);
            spawn_link_recursive(
                commands,
                meshes,
                materials,
                urdf,
                &joint.child,
                children_map,
                joint_map,
                child_transform,
            );
        }
    }
}

fn joint_to_transform(origin: &UrdfOrigin) -> Transform {
    let translation = Vec3::from(origin.xyz);
    let (r, p, y) = (origin.rpy[0], origin.rpy[1], origin.rpy[2]);
    let rotation = Quat::from_euler(EulerRot::XYZ, r, p, y);
    Transform::from_translation(translation).with_rotation(rotation)
} 