// force build again again again

use bevy::gltf::{GltfExtras, GltfSceneExtras};
use bevy::prelude::Color;
use bevy::{
    animation::animate_targets,
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    gltf::GltfPlugin,
    prelude::*,
    render::{
        mesh::{skinning::SkinnedMesh, MeshVertexAttribute},
        render_resource::VertexFormat,
    },
    scene::SceneInstanceReady,
};

use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use fill_material::FillMaterial;
use line_material::LineMaterial;
use mesh_ops::{
    get_smoothed_normals, line_list_to_mesh, MeshToLineList,
};
use outline_material::OutlineMaterial;
use parse_extras::JsonLineList;
use serde_json::Value;
use std::time::Duration;

mod camera_plugin;
mod fill_material;
mod line_material;
mod mesh_ops;
mod outline_material;
mod parse_extras;

const ASTRO_PATH: &str = "gltf/astro.gltf";
const TORUS_PATH: &str = "gltf/torus.gltf";
const COUPE_PATH: &str = "gltf/coupe.gltf";
const SPHERE_PATH: &str = "gltf/sphere.gltf";

#[derive(Resource, PartialEq)]
enum VisibleModel {
    Astro,
    Coupe,
    Torus,
    Sphere,
}

#[derive(Component)]
struct FillTag;
#[derive(Component)]
struct WireframeTag;
#[derive(Component)]
struct OutlineTag;
#[derive(Component)]
struct AstroSceneTag;

#[derive(Component)]
struct CoupeSceneTag;
#[derive(Component)]
struct SphereSceneTag;
#[derive(Component)]
struct TorusSceneTag;
#[derive(Resource)]
struct Animations {
    astro_animations: Vec<AnimationNodeIndex>,
    astro_graph: Handle<AnimationGraph>,
    coupe_animations: Vec<AnimationNodeIndex>,
    coupe_graph: Handle<AnimationGraph>,
}

#[derive(Resource)]
struct ShaderSettings {
    outline_width: f32,
    wireframe_displacement: f32,
    fill_displacement: f32,
    fill_shininess: f32,
    fill_specular_strength: f32,
    brightness: f32,
    vertex_color_mode: i32,
    color: Color,
    show_wireframe: bool,
    show_outline: bool,
    show_fill: bool,
}

impl Default for ShaderSettings {
    fn default() -> Self {
        Self {
            outline_width: 0.1,
            wireframe_displacement: 0.0,
            fill_displacement: 0.0,
            fill_shininess: 250.0,
            fill_specular_strength: 0.1,
            brightness: 15.0,
            vertex_color_mode: 1,
            color: Color::WHITE,
            show_wireframe: true,
            show_outline: true,
            show_fill: true,
        }
    }
}


const ATTRIBUTE_VERT_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("VERT_INDEX", 1237464976, VertexFormat::Float32);

const ATTRIBUTE_SMOOTHED_NORMAL: MeshVertexAttribute =
    MeshVertexAttribute::new("SmoothNormal", 723495149, VertexFormat::Float32x3);


fn main() {
    App::new()
        .insert_resource(VisibleModel::Astro)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(ShaderSettings::default())
        .add_plugins(
            DefaultPlugins.set(
                GltfPlugin::default()
                    .add_custom_vertex_attribute("VERT_INDEX", ATTRIBUTE_VERT_INDEX)
                    .add_custom_vertex_attribute("SMOOTH_NORMAL", ATTRIBUTE_SMOOTHED_NORMAL), // .add_custom_vertex_attribute("ALT_COLOR", ATTRIBUTE_ALT_COLOR),
            ),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(MaterialPlugin::<FillMaterial>::default())
        .add_plugins(MaterialPlugin::<OutlineMaterial>::default())
        .add_plugins(MaterialPlugin::<LineMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, spawn)
        .add_systems(Update, play_animation_once_loaded.before(animate_targets))
        .add_systems(Update, post_process)
        .add_systems(Update, ui_system) // Add this line
        .add_systems(Update, update_scene_visibility)
        // .add_systems(Update, handle_color_switching)
        .run();
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut graph_assets: ResMut<Assets<AnimationGraph>>,
) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            ..default()
        },
        PanOrbitCamera::default(),
        BloomSettings::NATURAL,
    ));

    // Build the animation graph for the astronaut
    let mut astro_graph = AnimationGraph::new();
    let astro_animations = astro_graph
        .add_clips(
            [GltfAssetLabel::Animation(0).from_asset(ASTRO_PATH)]
                .into_iter()
                .map(|path| assets.load(path)),
            1.0,
            astro_graph.root,
        )
        .collect();

    // Insert a resource with the current scene information
    let astro_graph_handle = graph_assets.add(astro_graph);

    // Build the animation graph for the coupe
    let mut coupe_path = AnimationGraph::new();
    let coupe_animations = coupe_path
        .add_clips(
            [GltfAssetLabel::Animation(0).from_asset(COUPE_PATH)]
                .into_iter()
                .map(|path| assets.load(path)),
            1.0,
            coupe_path.root,
        )
        .collect();

    // Insert a resource with the current scene information
    let coupe_graph_handle = graph_assets.add(coupe_path);

    commands.insert_resource(Animations {
        astro_animations,
        astro_graph: astro_graph_handle,
        coupe_animations,
        coupe_graph: coupe_graph_handle,
    });
}

