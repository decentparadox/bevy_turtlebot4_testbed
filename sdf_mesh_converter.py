#!/usr/bin/env python3
"""
SDF Mesh Converter - Converts SDF-referenced meshes to GLTF/GLB format for Bevy

This script:
1. Parses SDF files to find mesh references
2. Converts COLLADA (.dae), OBJ, STL files to GLTF/GLB
3. Updates SDF references to point to converted files
4. Preserves directory structure for organized assets

Dependencies:
- trimesh: pip install trimesh[easy]
- pygltflib: pip install pygltflib
- lxml: pip install lxml

Usage:
    python sdf_mesh_converter.py <sdf_file> [--output-dir <dir>] [--format glb|gltf]
"""

import argparse
import os
import sys
import shutil
import re
from pathlib import Path
from typing import List, Dict, Tuple, Optional
import xml.etree.ElementTree as ET

try:
    import trimesh
    import pygltflib
    from lxml import etree
except ImportError as e:
    print(f"Missing dependency: {e}")
    print("Install with: pip install trimesh[easy] pygltflib lxml")
    sys.exit(1)


class SdfMeshConverter:
    def __init__(self, output_dir: str = "assets/converted", format: str = "glb"):
        self.output_dir = Path(output_dir)
        self.format = format.lower()
        self.converted_meshes: Dict[str, str] = {}
        self.sdf_dir = None
        
        # Ensure output directory exists
        self.output_dir.mkdir(parents=True, exist_ok=True)
        
        # Supported input formats
        self.supported_formats = {'.dae', '.obj', '.stl', '.ply', '.off'}
        
    def parse_sdf_meshes(self, sdf_file: str) -> List[Dict]:
        """Parse SDF file and extract all mesh references"""
        meshes = []
        
        try:
            # Parse with lxml for better namespace handling
            parser = etree.XMLParser(strip_cdata=False)
            tree = etree.parse(sdf_file, parser)
            root = tree.getroot()
            
            # Handle namespaces
            namespaces = {'sdf': 'http://sdformat.org/schemas/sdf'} if root.nsmap else {}
            
            # Find all mesh elements
            mesh_elements = root.xpath('.//mesh', namespaces=namespaces) if namespaces else root.xpath('.//mesh')
            
            for mesh_elem in mesh_elements:
                uri_elem = mesh_elem.find('uri')
                scale_elem = mesh_elem.find('scale')
                
                if uri_elem is not None:
                    uri = uri_elem.text.strip()
                    scale = [1.0, 1.0, 1.0]
                    
                    if scale_elem is not None:
                        scale_text = scale_elem.text.strip()
                        try:
                            scale = [float(x) for x in scale_text.split()]
                            if len(scale) == 1:
                                scale = [scale[0], scale[0], scale[0]]
                            elif len(scale) != 3:
                                scale = [1.0, 1.0, 1.0]
                        except:
                            scale = [1.0, 1.0, 1.0]
                    
                    meshes.append({
                        'uri': uri,
                        'scale': scale,
                        'element': mesh_elem
                    })
                    
        except Exception as e:
            print(f"Error parsing SDF file {sdf_file}: {e}")
            return []
            
        return meshes
        
    def resolve_mesh_path(self, uri: str, sdf_file: str) -> Optional[str]:
        """Resolve mesh URI to actual file path"""
        # Handle different URI schemes
        if uri.startswith('file://'):
            mesh_path = uri[7:]  # Remove 'file://' prefix
        elif uri.startswith('model://'):
            # ROS/Gazebo model:// scheme
            # This typically maps to ~/.gazebo/models/ or similar
            model_name = uri.split('/')[2] if len(uri.split('/')) > 2 else ''
            relative_path = '/'.join(uri.split('/')[3:]) if len(uri.split('/')) > 3 else ''
            
            # Try common Gazebo model locations
            gazebo_dirs = [
                os.path.expanduser('~/.gazebo/models'),
                '/usr/share/gazebo/models',
                '/opt/ros/*/share/gazebo/models'
            ]
            
            for gazebo_dir in gazebo_dirs:
                candidate = os.path.join(gazebo_dir, model_name, relative_path)
                if os.path.exists(candidate):
                    mesh_path = candidate
                    break
            else:
                # If not found in standard locations, try relative to SDF
                mesh_path = os.path.join(os.path.dirname(sdf_file), relative_path)
        elif uri.startswith('package://'):
            # ROS package:// scheme
            parts = uri.split('/')
            if len(parts) >= 3:
                package_name = parts[2]
                relative_path = '/'.join(parts[3:])
                # This would require ROS package resolution - simplified for now
                mesh_path = os.path.join(os.path.dirname(sdf_file), relative_path)
            else:
                mesh_path = uri
        else:
            # Relative or absolute path
            if os.path.isabs(uri):
                mesh_path = uri
            else:
                mesh_path = os.path.join(os.path.dirname(sdf_file), uri)
        
        # Normalize path
        mesh_path = os.path.normpath(mesh_path)
        
        # Check if file exists
        if os.path.exists(mesh_path):
            return mesh_path
        else:
            print(f"Warning: Mesh file not found: {mesh_path}")
            return None
            
    def convert_mesh(self, input_path: str, scale: List[float] = [1.0, 1.0, 1.0]) -> Optional[str]:
        """Convert a mesh file to GLTF/GLB format"""
        input_path = Path(input_path)
        
        if input_path.suffix.lower() not in self.supported_formats:
            print(f"Unsupported format: {input_path.suffix}")
            return None
            
        # Generate output filename
        output_name = input_path.stem + f".{self.format}"
        output_path = self.output_dir / output_name
        
        # Check if already converted
        if str(input_path) in self.converted_meshes:
            return self.converted_meshes[str(input_path)]
            
        try:
            print(f"Converting {input_path} -> {output_path}")
            
            # Load mesh with trimesh
            if input_path.suffix.lower() == '.dae':
                # Special handling for COLLADA files
                mesh = trimesh.load(str(input_path))
                
                # Handle mesh collections (multi-object COLLADA files)
                if isinstance(mesh, trimesh.Scene):
                    # Combine all geometries into a single mesh
                    combined = trimesh.util.concatenate([
                        geom for geom in mesh.geometry.values() 
                        if hasattr(geom, 'vertices')
                    ])
                    mesh = combined
                    
            else:
                mesh = trimesh.load(str(input_path))
            
            # Apply scaling if needed
            if scale != [1.0, 1.0, 1.0]:
                mesh.apply_scale(scale)
            
            # Export to GLTF/GLB
            if self.format == 'glb':
                mesh.export(str(output_path))
            else:  # gltf
                mesh.export(str(output_path))
            
            # Store conversion mapping
            self.converted_meshes[str(input_path)] = str(output_path)
            
            print(f"Successfully converted: {output_path}")
            return str(output_path)
            
        except Exception as e:
            print(f"Error converting {input_path}: {e}")
            return None
    
    def update_sdf_references(self, sdf_file: str, meshes: List[Dict], output_sdf: str = None):
        """Update SDF file with new mesh references"""
        if output_sdf is None:
            output_sdf = sdf_file.replace('.sdf', '_converted.sdf')
        
        try:
            # Read original SDF content
            with open(sdf_file, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Update mesh URIs
            for mesh_info in meshes:
                original_uri = mesh_info['uri']
                mesh_path = self.resolve_mesh_path(original_uri, sdf_file)
                
                if mesh_path and mesh_path in self.converted_meshes:
                    new_uri = self.converted_meshes[mesh_path]
                    # Convert to relative path from SDF location
                    sdf_dir = os.path.dirname(output_sdf)
                    try:
                        new_uri = os.path.relpath(new_uri, sdf_dir)
                        # Use forward slashes for consistency
                        new_uri = new_uri.replace('\\', '/')
                    except:
                        pass  # Keep absolute path if relative conversion fails
                    
                    # Replace URI in content
                    content = content.replace(f'<uri>{original_uri}</uri>', f'<uri>{new_uri}</uri>')
                    print(f"Updated URI: {original_uri} -> {new_uri}")
            
            # Write updated SDF
            with open(output_sdf, 'w', encoding='utf-8') as f:
                f.write(content)
            
            print(f"Updated SDF saved as: {output_sdf}")
            
        except Exception as e:
            print(f"Error updating SDF file: {e}")
    
    def convert_sdf_meshes(self, sdf_file: str) -> bool:
        """Main function to convert all meshes in an SDF file"""
        print(f"Processing SDF file: {sdf_file}")
        
        # Parse SDF to find meshes
        meshes = self.parse_sdf_meshes(sdf_file)
        print(f"Found {len(meshes)} mesh references")
        
        if not meshes:
            print("No meshes found in SDF file")
            return True
        
        # Convert each mesh
        conversion_count = 0
        for mesh_info in meshes:
            uri = mesh_info['uri']
            scale = mesh_info['scale']
            
            mesh_path = self.resolve_mesh_path(uri, sdf_file)
            if mesh_path:
                converted_path = self.convert_mesh(mesh_path, scale)
                if converted_path:
                    conversion_count += 1
        
        print(f"Successfully converted {conversion_count}/{len(meshes)} meshes")
        
        # Update SDF file with new references
        if conversion_count > 0:
            self.update_sdf_references(sdf_file, meshes)
        
        return conversion_count > 0


def main():
    parser = argparse.ArgumentParser(description='Convert SDF-referenced meshes to GLTF/GLB')
    parser.add_argument('sdf_file', help='Input SDF file')
    parser.add_argument('--output-dir', '-o', default='assets/converted', 
                       help='Output directory for converted meshes')
    parser.add_argument('--format', '-f', choices=['glb', 'gltf'], default='glb',
                       help='Output format (default: glb)')
    parser.add_argument('--recursive', '-r', action='store_true',
                       help='Process all SDF files in directory recursively')
    
    args = parser.parse_args()
    
    converter = SdfMeshConverter(args.output_dir, args.format)
    
    if args.recursive and os.path.isdir(args.sdf_file):
        # Process all SDF files in directory
        sdf_files = []
        for root, dirs, files in os.walk(args.sdf_file):
            for file in files:
                if file.endswith('.sdf'):
                    sdf_files.append(os.path.join(root, file))
        
        print(f"Found {len(sdf_files)} SDF files to process")
        
        success_count = 0
        for sdf_file in sdf_files:
            if converter.convert_sdf_meshes(sdf_file):
                success_count += 1
        
        print(f"Successfully processed {success_count}/{len(sdf_files)} SDF files")
        
    else:
        # Process single SDF file
        if not os.path.exists(args.sdf_file):
            print(f"Error: SDF file not found: {args.sdf_file}")
            sys.exit(1)
        
        success = converter.convert_sdf_meshes(args.sdf_file)
        
        if success:
            print("Conversion completed successfully!")
        else:
            print("Conversion failed or no meshes were converted")
            sys.exit(1)


if __name__ == "__main__":
    main()
