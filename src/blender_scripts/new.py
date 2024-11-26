import bpy
import bmesh
import json
from bpy.types import Panel, Operator

def get_selected_edges(obj):
    if obj.type != 'MESH':
        return []
        
    was_in_edit_mode = (obj.mode == 'EDIT')
    if was_in_edit_mode:
        bpy.ops.object.mode_set(mode='OBJECT')
    
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    
    selection = [edge for edge in bm.edges if edge.select]
    result = selection.copy()
    
    bm.free()
    
    if was_in_edit_mode:
        bpy.ops.object.mode_set(mode='EDIT')
        
    return result

def add_to_visible_edges(obj):
    if obj.type != 'MESH':
        return
            
    was_in_edit_mode = (obj.mode == 'EDIT')
    if was_in_edit_mode:
        bpy.ops.object.mode_set(mode='OBJECT')
    
    # Create BMesh from the object
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    
    # Get or create the '_VISIBLE_EDGE' layer
    visible_layer = bm.edges.layers.int.get('_VISIBLE_EDGE')
    if visible_layer is None:
        visible_layer = bm.edges.layers.int.new('_VISIBLE_EDGE')
    
    selected_edges = [edge for edge in bm.edges if edge.select]
    for edge in selected_edges:
        edge[visible_layer] = 1
        edge.smooth = False  # Mark edge as sharp
    
    # Update the mesh
    bm.to_mesh(obj.data)
    bm.free()
    
    if was_in_edit_mode:
        bpy.ops.object.mode_set(mode='EDIT')

def remove_from_visible_edges(obj):
    if obj.type != 'MESH':
        return
            
    was_in_edit_mode = (obj.mode == 'EDIT')
    if was_in_edit_mode:
        bpy.ops.object.mode_set(mode='OBJECT')
    
    # Create BMesh from the object
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    
    # Get or create the '_VISIBLE_EDGE' layer
    visible_layer = bm.edges.layers.int.get('_VISIBLE_EDGE')
    if visible_layer is None:
        visible_layer = bm.edges.layers.int.new('_VISIBLE_EDGE')
    
    selected_edges = [edge for edge in bm.edges if edge.select]
    for edge in selected_edges:
        edge[visible_layer] = 0
        edge.smooth = True  # Unmark edge as sharp
    
    # Update the mesh
    bm.to_mesh(obj.data)
    bm.free()
    
    if was_in_edit_mode:
        bpy.ops.object.mode_set(mode='EDIT')

def clear_visible_edges(obj):
    if obj.type != 'MESH':
        return
    
    was_in_edit_mode = (obj.mode == 'EDIT')
    if was_in_edit_mode:
        bpy.ops.object.mode_set(mode='OBJECT')
    
    # Create BMesh from the object
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    
    # Get the '_VISIBLE_EDGE' layer, if it exists
    visible_layer = bm.edges.layers.int.get('_VISIBLE_EDGE')
    if visible_layer is not None:
        for edge in bm.edges:
            edge[visible_layer] = 0  # Unmark as visible
            edge.smooth = True       # Unmark edge as sharp
        # Optionally, remove the layer
        bm.edges.layers.int.remove(visible_layer)
    else:
        # Even if the layer doesn't exist, ensure edges are unsharp
        for edge in bm.edges:
            edge.smooth = True
    
    # Update the mesh
    bm.to_mesh(obj.data)
    bm.free()
    
    if was_in_edit_mode:
        bpy.ops.object.mode_set(mode='EDIT')

def get_visible_edges_indices(obj):
    if obj.type != 'MESH':
        print(f"Skipping {obj.name}: Not a mesh object")
        return None
        
    primitive_index = obj.get("gltf_primitive_index", -1)
    
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bm.verts.ensure_lookup_table()
    bm.edges.ensure_lookup_table()
    
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

    visible_edges = [e for e in bm.edges if e[visible_edge_layer] == 1]
    
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
            if line_list:
                all_visible_edges[str(primitive_index)] = line_list
    
    if all_visible_edges:
        json_string = json.dumps(all_visible_edges)
        bpy.context.scene["gltf_all_selected_edges"] = json_string
        print(f"Stored visible edges data for all objects in the scene")
        print(f"JSON data: {json_string}")
    else:
        print("No visible edges found in any object")

def store_primitive_extras(obj):
    visible_edges_indices = get_visible_edges_indices(obj)
    if visible_edges_indices is None:
        print("no visible edges found")
        return
    
    primitive_index, line_list = visible_edges_indices
    
    # Create the extras structure
    extras = {
        "visibleEdges": line_list
    }
    
    # Store as JSON string in custom property
    obj["gltf_primitive_extras"] = json.dumps(extras)
    print(f"Stored visible edges data for primitive {primitive_index}")

