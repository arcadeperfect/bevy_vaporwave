use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{Indices, MeshVertexAttribute, MeshVertexBufferLayoutRef, PrimitiveTopology, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError, VertexFormat,
        },
    }, utils::HashSet,
};

use std::hash::{Hash, Hasher};

const ATTRIBUTE_CUSTOM: MeshVertexAttribute =
    MeshVertexAttribute::new("Custom", 2137464976, VertexFormat::Float32);

// const LINE_SHADER_PATH: &str = "shaders/line.wgsl";
// const SOLID_SHADER_PATH: &str = "shaders/line.wgsl";

#[derive(Asset, TypePath, Default, AsBindGroup, Debug, Clone)]
pub struct LineMaterial {}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/line.wgsl".into()
    }

    // fn specialize(
    //     _pipeline: &MaterialPipeline<Self>,
    //     descriptor: &mut RenderPipelineDescriptor,
    //     _layout: &MeshVertexBufferLayoutRef,
    //     _key: MaterialPipelineKey<Self>,
    // ) -> Result<(), SpecializedMeshPipelineError> {
    //     // I don't think we need this because it infers it from the primitive topology type
    //     // descriptor.primitive.polygon_mode = PolygonMode::Line;
    //     Ok(())
    // }
    
}


/// A list of lines with a start and end position
#[derive(Clone, Default)]
pub struct IndexLineList {
    pub lines: Vec<(IndexVert, IndexVert)>,
}

impl IndexLineList {
    pub fn print(&self) {
        for line in self.lines.iter() {
            println!("{} -> {}", line.0.index, line.1.index);
        }
    }
}

#[derive(Clone, Default)]
pub struct DataLineList {
    pub lines: Vec<(DataVert, DataVert)>,
}

#[derive(Debug, Clone, Default)]
pub struct IndexVert {
    pub index: u32,
}

#[derive(Debug, Clone, Default)]
pub struct DataVert {
    pub position: [f32; 3],
    pub color: Option<[f32; 4]>,
    pub normal: Option<[f32; 3]>,
    pub joint_indices: Option<[u16; 4]>,
    pub joint_weights: Option<[f32; 4]>,
}