fn spawn(mut commands: Commands, assets: Res<AssetServer>) {
    let coupe = commands
        .spawn((
            SceneBundle {
                scene: assets.load(GltfAssetLabel::Scene(0).from_asset(COUPE_PATH)),
                transform: Transform::from_xyz(0.0, 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_y(0.0))
                    .with_scale(Vec3::splat(1.0)),
                ..default()
            },
            // WireframeSettings::default(),
            CoupeSceneTag,
        ))
        .id();

    let astro = commands
        .spawn((
            SceneBundle {
                scene: assets.load(GltfAssetLabel::Scene(0).from_asset(ASTRO_PATH)),
                transform: Transform::from_xyz(0.0, -1.2, 0.0)
                    .with_rotation(Quat::from_rotation_y(0.0))
                    .with_scale(Vec3::splat(1.)),
                ..default()
            },
            // WireframeSettings::default(),
            AstroSceneTag,
        ))
        .id();

    let torus = commands
        .spawn((
            SceneBundle {
                scene: assets.load(GltfAssetLabel::Scene(0).from_asset(TORUS_PATH)),
                transform: Transform::from_xyz(0.0, 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_y(0.0))
                    .with_scale(Vec3::splat(1.)),
                ..default()
            },
            // WireframeSettings::default(),
            TorusSceneTag,
        ))
        .id();

    let sphere = commands
        .spawn((
            SceneBundle {
                scene: assets.load(GltfAssetLabel::Scene(0).from_asset(SPHERE_PATH)),
                transform: Transform::from_xyz(0.0, 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_y(0.0))
                    .with_scale(Vec3::splat(1.)),
                ..default()
            },
            // WireframeSettings::default(),
            SphereSceneTag,
        ))
        .id();
}



