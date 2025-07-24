use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::BufReader;
use bevy::prelude::*;

/// Parsed URDF visual element (minimal for now)
#[derive(Debug, Clone)]
pub struct UrdfVisual {
    pub link_name: String,
    // Add geometry/material fields as needed
}

/// Parsed URDF joint element (minimal for now)
#[derive(Debug, Clone)]
pub struct UrdfJoint {
    pub name: String,
    pub joint_type: String,
    pub parent: String,
    pub child: String,
}

/// Parsed URDF robot structure
#[derive(Debug)]
pub struct UrdfRobot {
    pub name: String,
    pub links: Vec<String>,
    pub joints: Vec<UrdfJoint>,
    pub visuals: Vec<UrdfVisual>,
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
                            visuals.push(UrdfVisual {
                                link_name: link_name.clone(),
                            });
                        }
                    }
                    b"joint" if in_robot => {
                        let mut name = String::new();
                        let mut joint_type = String::new();
                        let mut parent = String::new();
                        let mut child = String::new();
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"name" {
                                name = attr.unescape_value().unwrap_or_default().to_string();
                            }
                            if attr.key.as_ref() == b"type" {
                                joint_type = attr.unescape_value().unwrap_or_default().to_string();
                            }
                        }
                        // Parse parent/child from nested elements
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
                        joints.push(UrdfJoint { name, joint_type, parent, child });
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
    Ok(UrdfRobot { name: robot_name, links, joints, visuals })
}

/// Spawns a simple Bevy scene from a parsed URDF robot.
/// Each link is represented as a colored cube; joints are printed for now.
pub fn spawn_urdf_scene(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    urdf: &UrdfRobot,
) {
    let mut link_entities = std::collections::HashMap::new();
    for (i, link_name) in urdf.links.iter().enumerate() {
        // Spawn a colored cube for each link
        let color = Color::hsl((i as f32) * 360.0 / (urdf.links.len().max(1) as f32), 0.7, 0.5);
        let mesh_handle = meshes.add(Mesh::from(Cuboid::new(0.2, 0.2, 0.2)));
        let material_handle = materials.add(StandardMaterial {
            base_color: color,
            ..Default::default()
        });
        let entity = commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            Transform::from_xyz(i as f32 * 0.3, 0.2, 0.0),
            Name::new(link_name.clone()),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        )).id();
        link_entities.insert(link_name.clone(), entity);
    }
    // Print joint connections
    for joint in &urdf.joints {
        println!("Joint '{}' (type: {}) connects parent '{}' to child '{}'", joint.name, joint.joint_type, joint.parent, joint.child);
    }
} 