## Generate json line list from "_VISIBLE_EDGES" edge attribute

import bpy
import bmesh
import json

def get_visible_edges_indices(obj):
    if obj.type != 'MESH':
        print(f"Skipping {obj.name}: Not a mesh object")
        return None

    primitive_index = obj.get("gltf_primitive_index", -1)
    
    # Create BMesh
    bm = bmesh.new()
    bm.from_mesh(obj.data)

    bm.verts.ensure_lookup_table()
    bm.edges.ensure_lookup_table()

    # Get the custom vertex indices and visible edges layers
    vert_index_layer = bm.verts.layers.int.get('_VERT_INDEX')
    visible_edge_layer = bm.edges.layers.int.get('_VISIBLE_EDGE')
    
    if vert_index_layer is None:
        print(f"Custom vertex index layer not found for {obj.name}")
        bm.free()
        return None
        
    if visible_edge_layer is None:
        print(f"Visible edge layer not found for {obj.name}")
        bm.free()
        return None

    # Get edges marked as visible
    visible_edges = [e for e in bm.edges if e[visible_edge_layer] == 1]

    # Create line list from visible edges
    line_list = []
    for edge in visible_edges:
        v1, v2 = edge.verts
        line_list.append([v1[vert_index_layer], v2[vert_index_layer]])

    bm.free()
    return primitive_index, line_list

def create_and_store_json_line_list():
    selected_objects = bpy.context.selected_objects

    if not selected_objects:
        print("No objects selected. Please select at least one object.")
        return

    all_visible_edges = {}

    for obj in selected_objects:
        result = get_visible_edges_indices(obj)
        if result is not None:
            primitive_index, line_list = result
            if line_list:  # Only add if there are visible edges
                all_visible_edges[str(primitive_index)] = line_list

    if all_visible_edges:
        json_string = json.dumps(all_visible_edges)
        
        # Store the JSON string as a custom property on the scene
        bpy.context.scene["gltf_all_selected_edges"] = json_string
        
        print(f"Stored visible edges data for all objects in the scene")
        print(f"JSON data: {json_string}")
    else:
        print("No visible edges found in any object")

# Run the function
create_and_store_json_line_list()