fn post_process(
    mut commands: Commands,
    mut events: EventReader<SceneInstanceReady>,
    extras: Query<(&Parent, &GltfExtras)>,
    scene_extras: Query<&GltfSceneExtras>,
    mesh: Query<(&Handle<Mesh>, &Parent)>,
    children: Query<&Children>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
    mut fill_materials: ResMut<Assets<FillMaterial>>,
    mut outline_materials: ResMut<Assets<OutlineMaterial>>, // Add FillMaterial resource
    mut mesh_assets: ResMut<Assets<Mesh>>,
    shader_settings: Res<ShaderSettings>,
    // mut wf: Query<&mut WireframeSettings>,
    skinned_meshes: Query<&SkinnedMesh>,

) {

    for event in events.read() {
        
        // Out of laziness I iterate through the whole scene until I find the scene level extra which contains a json dictionary
        // that encodes the line lists generated in blender, with the index of the mesh as the key
        // there will only be one of these, so once it finds it, it parses it and breaks the loop
        // TODO: better way to locate the scene level extra


        // Iterate through each mesh and apply the wireframe post processing
        // If scene extras were found, it will generate the line lists according to the json dictionary
        // Otherwise it will generate a line list for every edge of the mesh

        for this_entity in children.iter_descendants(event.parent) {
            if let Ok((mesh_handle, parent)) = mesh.get(this_entity) {
                if let Some(mesh) = mesh_assets.get_mut(mesh_handle) {
                    commands
                        .entity(this_entity)
                        .remove::<Handle<StandardMaterial>>();

                    let smoothed_normals: Vec<[f32; 3]> = get_smoothed_normals(mesh).unwrap();
                    // invert_normals(&mut smoothed_normals);
                    mesh.insert_attribute(ATTRIBUTE_SMOOTHED_NORMAL, smoothed_normals);
                    mesh.duplicate_vertices();
                    mesh.compute_flat_normals();

                    // Check for Vertex_Color attribute
                    if !mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_some() {
                        warn!("Vertex_Color attribute not found");
                        // If Vertex_Color is not present, add it with a constant color
                        let vertex_count = mesh.count_vertices();
                        let constant_color = [1.0, 0.0, 1.0, 0.0]; // White color, adjust as needed
                        let colors: Vec<[f32; 4]> = vec![constant_color; vertex_count];
                        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                    }


                    // FILL

                    let fill_material_handle = fill_materials.add(FillMaterial {
                        color: Vec4::new(1.0, 0.0, 0.0, 1.0),
                        displacement: 0.0,
                        shininess: 200.0,
                        specular_strength: 1.0,
                        vertex_color_mode: 1,
                        visibility: 1.0,
                    });

                    let skinned_mesh = skinned_meshes.get(this_entity).cloned(); // required for scenes with skinned mesh animations

                    commands.entity(this_entity).with_children(|parent| {
                        let mut child_entity = parent.spawn((
                            MaterialMeshBundle {
                                mesh: mesh_handle.clone(),
                                material: fill_material_handle,
                                visibility: Visibility::Inherited,
                                ..Default::default()
                            },
                            FillTag,
                        ));

                        // If the original entity had a SkinnedMesh component, add it to the new entity
                        if let Ok(skinned_mesh) = skinned_mesh {
                            child_entity.insert(skinned_mesh);
                        }
                    });
                    
                    
                    // OUTLINE

                    // Add OutlineMaterial component
                    let outline_material_handle = outline_materials.add(OutlineMaterial {
                        outline_width: shader_settings.outline_width,
                        ..default()
                    });


                    let skinned_mesh = skinned_meshes.get(this_entity).cloned(); // required for scenes with skinned mesh animations

                    commands.entity(this_entity).with_children(|parent| {
                        let mut child_entity = parent.spawn((
                            MaterialMeshBundle {
                                mesh: mesh_handle.clone(),
                                material: outline_material_handle,
                                visibility: Visibility::Inherited,
                                ..Default::default()
                            },
                            OutlineTag,
                        ));

                        // If the original entity had a SkinnedMesh component, add it to the new entity
                        if let Ok(skinned_mesh) = skinned_mesh {
                            child_entity.insert(skinned_mesh);
                        }
                    });

                    // WIRE FRAME

                    // LineList stores the data required to build a mesh of lines
                    // It can be derived from gltf extra data, or generated for every triangle in the absence

                    let parsed_line_list = if let Ok(mesh_extra) = extras.get(parent.get()) {
                        if let Ok(json_value) = serde_json::from_str::<Value>(&mesh_extra.1.value) {
                            if let Some(v) = json_value.get("gltf_primitive_extras") {
                                if let Ok(edge_data) =
                                    serde_json::from_str::<JsonLineList>(v.as_str().unwrap_or(""))
                                {
                                    Some(edge_data)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let line_list;

                    match parsed_line_list {
                        Some(p) => {
                            line_list = mesh.mesh_to_line_list_from_json(&p);
                        }
                        None => {
                            line_list = mesh.mesh_to_line_list();
                        }
                    }

                    let line_mesh = line_list_to_mesh(&line_list, &mesh);
                    let new_mesh_handle = mesh_assets.add(line_mesh);
                    let skinned_mesh = skinned_meshes.get(this_entity).cloned(); // required for scenes with skinned mesh animations

                    commands.entity(this_entity).with_children(|parent| {
                        let mut child_entity = parent.spawn((
                            MaterialMeshBundle {
                                mesh: new_mesh_handle,
                                material: line_materials.add(LineMaterial {
                                    displacement: 1.5,
                                    ..default()
                                }),
                                visibility: Visibility::Inherited,
                                ..Default::default()
                            },
                            WireframeTag,
                        ));

                        // If the original entity had a SkinnedMesh component, add it to the new entity
                        if let Ok(skinned_mesh) = skinned_mesh {
                            child_entity.insert(skinned_mesh);
                        }
                    });
                }
            }
        }
    }
}

// from the example
fn play_animation_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    parent_query: Query<&Parent>,
    astro_scenes: Query<Entity, With<AstroSceneTag>>,
    coupe_scenes: Query<Entity, With<CoupeSceneTag>>,
) {
    for (entity, mut player) in &mut players {


        // Start with the current entity, which has an AnimationPlayer
        let mut current_entity = entity;

        // Traverse up the hierarchy to find the top-level parent
        loop {
            // Attempt to find the Parent component of the current entity
            match parent_query.get(current_entity) {
                Ok(parent) => {
                    // If the current entity has a parent, update current_entity to the parent
                    current_entity = parent.get();
                }
                Err(_) => {
                    // If the current entity does not have a parent, break the loop
                    break;
                }
            }
        }

        if astro_scenes.get(current_entity).is_ok() {
            println!("astro scene");
            let mut transitions = AnimationTransitions::new();
            transitions
                .play(&mut player, animations.astro_animations[0], Duration::ZERO)
                .repeat();
            commands
                .entity(entity)
                .insert(animations.astro_graph.clone())
                .insert(transitions);
        } else if coupe_scenes.get(current_entity).is_ok() {
            println!("coupe scene");
            let mut transitions = AnimationTransitions::new();
            transitions
                .play(&mut player, animations.coupe_animations[0], Duration::ZERO)
                .repeat();
            commands
                .entity(entity)
                .insert(animations.coupe_graph.clone())
                .insert(transitions);
        }
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut shader_settings: ResMut<ShaderSettings>,
    mut outline_materials_assets: ResMut<Assets<OutlineMaterial>>,
    outline_materials: Query<&Handle<OutlineMaterial>>,
    mut line_materials_assets: ResMut<Assets<LineMaterial>>,
    line_materials: Query<&Handle<LineMaterial>>,
    mut fill_materials_assets: ResMut<Assets<FillMaterial>>,
    fill_materials: Query<&Handle<FillMaterial>>,
    mut visible_model: ResMut<VisibleModel>,
    mut visibility_set: ParamSet<(
        Query<&mut Visibility, With<FillTag>>,
        Query<&mut Visibility, With<OutlineTag>>,
        Query<&mut Visibility, With<WireframeTag>>,
    )>,
) {
    egui::Window::new("Shader Controls").show(contexts.ctx_mut(), |ui| {
        ui.add(
            egui::Slider::new(&mut shader_settings.outline_width, 0.0..=1.0).text("Outline Width"),
        );
        ui.add(
            egui::Slider::new(&mut shader_settings.wireframe_displacement, 0.0..=5.0)
                .text("Wireframe Displacement"),
        );
        ui.add(
            egui::Slider::new(&mut shader_settings.fill_displacement, 0.0..=1.0)
                .text("Fill Displacement"),
        );
        ui.add(
            egui::Slider::new(&mut shader_settings.fill_shininess, 1.0..=256.0).text("Shininess"),
        );
        ui.add(
            egui::Slider::new(&mut shader_settings.fill_specular_strength, 0.0..=1.0)
                .text("Specular Strength"),
        );
        ui.add(egui::Slider::new(&mut shader_settings.brightness, 0.0..=30.0).text("Brightness"));
        ui.separator();
        ui.heading("Visible Model");
        ui.radio_value(&mut *visible_model, VisibleModel::Coupe, "Coupe");
        ui.radio_value(&mut *visible_model, VisibleModel::Astro, "Astro");
        ui.radio_value(&mut *visible_model, VisibleModel::Torus, "Torus");
        ui.radio_value(&mut *visible_model, VisibleModel::Sphere, "Sphere");
        ui.separator();
        ui.heading("Color Source");

        ui.radio_value(
            &mut shader_settings.vertex_color_mode,
            0,
            "Use Material Color",
        );
        ui.radio_value(
            &mut shader_settings.vertex_color_mode,
            1,
            "Use Vertex Color",
        );

        ui.separator();
        ui.heading("Color");

        let mut color = shader_settings.color.to_linear().to_f32_array();
        if ui.color_edit_button_rgba_unmultiplied(&mut color).changed() {
            shader_settings.color = Color::rgba(color[0], color[1], color[2], color[3]);
        }

        ui.separator();
        ui.heading("Shader Visibility");
        ui.checkbox(&mut shader_settings.show_wireframe, "Show Wireframe");
        ui.checkbox(&mut shader_settings.show_outline, "Show Outline");
        ui.checkbox(&mut shader_settings.show_fill, "Show Fill");
    });

    // Update all OutlineMaterial instances
    for material_handle in outline_materials.iter() {
        if let Some(material) = outline_materials_assets.get_mut(material_handle) {
            material.outline_width = shader_settings.outline_width;
            material.brightness = shader_settings.brightness;
            material.vertex_color_mode = shader_settings.vertex_color_mode;
            material.color = shader_settings.color.to_linear().to_vec4();
            material.visibility = if shader_settings.show_outline {
                1.0
            } else {
                0.0
            };
        }
    }

    // Update all LineMaterial instances
    for material_handle in line_materials.iter() {
        if let Some(material) = line_materials_assets.get_mut(material_handle) {
            material.displacement = shader_settings.wireframe_displacement;
            material.brightness = shader_settings.brightness;
            material.vertex_color_mode = shader_settings.vertex_color_mode;
            material.color = shader_settings.color.to_linear().to_vec4();
            material.visibility = if shader_settings.show_wireframe {
                1.0
            } else {
                0.0
            };
        }
    }

    // Update all FillMaterial instances
    for material_handle in fill_materials.iter() {
        if let Some(material) = fill_materials_assets.get_mut(material_handle) {
            material.displacement = shader_settings.fill_displacement;
            material.shininess = shader_settings.fill_shininess;
            material.specular_strength = shader_settings.fill_specular_strength;
            material.vertex_color_mode = shader_settings.vertex_color_mode;
            material.color = shader_settings.color.to_linear().to_vec4();
            material.visibility = if shader_settings.show_fill { 1.0 } else { 0.0 };
        }
    }

    // Update visibility
    for mut visibility in visibility_set.p2().iter_mut() {
        *visibility = if shader_settings.show_wireframe {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for mut visibility in visibility_set.p1().iter_mut() {
        *visibility = if shader_settings.show_outline {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for mut visibility in visibility_set.p0().iter_mut() {
        *visibility = if shader_settings.show_fill {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn update_scene_visibility(
    visible_model: Res<VisibleModel>,
    mut coupe_query: Query<
        &mut Visibility,
        (
            With<CoupeSceneTag>,
            Without<AstroSceneTag>,
            Without<TorusSceneTag>,
            Without<SphereSceneTag>,
        ),
    >,
    mut astro_query: Query<
        &mut Visibility,
        (
            With<AstroSceneTag>,
            Without<CoupeSceneTag>,
            Without<TorusSceneTag>,
            Without<SphereSceneTag>,
        ),
    >,
    mut torus_query: Query<
        &mut Visibility,
        (
            With<TorusSceneTag>,
            Without<CoupeSceneTag>,
            Without<AstroSceneTag>,
            Without<SphereSceneTag>,
        ),
    >,
    mut sphere_query: Query<
        &mut Visibility,
        (
            With<SphereSceneTag>,
            Without<CoupeSceneTag>,
            Without<AstroSceneTag>,
            Without<TorusSceneTag>,
        ),
    >,
) {
    for mut visibility in coupe_query.iter_mut() {
        *visibility = if *visible_model == VisibleModel::Coupe {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for mut visibility in astro_query.iter_mut() {
        *visibility = if *visible_model == VisibleModel::Astro {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for mut visibility in torus_query.iter_mut() {
        *visibility = if *visible_model == VisibleModel::Torus {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for mut visibility in sphere_query.iter_mut() {
        *visibility = if *visible_model == VisibleModel::Sphere {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
