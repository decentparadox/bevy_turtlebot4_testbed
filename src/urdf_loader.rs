use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::BufReader;
use bevy::prelude::*;
use bevy_rapier3d::geometry::Collider;
use crate::RobotChassis;
use crate::robot_drag::DraggableRobot;
use crate::stl_loader;
use std::path::PathBuf;

/// Parsed URDF visual element (minimal for now)
#[derive(Debug, Clone)]
pub enum UrdfGeometry {
    Box { size: [f32; 3] },
    Sphere { radius: f32 },
    Cylinder { radius: f32, length: f32 },
    Mesh { filename: String }, // Add mesh support
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
    let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    
    let mut robot_name = String::new();
    let mut links = Vec::new();
    let mut joints = Vec::new();
    let mut visuals = Vec::new();
    let mut collisions = Vec::new();
    
    // Extract robot name
    if let Some(cap) = regex::Regex::new(r#"<robot\s+name="([^"]+)""#).unwrap().captures(&content) {
        robot_name = cap[1].to_string();
    }
    
    // Extract all links
    let link_regex = regex::Regex::new(r#"<link\s+name="([^"]+)""#).unwrap();
    for cap in link_regex.captures_iter(&content) {
        links.push(cap[1].to_string());
    }
    
    // Extract all joints
    let joint_regex = regex::Regex::new(r#"<joint\s+name="([^"]+)"\s+type="([^"]+)""#).unwrap();
    for cap in joint_regex.captures_iter(&content) {
        let name = cap[1].to_string();
        let joint_type = cap[2].to_string();
        
        // Find parent and child for this joint
        let joint_section = extract_joint_section(&content, &name);
        println!("Joint '{}' section: {}", name, joint_section);
        
        let parent = extract_parent_link(&joint_section);
        let child = extract_child_link(&joint_section);
        let origin = extract_origin(&joint_section);
        
        println!("  Parent: '{}', Child: '{}', Origin: {:?}", parent, child, origin);
        
        joints.push(UrdfJoint {
            name,
            joint_type,
            parent,
            child,
            origin,
        });
    }
    
    // Extract visuals and collisions for each link
    for link_name in &links {
        let link_section = extract_link_section(&content, link_name);
        println!("Link '{}' section length: {}", link_name, link_section.len());
        if link_section.len() < 200 {
            println!("Link '{}' section: {}", link_name, link_section);
        } else {
            println!("Link '{}' section (first 200 chars): {}", link_name, &link_section[..200.min(link_section.len())]);
        }
        
        // Extract visual
        if let Some(visual_geometry) = extract_geometry_from_section(&link_section, "visual") {
            println!("  Found visual geometry: {:?}", visual_geometry);
            visuals.push(UrdfVisual {
                link_name: link_name.clone(),
                geometry: visual_geometry,
            });
        } else {
            println!("  No visual geometry found for link: {}", link_name);
        }
        
        // Extract collision
        if let Some(collision_geometry) = extract_geometry_from_section(&link_section, "collision") {
            println!("  Found collision geometry: {:?}", collision_geometry);
            collisions.push(UrdfCollision {
                link_name: link_name.clone(),
                geometry: collision_geometry,
            });
        } else {
            println!("  No collision geometry found for link: {}", link_name);
        }
    }
    

    
    if robot_name.is_empty() {
        return Err("No <robot> element with name attribute found".to_string());
    }
    
    println!("URDF loaded successfully:");
    println!("  Robot name: {}", robot_name);
    println!("  Links found: {}", links.len());
    println!("  Joints found: {}", joints.len());
    println!("  Visuals found: {}", visuals.len());
    println!("  Collisions found: {}", collisions.len());
    println!("  Links: {:?}", links);
    
    Ok(UrdfRobot { name: robot_name, links, joints, visuals, collisions })
}

fn extract_joint_section(content: &str, joint_name: &str) -> String {
    // Use regex to find the joint tag that spans multiple lines
    let joint_pattern = format!(r#"<joint\s*\n\s*name="{}""#, joint_name);
    if let Some(cap) = regex::Regex::new(&joint_pattern).unwrap().captures(content) {
        let start = cap.get(0).unwrap().start();
        let mut depth = 0;
        let mut end = start;
        for (i, ch) in content[start..].char_indices() {
            if ch == '<' {
                let tag_start = start + i;
                let tag_end = content[tag_start..].find('>').unwrap_or(0) + tag_start;
                let tag = &content[tag_start..tag_end];
                
                if tag.starts_with("<joint") {
                    depth += 1;
                } else if tag.starts_with("</joint") {
                    depth -= 1;
                    if depth == 0 {
                        end = tag_end;
                        break;
                    }
                }
            }
        }
        return content[start..end].to_string();
    }
    
    // Fallback: try simpler pattern without newlines
    let simple_pattern = format!(r#"<joint\s+name="{}""#, joint_name);
    if let Some(start) = content.find(&simple_pattern) {
        let mut depth = 0;
        let mut end = start;
        for (i, ch) in content[start..].char_indices() {
            if ch == '<' {
                let tag_start = start + i;
                let tag_end = content[tag_start..].find('>').unwrap_or(0) + tag_start;
                let tag = &content[tag_start..tag_end];
                
                if tag.starts_with("<joint") {
                    depth += 1;
                } else if tag.starts_with("</joint") {
                    depth -= 1;
                    if depth == 0 {
                        end = tag_end;
                        break;
                    }
                }
            }
        }
        return content[start..end].to_string();
    }
    
    String::new()
}

fn extract_link_section(content: &str, link_name: &str) -> String {
    // Use regex to find the link tag that spans multiple lines
    let link_pattern = format!(r#"<link\s*\n\s*name="{}""#, link_name);
    if let Some(cap) = regex::Regex::new(&link_pattern).unwrap().captures(content) {
        let start = cap.get(0).unwrap().start();
        let mut depth = 0;
        let mut end = start;
        for (i, ch) in content[start..].char_indices() {
            if ch == '<' {
                let tag_start = start + i;
                let tag_end = content[tag_start..].find('>').unwrap_or(0) + tag_start;
                let tag = &content[tag_start..tag_end];
                
                if tag.starts_with("<link") {
                    depth += 1;
                } else if tag.starts_with("</link") {
                    depth -= 1;
                    if depth == 0 {
                        end = tag_end;
                        break;
                    }
                }
            }
        }
        return content[start..end].to_string();
    }
    
    // Fallback: try simpler pattern without newlines
    let simple_pattern = format!(r#"<link\s+name="{}""#, link_name);
    if let Some(start) = content.find(&simple_pattern) {
        let mut depth = 0;
        let mut end = start;
        for (i, ch) in content[start..].char_indices() {
            if ch == '<' {
                let tag_start = start + i;
                let tag_end = content[tag_start..].find('>').unwrap_or(0) + tag_start;
                let tag = &content[tag_start..tag_end];
                
                if tag.starts_with("<link") {
                    depth += 1;
                } else if tag.starts_with("</link") {
                    depth -= 1;
                    if depth == 0 {
                        end = tag_end;
                        break;
                    }
                }
            }
        }
        return content[start..end].to_string();
    }
    
    String::new()
}

fn extract_parent_link(joint_section: &str) -> String {
    // Handle multi-line parent tags with newlines and spaces
    if let Some(cap) = regex::Regex::new(r#"<parent\s*\n\s*link="([^"]+)"\s*/>"#).unwrap().captures(joint_section) {
        cap[1].to_string()
    } else if let Some(cap) = regex::Regex::new(r#"<parent\s+link="([^"]+)"\s*/>"#).unwrap().captures(joint_section) {
        cap[1].to_string()
    } else if let Some(cap) = regex::Regex::new(r#"<parent\s*\n\s*link="([^"]+)""#).unwrap().captures(joint_section) {
        cap[1].to_string()
    } else if let Some(cap) = regex::Regex::new(r#"<parent\s+link="([^"]+)""#).unwrap().captures(joint_section) {
        cap[1].to_string()
    } else {
        String::new()
    }
}

fn extract_child_link(joint_section: &str) -> String {
    // Handle multi-line child tags with newlines and spaces
    if let Some(cap) = regex::Regex::new(r#"<child\s*\n\s*link="([^"]+)"\s*/>"#).unwrap().captures(joint_section) {
        cap[1].to_string()
    } else if let Some(cap) = regex::Regex::new(r#"<child\s+link="([^"]+)"\s*/>"#).unwrap().captures(joint_section) {
        cap[1].to_string()
    } else if let Some(cap) = regex::Regex::new(r#"<child\s*\n\s*link="([^"]+)""#).unwrap().captures(joint_section) {
        cap[1].to_string()
    } else if let Some(cap) = regex::Regex::new(r#"<child\s+link="([^"]+)""#).unwrap().captures(joint_section) {
        cap[1].to_string()
    } else {
        String::new()
    }
}

fn extract_origin(joint_section: &str) -> UrdfOrigin {
    let mut origin = UrdfOrigin::default();
    
    // Try to match origin with both xyz and rpy, handling multi-line format
    if let Some(cap) = regex::Regex::new(r#"<origin\s*\n\s*xyz="([^"]+)"\s*\n\s*rpy="([^"]+)""#).unwrap().captures(joint_section) {
        origin.xyz = parse_xyz(&cap[1]);
        origin.rpy = parse_xyz(&cap[2]);
    } else if let Some(cap) = regex::Regex::new(r#"<origin\s*\n\s*rpy="([^"]+)"\s*\n\s*xyz="([^"]+)""#).unwrap().captures(joint_section) {
        origin.rpy = parse_xyz(&cap[1]);
        origin.xyz = parse_xyz(&cap[2]);
    } else if let Some(cap) = regex::Regex::new(r#"<origin\s+xyz="([^"]+)"\s+rpy="([^"]+)""#).unwrap().captures(joint_section) {
        origin.xyz = parse_xyz(&cap[1]);
        origin.rpy = parse_xyz(&cap[2]);
    } else if let Some(cap) = regex::Regex::new(r#"<origin\s+rpy="([^"]+)"\s+xyz="([^"]+)""#).unwrap().captures(joint_section) {
        origin.rpy = parse_xyz(&cap[1]);
        origin.xyz = parse_xyz(&cap[2]);
    } else if let Some(cap) = regex::Regex::new(r#"<origin\s*\n\s*xyz="([^"]+)""#).unwrap().captures(joint_section) {
        origin.xyz = parse_xyz(&cap[1]);
    } else if let Some(cap) = regex::Regex::new(r#"<origin\s+xyz="([^"]+)""#).unwrap().captures(joint_section) {
        origin.xyz = parse_xyz(&cap[1]);
    } else if let Some(cap) = regex::Regex::new(r#"<origin\s*\n\s*rpy="([^"]+)""#).unwrap().captures(joint_section) {
        origin.rpy = parse_xyz(&cap[1]);
    } else if let Some(cap) = regex::Regex::new(r#"<origin\s+rpy="([^"]+)""#).unwrap().captures(joint_section) {
        origin.rpy = parse_xyz(&cap[1]);
    }
    
    origin
}

fn extract_geometry_from_section(section: &str, element_type: &str) -> Option<UrdfGeometry> {
    println!("  Looking for {} section in: {}", element_type, section);
    
    // Handle both <visual> and <visual> tags with potential whitespace
    let start_patterns = [
        format!("<{}>", element_type),
        format!("<{} ", element_type),
    ];
    
    for start_pattern in &start_patterns {
        println!("  Trying pattern: '{}'", start_pattern);
        if let Some(start) = section.find(start_pattern) {
            println!("  Found {} start at position {} with pattern '{}'", element_type, start, start_pattern);
            let mut depth = 0;
            let mut end = start;
            for (i, ch) in section[start..].char_indices() {
                if ch == '<' {
                    let tag_start = start + i;
                    let tag_end = section[tag_start..].find('>').unwrap_or(0) + tag_start;
                    let tag = &section[tag_start..tag_end];
                    
                    if tag.starts_with(&format!("<{}", element_type)) {
                        depth += 1;
                        println!("    Found opening {} tag: '{}', depth: {}", element_type, tag, depth);
                    } else if tag.starts_with(&format!("</{}", element_type)) {
                        depth -= 1;
                        println!("    Found closing {} tag: '{}', depth now: {}", element_type, tag, depth);
                        if depth == 0 {
                            end = tag_end;
                            println!("    {} section complete, ending at position {}", element_type, end);
                            break;
                        }
                    }
                }
            }
            let element_section = &section[start..end];
            println!("  Extracted {} section: {}", element_type, element_section);
            
            // Extract geometry from the element section
            if let Some(geometry_section) = extract_geometry_section(element_section) {
                println!("  Found geometry section: {}", geometry_section);
                return Some(parse_geometry_from_string(geometry_section));
            } else {
                println!("  No geometry section found in {}", element_type);
            }
        } else {
            println!("  Pattern '{}' not found", start_pattern);
        }
    }
    println!("  No {} section found", element_type);
    None
}

fn extract_geometry_section(element_section: &str) -> Option<String> {
    println!("    Looking for geometry in element section: {}", element_section);
    
    // Handle both <geometry> and <geometry> tags with potential whitespace
    let start_patterns = ["<geometry>", "<geometry "];
    
    for start_pattern in &start_patterns {
        println!("    Trying pattern: '{}'", start_pattern);
        if let Some(start) = element_section.find(start_pattern) {
            println!("    Found geometry start at position {} with pattern '{}'", start, start_pattern);
            let mut depth = 0;
            let mut end = start;
            for (i, ch) in element_section[start..].char_indices() {
                if ch == '<' {
                    let tag_start = start + i;
                    let tag_end = element_section[tag_start..].find('>').unwrap_or(0) + tag_start;
                    let tag = &element_section[tag_start..tag_end];
                    
                    if tag.starts_with("<geometry") {
                        depth += 1;
                        println!("      Found opening geometry tag: '{}', depth: {}", tag, depth);
                    } else if tag.starts_with("</geometry") {
                        depth -= 1;
                        println!("      Found closing geometry tag: '{}', depth now: {}", tag, depth);
                        if depth == 0 {
                            end = tag_end;
                            println!("      Geometry section complete, ending at position {}", end);
                            break;
                        }
                    }
                }
            }
            let geometry_section = element_section[start..end].to_string();
            println!("    Extracted geometry section: {}", geometry_section);
            return Some(geometry_section);
        } else {
            println!("    Pattern '{}' not found", start_pattern);
        }
    }
    println!("    No geometry section found in element");
    None
}

fn parse_geometry_from_string(geometry_section: String) -> UrdfGeometry {
    println!("Parsing geometry section: '{}'", geometry_section);
    println!("  Geometry section length: {}", geometry_section.len());
    println!("  Contains '<box': {}", geometry_section.contains("<box"));
    println!("  Contains '<sphere': {}", geometry_section.contains("<sphere"));
    println!("  Contains '<cylinder': {}", geometry_section.contains("<cylinder"));
    println!("  Contains '<mesh': {}", geometry_section.contains("<mesh"));
    
    if geometry_section.contains("<box") {
        if let Some(cap) = regex::Regex::new(r#"<box\s+size="([^"]+)""#).unwrap().captures(&geometry_section) {
            println!("  Found box with size: {}", &cap[1]);
            return UrdfGeometry::Box { size: parse_xyz(&cap[1]) };
        }
    } else if geometry_section.contains("<sphere") {
        if let Some(cap) = regex::Regex::new(r#"<sphere\s+radius="([^"]+)""#).unwrap().captures(&geometry_section) {
            if let Ok(radius) = cap[1].parse() {
                println!("  Found sphere with radius: {}", radius);
                return UrdfGeometry::Sphere { radius };
            }
        }
    } else if geometry_section.contains("<cylinder") {
        if let Some(cap) = regex::Regex::new(r#"<cylinder\s+radius="([^"]+)"\s+length="([^"]+)""#).unwrap().captures(&geometry_section) {
            if let (Ok(radius), Ok(length)) = (cap[1].parse(), cap[2].parse()) {
                println!("  Found cylinder with radius: {}, length: {}", radius, length);
                return UrdfGeometry::Cylinder { radius, length };
            }
        }
    } else if geometry_section.contains("<mesh") {
        // Try both regular and self-closing mesh tags
        if let Some(cap) = regex::Regex::new(r#"<mesh\s+filename="([^"]+)""#).unwrap().captures(&geometry_section) {
            println!("  Found mesh with filename: {}", &cap[1]);
            return UrdfGeometry::Mesh { filename: cap[1].to_string() };
        }
        if let Some(cap) = regex::Regex::new(r#"<mesh\s+filename="([^"]+)"\s*/>"#).unwrap().captures(&geometry_section) {
            println!("  Found self-closing mesh with filename: {}", &cap[1]);
            return UrdfGeometry::Mesh { filename: cap[1].to_string() };
        }
    }
    
    println!("  No recognized geometry found, returning Unknown");
    UrdfGeometry::Unknown
}

fn parse_xyz(s: &str) -> [f32; 3] {
    let mut out = [0.0; 3];
    for (i, v) in s.split_whitespace().enumerate().take(3) {
        out[i] = v.parse().unwrap_or(0.0);
    }
    out
}

/// Creates appropriate fallback geometry based on link name
fn create_fallback_geometry(link_name: &str) -> UrdfGeometry {
    let link_lower = link_name.to_lowercase();
    
    if link_lower.contains("wheel") {
        // Wheels should be cylinders
        UrdfGeometry::Cylinder { radius: 0.05, length: 0.02 }
    } else if link_lower.contains("base") {
        // Base link should be a larger box
        UrdfGeometry::Box { size: [0.3, 0.15, 0.2] }
    } else if link_lower.contains("shoulder") || link_lower.contains("leg") {
        // Shoulder and leg links should be medium boxes
        UrdfGeometry::Box { size: [0.1, 0.1, 0.15] }
    } else if link_lower.contains("cover") {
        // Cover links should be thin boxes
        UrdfGeometry::Box { size: [0.25, 0.15, 0.02] }
    } else {
        // Default fallback
        UrdfGeometry::Box { size: [0.1, 0.1, 0.1] }
    }
}

/// Spawns a complete Bevy scene from a parsed URDF robot.
/// Each link is represented with appropriate geometry; joints create parent-child relationships.
pub fn spawn_urdf_scene(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    urdf: &UrdfRobot,
) {
    use std::collections::HashMap;
    
    // Build parent->children map
    let mut children_map: HashMap<String, Vec<&UrdfJoint>> = HashMap::new();
    let mut joint_map: HashMap<String, &UrdfJoint> = HashMap::new();
    
    for joint in &urdf.joints {
        children_map.entry(joint.parent.clone()).or_default().push(joint);
        joint_map.insert(joint.child.clone(), joint);
    }
    
    // Find root links (not a child in any joint)
    let all_children: std::collections::HashSet<&String> = urdf.joints.iter().map(|j| &j.child).collect();
    let root_links: Vec<&String> = urdf.links.iter().filter(|l| !all_children.contains(l)).collect();
    
    println!("Found {} root links: {:?}", root_links.len(), root_links);
    
    // Recursively spawn links starting from root links
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
            None, // No parent entity for root links
        );
    }
    
    // Print joint connections for debugging
    for joint in &urdf.joints {
        println!("Joint '{}' (type: {}) connects parent '{}' to child '{}' at origin {:?}", 
                joint.name, joint.joint_type, joint.parent, joint.child, joint.origin);
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
    parent_entity: Option<Entity>,
) {
    println!("Spawning link: {}", link_name);
    
    // Find the first visual for this link, or create fallback geometry
    let visual = urdf.visuals.iter().find(|v| v.link_name == link_name);
    let geometry = match visual {
        Some(v) => match &v.geometry {
            UrdfGeometry::Mesh { filename } => {
                println!("  Found mesh: {}", filename);
                // For now, create fallback geometry based on link name
                create_fallback_geometry(link_name)
            }
            geo => {
                println!("  Found geometry: {:?}", geo);
                geo.clone()
            }
        }
        None => {
            println!("  No visual found, creating fallback geometry");
            create_fallback_geometry(link_name)
        }
    };
    
    let (mesh_handle, material_handle) = match &geometry {
        UrdfGeometry::Box { size } => {
            let mesh = meshes.add(Mesh::from(Cuboid::new(size[0], size[1], size[2])));
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(size[0] / 2.0, size[1] / 2.0, size[2] / 2.0),
                ..Default::default()
            });
            (mesh, mat)
        }
        UrdfGeometry::Sphere { radius } => {
            let mesh = meshes.add(Mesh::from(Sphere { radius: *radius, ..Default::default() }));
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.5, 1.0),
                ..Default::default()
            });
            (mesh, mat)
        }
        UrdfGeometry::Cylinder { radius, length } => {
            let mesh = meshes.add(Mesh::from(Cylinder { radius: *radius, half_height: *length / 2.0, ..Default::default() }));
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.5, 0.5),
                ..Default::default()
            });
            (mesh, mat)
        }
        UrdfGeometry::Mesh { filename } => {
            // Try to load the STL file
            println!("Attempting to load STL file: {}", filename);
            
            // Construct the path - assuming STL files are in assets/robots/urdf/
            let base_path = PathBuf::from("assets/robots/urdf");
            let stl_path = base_path.join(filename);
            
            match stl_loader::load_stl_file(&stl_path) {
                Ok(mesh_data) => {
                    println!("Successfully loaded STL: {}", filename);
                    let mesh = meshes.add(mesh_data);
                    
                    // Color based on link name
                    let link_lower = link_name.to_lowercase();
                    let color = if link_lower.contains("wheel") {
                        Color::srgb(0.2, 0.2, 0.2) // Dark gray for wheels
                    } else if link_lower.contains("base") {
                        Color::srgb(0.7, 0.7, 0.7) // Light gray for base
                    } else if link_lower.contains("cover") {
                        Color::srgb(0.8, 0.85, 0.9) // Light blue-gray for covers
                    } else if link_lower.contains("shoulder") || link_lower.contains("leg") {
                        Color::srgb(0.82, 0.82, 1.0) // Light purple-blue for limbs
                    } else {
                        Color::hsl((link_name.len() as f32 * 30.0) % 360.0, 0.7, 0.5) // Colorful for other parts
                    };
                    
                    let mat = materials.add(StandardMaterial {
                        base_color: color,
                        ..Default::default()
                    });
                    (mesh, mat)
                }
                Err(e) => {
                    println!("Failed to load STL file '{}': {}", filename, e);
                    println!("Falling back to colored box");
                    
                    // Fallback to colored box
                    let link_lower = link_name.to_lowercase();
                    let color = if link_lower.contains("wheel") {
                        Color::srgb(0.2, 0.2, 0.2) // Dark gray for wheels
                    } else if link_lower.contains("base") {
                        Color::srgb(0.7, 0.7, 0.7) // Light gray for base
                    } else {
                        Color::hsl((link_name.len() as f32 * 30.0) % 360.0, 0.7, 0.5) // Colorful for other parts
                    };
                    let mesh = meshes.add(Mesh::from(Cuboid::new(0.1, 0.1, 0.1)));
                    let mat = materials.add(StandardMaterial {
                        base_color: color,
                        ..Default::default()
                    });
                    (mesh, mat)
                }
            }
        }
        UrdfGeometry::Unknown => {
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
        UrdfGeometry::Mesh { filename } => {
            // Try to load the STL for collision
            let base_path = PathBuf::from("assets/robots/urdf");
            let stl_path = base_path.join(filename);
            
            match stl_loader::load_stl_file(&stl_path) {
                Ok(mesh_data) => {
                    println!("Creating trimesh collider from STL: {}", filename);
                    // Extract vertices and indices from the mesh
                    if let Some(vertex_attr) = mesh_data.attribute(Mesh::ATTRIBUTE_POSITION) {
                        match vertex_attr {
                            bevy::render::mesh::VertexAttributeValues::Float32x3(positions) => {
                                let vertices: Vec<Vec3> = positions.iter()
                                    .map(|p| Vec3::new(p[0], p[1], p[2]))
                                    .collect();
                                
                                if let Some(indices) = mesh_data.indices() {
                                    match indices {
                                        bevy::render::mesh::Indices::U32(idx) => {
                                            let indices: Vec<[u32; 3]> = idx.chunks_exact(3)
                                                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                                                .collect();
                                            
                                            Collider::trimesh(vertices, indices)
                                        }
                                        bevy::render::mesh::Indices::U16(idx) => {
                                            let indices: Vec<[u32; 3]> = idx.chunks_exact(3)
                                                .map(|chunk| [chunk[0] as u32, chunk[1] as u32, chunk[2] as u32])
                                                .collect();
                                            
                                            Collider::trimesh(vertices, indices)
                                        }
                                    }
                                } else {
                                    println!("No indices found in STL mesh, using default collider");
                                    Collider::cuboid(0.1, 0.1, 0.1)
                                }
                            }
                            _ => {
                                println!("Unexpected vertex format, using default collider");
                                Collider::cuboid(0.1, 0.1, 0.1)
                            }
                        }
                    } else {
                        println!("No vertices found in STL mesh, using default collider");
                        Collider::cuboid(0.1, 0.1, 0.1)
                    }
                }
                Err(e) => {
                    println!("Failed to load collision STL '{}': {}, using default collider", filename, e);
                    Collider::cuboid(0.1, 0.1, 0.1)
                }
            }
        }
        _ => Collider::cuboid(0.1, 0.1, 0.1),
    });
    
    // Create the entity
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
    
    // Set parent-child relationship if this link has a parent
    if let Some(parent_entity) = parent_entity {
        commands.entity(parent_entity).add_child(entity);
    }
    
    // Recurse for children
    if let Some(joints) = children_map.get(link_name) {
        for joint in joints {
            let joint_transform = joint_to_transform(&joint.origin);
            let child_transform = parent_transform.mul_transform(joint_transform);
            
            spawn_link_recursive(
                commands,
                meshes,
                materials,
                urdf,
                &joint.child,
                children_map,
                joint_map,
                child_transform,
                Some(entity), // Pass this entity as parent
            );
        }
    }
}

fn joint_to_transform(origin: &UrdfOrigin) -> Transform {
    use bevy::math::{Quat, Vec3};
    let translation = Vec3::from(origin.xyz);
    let (r, p, y) = (origin.rpy[0], origin.rpy[1], origin.rpy[2]);
    let rotation = Quat::from_euler(EulerRot::XYZ, r, p, y);
    Transform::from_translation(translation).with_rotation(rotation)
} 