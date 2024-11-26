# Add or subtract selected edges to / from  an edge attribute called _VISIBLE_EDGES


import bpy
import bmesh
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
    
    # Important: Return the selection before cleaning up
    result = selection.copy()  # Make a copy of the selection
    
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
    
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    
    visible_layer = bm.edges.layers.int.get('_VISIBLE_EDGE')
    if visible_layer is None:
        visible_layer = bm.edges.layers.int.new('_VISIBLE_EDGE')
    
    selected_edges = [edge for edge in bm.edges if edge.select]
    for edge in selected_edges:
        edge[visible_layer] = 1
    
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
    
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    
    visible_layer = bm.edges.layers.int.get('_VISIBLE_EDGE')
    if visible_layer is None:
        visible_layer = bm.edges.layers.int.new('_VISIBLE_EDGE')
    
    selected_edges = [edge for edge in bm.edges if edge.select]
    for edge in selected_edges:
        edge[visible_layer] = 0
    
    bm.to_mesh(obj.data)
    bm.free()
    
    if was_in_edit_mode:
        bpy.ops.object.mode_set(mode='EDIT')

# Operators
class VIEW3D_OT_add_visible_edges(Operator):
    bl_idname = "view3d.add_visible_edges"
    bl_label = "Add Visible Edges"
    bl_description = "Mark selected edges as visible"
    
    def execute(self, context):
        for obj in context.selected_objects:
            add_to_visible_edges(obj)
        return {'FINISHED'}

class VIEW3D_OT_remove_visible_edges(Operator):
    bl_idname = "view3d.remove_visible_edges"
    bl_label = "Remove Visible Edges"
    bl_description = "Mark selected edges as not visible"
    
    def execute(self, context):
        for obj in context.selected_objects:
            remove_from_visible_edges(obj)
        return {'FINISHED'}

# Panel
class VIEW3D_PT_edge_visibility(Panel):
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = 'Edge Visibility'
    bl_label = "Edge Visibility"
    
    def draw(self, context):
        layout = self.layout
        
        row = layout.row()
        row.operator("view3d.add_visible_edges", text="Add Visible Edges")
        
        row = layout.row()
        row.operator("view3d.remove_visible_edges", text="Remove Visible Edges")

# Registration
classes = (
    VIEW3D_OT_add_visible_edges,
    VIEW3D_OT_remove_visible_edges,
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