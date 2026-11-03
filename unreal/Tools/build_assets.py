"""Rebuild the small M2 kit from engine primitives; no Python in the cooked game."""
import unreal

assets = unreal.AssetToolsHelpers.get_asset_tools()
library = unreal.EditorAssetLibrary

def material(name, color):
    path = '/Game/Kit/' + name
    mat = unreal.load_asset(path)
    if not mat:
        mat = assets.create_asset(name, '/Game/Kit', unreal.Material, unreal.MaterialFactoryNew())
        node = unreal.MaterialEditingLibrary.create_material_expression(mat, unreal.MaterialExpressionConstant3Vector)
        node.set_editor_property('constant', unreal.LinearColor(*color))
        unreal.MaterialEditingLibrary.connect_material_property(node, '', unreal.MaterialProperty.MP_BASE_COLOR)
        rough = unreal.MaterialEditingLibrary.create_material_expression(mat, unreal.MaterialExpressionConstant)
        rough.set_editor_property('r', .9)
        unreal.MaterialEditingLibrary.connect_material_property(rough, '', unreal.MaterialProperty.MP_ROUGHNESS)
        unreal.MaterialEditingLibrary.recompile_material(mat)
        library.save_loaded_asset(mat)
    mat.set_editor_property('used_with_instanced_static_meshes', True)
    mat.set_editor_property('used_with_nanite', True)
    if name == 'Ground':
        mat.set_editor_property('two_sided', True)
    unreal.MaterialEditingLibrary.recompile_material(mat)
    library.save_loaded_asset(mat)
    return mat

for name, source, color in [
    ('Timber', 'Cylinder', (.17,.08,.035,1)),
    ('EarthWall', 'Cube', (.48,.31,.16,1)),
    ('Thatch', 'Cone', (.39,.27,.07,1)),
    ('Citizen', 'Cylinder', (.22,.32,.44,1)),
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
if not unreal.load_asset('/Game/Kit/Ambient'):
    source = unreal.load_asset('/Engine/MapTemplates/Sky/DaylightAmbientCubemap')
    if not source:
        raise RuntimeError('Missing ambient cubemap in engine installation')
    ambient = assets.duplicate_asset('Ambient', '/Game/Kit', source)
    library.save_loaded_asset(ambient)
unreal.log('TCE_ASSETS_OK')