pub fn generate_edge_line_list_data(mesh: &Mesh) -> DataLineList {

    let mut line_list = DataLineList::default();
    let mut edge_set = HashSet::new();
    println!("a");

    if let Some(VertexAttributeValues::Float32x3(_)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        println!("position data present");
    }

    if let Some(VertexAttributeValues::Float32x4(_)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
        println!("color data present");
    }

    if let Some(VertexAttributeValues::Float32x3(_)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        println!("normal data present");
    }

    if let Some(VertexAttributeValues::Uint16x4(_)) = mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX) {
        println!("joint index data present");
    }
    
    if let Some(VertexAttributeValues::Float32x4(_)) = mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT) {
        println!("joint weight data present");
    }

    if let Some(indices) = mesh.indices() {
        println!("indices present");
    }


    if let (
        Some(VertexAttributeValues::Float32x3(positions)),
        Some(VertexAttributeValues::Float32x4(colors)),
        Some(VertexAttributeValues::Float32x3(normal)),
        Some(VertexAttributeValues::Uint16x4(joint_indices)),
        Some(VertexAttributeValues::Float32x4(joint_weights)),
        Some(indices),
    ) = (
        mesh.attribute(Mesh::ATTRIBUTE_POSITION),
        mesh.attribute(Mesh::ATTRIBUTE_COLOR),
        mesh.attribute(Mesh::ATTRIBUTE_NORMAL),
        mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX),
        mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT),
        mesh.indices(),
    ) {

        println!("b");

        let custom = mesh.attribute(ATTRIBUTE_CUSTOM).and_then(|attr| {
            if let VertexAttributeValues::Float32(values) = attr {
                Some(values)
            } else {
                None
            }
        });

        let mut process_triangle = |a: u32, b: u32, c: u32| {
            let mut add_edge = |v1: u32, v2: u32| {
                let should_connect = match &custom {
                    Some(custom_values) => {
                        let v1_value = custom_values[v1 as usize];
                        let v2_value = custom_values[v2 as usize];
                        v1_value == v2_value && v1_value != 0.0
                    }
                    None => true, // If no custom attribute, always connect
                };

                if should_connect {
                    let edge = if v1 < v2 { (v1, v2) } else { (v2, v1) };
                    if edge_set.insert(edge) {
                        let start = DataVert {
                            position: positions[v1 as usize],
                            color: Some(colors[v1 as usize]),
                            
                            normal: Some(normal[v1 as usize]),
                            joint_indices: Some(joint_indices[v1 as usize]),
                            joint_weights: Some(joint_weights[v1 as usize]),
                        };
                        let end = DataVert {
                            position: positions[v2 as usize],
                            color: Some(colors[v2 as usize]),
                            normal: Some(normal[v2 as usize]),
                            joint_indices: Some(joint_indices[v2 as usize]),
                            joint_weights: Some(joint_weights[v2 as usize]),
                        };
                        line_list.lines.push((start, end));
                    }
                }
            };

            add_edge(a, b);
            add_edge(b, c);
            add_edge(c, a);
        };
        match indices {
            Indices::U16(idx) => {
                for triangle in idx.chunks(3) {
                    process_triangle(triangle[0] as u32, triangle[1] as u32, triangle[2] as u32);
                }
            }
            Indices::U32(idx) => {
                for triangle in idx.chunks(3) {
                    process_triangle(triangle[0], triangle[1], triangle[2]);
                }
            }
        }
    }

    line_list
}




pub fn generate_edge_line_list_indices(mesh: &Mesh) -> IndexLineList {
    let mut line_list = IndexLineList::default();
    let mut edge_set = HashSet::new();
    println!("a");

    if let (
        // Some(VertexAttributeValues::Float32x3(positions)),
        Some(indices),
    ) = (
        // mesh.attribute(Mesh::ATTRIBUTE_POSITION),
        mesh.indices(),
    ) {
        let mut process_triangle_index = |a: u32, b: u32, c: u32| {
            let mut add_edge = |v1: u32, v2: u32| {
                if true {
                    
                    let edge = if v1 < v2 { (v1, v2) } else { (v2, v1) };
                    if edge_set.insert(edge) {
                        let start = IndexVert {
                            index: v1
                        };
                        let end = IndexVert {
                            index: v2
                        };
                        line_list.lines.push((start, end));
                    }
                }
            };

            add_edge(a, b);
            add_edge(b, c);
            add_edge(c, a);
        };

        match indices {
            Indices::U16(idx) => {
                for triangle in idx.chunks(3) {
                    process_triangle_index(triangle[0] as u32, triangle[1] as u32, triangle[2] as u32);
                }
            }
            Indices::U32(idx) => {
                for triangle in idx.chunks(3) {
                    process_triangle_index(triangle[0], triangle[1], triangle[2]);
                }
            }
        }
    }

    println!("Generated {} lines", line_list.lines.len());

    line_list
}




#[derive(PartialEq, Eq, Hash)]
pub struct IntEdge {
    pub p1: [i32; 3],
    pub p2: [i32; 3],
}

fn f32_to_i32(f: f32) -> i32 {
    // You can choose the scaling factor depending on the precision you need
    (f * 100000.0) as i32
}

impl IntEdge{
   pub fn new_from_floats(p1: [f32; 3], p2: [f32; 3]) -> Self {
        IntEdge {
            p1: [f32_to_i32(p1[0]), f32_to_i32(p1[1]), f32_to_i32(p1[2])],
            p2: [f32_to_i32(p2[0]), f32_to_i32(p2[1]), f32_to_i32(p2[2])],
        }
    }
}