def print_primitive_extras(obj):
    if "gltf_primitive_extras" not in obj:
        print(f"No GLTF extras found on object {obj.name}")
        return
        
    try:
        extras = json.loads(obj["gltf_primitive_extras"])
        print(f"GLTF extras for {obj.name}:")
        print(json.dumps(extras, indent=2))
    except json.JSONDecodeError:
        print(f"Invalid JSON data stored in extras for {obj.name}")

def clear_primitive_extras(obj):
    if "gltf_primitive_extras" in obj:
        del obj["gltf_primitive_extras"]
        print(f"Cleared GLTF extras from object {obj.name}")
    else:
        print(f"No GLTF extras found on object {obj.name}")

# Operators
class VIEW3D_OT_add_visible_edges(Operator):
    bl_idname = "view3d.add_visible_edges"
    bl_label = "Add Visible Edges"
    bl_description = "Mark selected edges as visible and sharp"
    
    def execute(self, context):
        for obj in context.selected_objects:
            add_to_visible_edges(obj)
        return {'FINISHED'}

class VIEW3D_OT_remove_visible_edges(Operator):
    bl_idname = "view3d.remove_visible_edges"
    bl_label = "Remove Visible Edges"
    bl_description = "Mark selected edges as not visible and unsharp"
    
    def execute(self, context):
        for obj in context.selected_objects:
            remove_from_visible_edges(obj)
        return {'FINISHED'}

class VIEW3D_OT_clear_visible_edges(Operator):
    bl_idname = "view3d.clear_visible_edges"
    bl_label = "Clear All Visible Edges"
    bl_description = "Unmark all edges as visible and unsharp in selected objects"
    
    def execute(self, context):
        for obj in context.selected_objects:
            clear_visible_edges(obj)
        return {'FINISHED'}

class VIEW3D_OT_generate_edge_json(Operator):
    bl_idname = "view3d.generate_edge_json"
    bl_label = "Export Edge List"
    bl_description = "Generate JSON line list from visible edges"
    
    def execute(self, context):
        create_and_store_json_line_list()
        return {'FINISHED'}

class VIEW3D_OT_store_primitive_extras(Operator):
    bl_idname = "view3d.store_primitive_extras"
    bl_label = "Store Edge Data"
    bl_description = "Store visible edges as GLTF primitive extras"
    
    def execute(self, context):
        for obj in context.selected_objects:
            store_primitive_extras(obj)
        return {'FINISHED'}

class VIEW3D_OT_print_primitive_extras(Operator):
    bl_idname = "view3d.print_primitive_extras" 
    bl_label = "Print Edge Data"
    bl_description = "Print the GLTF extras stored on selected objects"

    def execute(self, context):
        for obj in context.selected_objects:
            print_primitive_extras(obj)
        return {'FINISHED'}

class VIEW3D_OT_clear_primitive_extras(Operator):
    bl_idname = "view3d.clear_primitive_extras"
    bl_label = "Clear Edge Data" 
    bl_description = "Clear any GLTF extras stored on selected objects"
    
    def execute(self, context):
        for obj in context.selected_objects:
            clear_primitive_extras(obj)
        return {'FINISHED'}

# Panel
class VIEW3D_PT_edge_visibility(Panel):
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = 'Edge Visibility'
    bl_label = "Edge Visibility"
    
    def draw(self, context):
        layout = self.layout
        
        box = layout.box()
        box.label(text="Edge Selection")
        row = box.row()
        row.operator("view3d.add_visible_edges", text="Add Visible Edges")
        row = box.row()
        row.operator("view3d.remove_visible_edges", text="Remove Visible Edges")
        row = box.row()
        row.operator("view3d.clear_visible_edges", text="Clear All Visible Edges")
        
        box = layout.box()
        box.label(text="GLTF Extras")
        row = box.row()
        row.operator("view3d.store_primitive_extras", text="Store Edge Data")
        row = box.row()
        row.operator("view3d.print_primitive_extras", text="Print Edge Data")
        row = box.row()
        row.operator("view3d.clear_primitive_extras", text="Clear Edge Data")
        
        box = layout.box()
        box.label(text="Export")
        row = box.row()
        row.operator("view3d.generate_edge_json", text="Export Edge List")

# Registration
classes = (
    VIEW3D_OT_add_visible_edges,
    VIEW3D_OT_remove_visible_edges,
    VIEW3D_OT_clear_visible_edges,
    VIEW3D_OT_generate_edge_json,
    VIEW3D_OT_store_primitive_extras,
    VIEW3D_OT_print_primitive_extras,
    VIEW3D_OT_clear_primitive_extras,
    VIEW3D_PT_edge_visibility
)

def register():
    for cls in classes:
        bpy.utils.register_class(cls)

def unregister():
    for cls in classes:
        bpy.utils.unregister_class(cls)

if __name__ == "__main__":
    register()