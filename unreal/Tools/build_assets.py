"""Import the Blender kit and compile its materials; no Python in the cooked game."""
import unreal
import hashlib
import json
from pathlib import Path

assets = unreal.AssetToolsHelpers.get_asset_tools()
library = unreal.EditorAssetLibrary

def material(name, color, emissive=False):
    path = '/Game/Kit/' + name
    mat = unreal.load_asset(path)
    if not mat:
        mat = assets.create_asset(name, '/Game/Kit', unreal.Material, unreal.MaterialFactoryNew())
    if str(library.get_metadata_tag(mat, 'TCE.MaterialVersion')) != '2':
        # Reconnect outputs without deleting rooted expressions in a loaded UE 5.8 material.
        node = unreal.MaterialEditingLibrary.create_material_expression(mat, unreal.MaterialExpressionConstant3Vector)
        node.set_editor_property('constant', unreal.LinearColor(*color))
        unreal.MaterialEditingLibrary.connect_material_property(node, '', unreal.MaterialProperty.MP_BASE_COLOR)
        if name in ('Ground', 'Soil', 'ThatchMaterial', 'EarthWallMaterial', 'TimberMaterial', 'Water'):
            noise = unreal.MaterialEditingLibrary.create_material_expression(mat, unreal.MaterialExpressionNoise)
            noise.set_editor_property('scale', .001 if name == 'Ground' else .025)
            noise.set_editor_property('levels', 2)
            noise.set_editor_property('quality', 1)
            noise.set_editor_property('output_min', 0)
            noise.set_editor_property('output_max', 1)
            dark = unreal.MaterialEditingLibrary.create_material_expression(mat, unreal.MaterialExpressionConstant3Vector)
            dark.set_editor_property('constant', unreal.LinearColor(*(c*.6 for c in color[:3]), 1))
            blend = unreal.MaterialEditingLibrary.create_material_expression(mat, unreal.MaterialExpressionLinearInterpolate)
            unreal.MaterialEditingLibrary.connect_material_expressions(dark, '', blend, 'A')
            unreal.MaterialEditingLibrary.connect_material_expressions(node, '', blend, 'B')
            unreal.MaterialEditingLibrary.connect_material_expressions(noise, '', blend, 'Alpha')
            unreal.MaterialEditingLibrary.connect_material_property(blend, '', unreal.MaterialProperty.MP_BASE_COLOR)
        if emissive:
            unreal.MaterialEditingLibrary.connect_material_property(node, '', unreal.MaterialProperty.MP_EMISSIVE_COLOR)
        rough = unreal.MaterialEditingLibrary.create_material_expression(mat, unreal.MaterialExpressionConstant)
        rough.set_editor_property('r', .18 if name == 'Water' else .9)
        unreal.MaterialEditingLibrary.connect_material_property(rough, '', unreal.MaterialProperty.MP_ROUGHNESS)
        unreal.MaterialEditingLibrary.recompile_material(mat)
        library.set_metadata_tag(mat, 'TCE.MaterialVersion', '2')
        library.save_loaded_asset(mat)
    mat.set_editor_property('used_with_instanced_static_meshes', True)
    mat.set_editor_property('used_with_nanite', True)
    if name in ('Ground', 'ThatchMaterial'):
        mat.set_editor_property('two_sided', True)
    unreal.MaterialEditingLibrary.recompile_material(mat)
    library.save_loaded_asset(mat)
    return mat

for name, source, color in [
    ('Timber', 'Cylinder', (.17,.08,.035,1)),
    ('EarthWall', 'Cube', (.48,.31,.16,1)),
    ('Thatch', 'Cone', (.39,.27,.07,1)),
    ('Citizen', 'Cylinder', (.22,.32,.44,1)),
    ('Grass', 'Cone', (.12,.20,.055,1)),
    ('Head', 'Sphere', (.52,.34,.21,1)),
    ('Limb', 'Cylinder', (.29,.18,.10,1)),
]:
    path = '/Game/Kit/' + name
    mesh = unreal.load_asset(path)
    if not mesh:
        mesh = library.duplicate_asset('/Engine/BasicShapes/' + source, path)
    if not mesh:
        raise RuntimeError('Could not create kit mesh: ' + name)
    mesh.set_material(0, material(name + 'Material', color))
    if name != 'Citizen':
        settings = mesh.get_editor_property('nanite_settings')
        settings.set_editor_property('enabled', True)
        mesh.set_editor_property('nanite_settings', settings)
    library.save_loaded_asset(mesh)

material('Ground', (.20,.28,.105,1))
material('Soil', (.19,.115,.055,1))
material('Water', (.025,.13,.18,1))
material('StoneMaterial', (.22,.20,.17,1))
material('HearthGlow', (5,1.3,.12,1), emissive=True)

kit_root=Path(__file__).resolve().parents[2] / 'art' / 'vernacular'
if (kit_root/'kit.json').exists():
    kit=json.loads((kit_root/'kit.json').read_text(encoding='utf-8'))
    for entry in kit['modules']:
        source=kit_root/entry['file']
        digest='FbxFactory-v1:'+hashlib.sha256(source.read_bytes()).hexdigest()
        destination='/Game/Kit/'+entry['name']
        mesh=unreal.load_asset(destination)
        if not mesh or str(library.get_metadata_tag(mesh,'TCE.SourceSHA256')) != digest:
            task=unreal.AssetImportTask()
            task.filename=str(source)
            task.destination_path='/Game/Kit'
            task.destination_name=entry['name']
            task.automated=True
            task.replace_existing=True
            task.save=True
            task.factory=unreal.FbxFactory()
            options=unreal.FbxImportUI()
            options.import_mesh=True
            options.import_materials=False
            options.import_textures=False
            options.mesh_type_to_import=unreal.FBXImportType.FBXIT_STATIC_MESH
            options.static_mesh_import_data.combine_meshes=True
            options.static_mesh_import_data.auto_generate_collision=False
            task.options=options
            assets.import_asset_tasks([task])
            mesh=unreal.load_asset(destination)
            if not mesh:
                raise RuntimeError('Kit import failed: '+entry['name'])
            library.set_metadata_tag(mesh,'TCE.SourceSHA256',digest)
        settings=mesh.get_editor_property('nanite_settings')
        settings.set_editor_property('enabled',True)
        mesh.set_editor_property('nanite_settings',settings)
        for index,slot in enumerate(mesh.get_editor_property('static_materials')):
            name=str(slot.material_slot_name)
            mat=unreal.load_asset('/Game/Kit/'+name)
            if not mat:
                mat=unreal.load_asset('/Game/Kit/TimberMaterial')
            mesh.set_material(index,mat)
        library.save_loaded_asset(mesh)
        bounds=mesh.get_bounding_box()
        extent=bounds.max-bounds.min
        if max(extent.x,extent.y,extent.z)<95 or max(extent.x,extent.y,extent.z)>106:
            raise RuntimeError('Kit module is not in centimetres: '+entry['name']+' '+str(extent))
    unreal.log('TCE_KIT_IMPORT_OK '+str(len(kit['modules'])))
if not unreal.load_asset('/Game/Kit/Ambient'):
    source = unreal.load_asset('/Engine/MapTemplates/Sky/DaylightAmbientCubemap')
    if not source:
        raise RuntimeError('Missing ambient cubemap in engine installation')
    ambient = assets.duplicate_asset('Ambient', '/Game/Kit', source)
    library.save_loaded_asset(ambient)
unreal.log('TCE_ASSETS_OK')
