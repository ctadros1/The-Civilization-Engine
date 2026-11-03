"""Blender 5.2.3: author normalized reusable modules, never a particular settlement."""
import bpy
import json
import math
import random
from pathlib import Path

ROOT = Path(__file__).resolve().parent
OUT = ROOT / 'export'
OUT.mkdir(parents=True, exist_ok=True)
random.seed(2401)
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete(use_global=False)
bpy.context.scene.unit_settings.system = 'METRIC'
bpy.context.scene.unit_settings.scale_length = 1.0

def material(name, color):
    result = bpy.data.materials.new(name)
    result.diffuse_color = (*color, 1)
    return result

wood = material('TimberMaterial', (.20, .095, .035))
earth = material('EarthWallMaterial', (.52, .34, .17))
straw = material('ThatchMaterial', (.48, .34, .11))
stone = material('StoneMaterial', (.22, .20, .17))
modules = []

def cylinder(name, radius, depth, location=(0,0,0), rotation=(0,0,0), mat=wood, vertices=12):
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=depth, location=location, rotation=rotation)
    obj = bpy.context.object
    obj.name = name
    obj.data.materials.append(mat)
    return obj

def cube(name, size, location=(0,0,0), mat=earth):
    bpy.ops.mesh.primitive_cube_add(size=1, location=location)
    obj = bpy.context.object
    obj.name = name
    obj.scale = size
    obj.data.materials.append(mat)
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    return obj

def finish(name, objects):
    for existing in bpy.context.scene.objects: existing.select_set(False)
    for obj in objects: obj.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    bpy.ops.object.join()
    obj = bpy.context.object
    obj.name = name
    bpy.context.scene.cursor.location = (0,0,0)
    bpy.ops.object.origin_set(type='ORIGIN_CURSOR')
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    if not name.startswith('Thatch'):
        bevel = obj.modifiers.new('Worked edges', 'BEVEL')
        bevel.width = .009
        bevel.segments = 1
        bpy.ops.object.modifier_apply(modifier=bevel.name)
    modules.append(obj)
    bpy.ops.export_scene.fbx(filepath=str(OUT / (name + '.fbx')), use_selection=True,
        object_types={'MESH'}, axis_forward='X', axis_up='Z', apply_unit_scale=True,
        bake_anim=False, add_leaf_bones=False, use_mesh_modifiers=True, mesh_smooth_type='FACE')
    obj.hide_set(True)

finish('TimberPost', [cylinder('Hand-hewn post', .5, 1, vertices=10)])
finish('TimberBeam', [cylinder('Hand-hewn beam', .5, 1, rotation=(0,math.pi/2,0), vertices=10)])

rods=[]
for row in range(11):
    z=-.47+row*.094
    rods.append(cylinder('Woven horizontal', .026, .96, (0, .43 if row%2 else -.43,z), (0,math.pi/2,0),vertices=6))
for column in range(7):
    x=-.45+column*.15
    rods.append(cylinder('Upright stave', .035, .98, (x,0,0),vertices=6))
finish('WattlePanel',rods)
finish('DaubPanel',[cube('Earth infill',(1,1,1))])

def roof(name, angle):
    vertices=[]; faces=[]
    steps=max(8,round(96*angle/(2*math.pi)))
    rings=12
    # Overlapping conical courses give the thatch physical eaves and ridges.
    for ring in range(rings):
        t0=ring/rings; t1=(ring+1)/rings
        # The grammar describes the supporting cone. A thin outer thatch skin covers rafters.
        z0=t0-.5+.015; z1=t1-.5+.015
        r0=.512*(1-t0); r1=.512*(1-t1)
        start=len(vertices)
        for j in range(steps+1):
            a=angle*j/steps
            rough=.002*math.sin(j*5.13+ring*3.7)
            vertices.extend([(math.cos(a)*(r0+rough),math.sin(a)*(r0+rough),z0),
                             (math.cos(a)*r1,math.sin(a)*r1,z1+.003)])
        for j in range(steps):
            a=start+j*2; faces.append((a,a+2,a+3,a+1))
    mesh=bpy.data.meshes.new(name)
    mesh.from_pydata(vertices,[],faces); mesh.materials.append(straw)
    obj=bpy.data.objects.new(name,mesh); bpy.context.collection.objects.link(obj)
    finish(name,[obj])

roof('ThatchCone',2*math.pi)
roof('ThatchWedge',2*math.pi/16)
finish('EarthFloor',[cylinder('Beaten earth',.5,.02,mat=earth,vertices=48)])
finish('HearthBowl',[cylinder('Stone hearth',.5,1,mat=stone,vertices=12)])
finish('Plank',[cube('Worked plank',(1,1,1),mat=wood)])

for obj in modules: obj.hide_set(False)
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT / 'vernacular-kit.blend'))
manifest={'version':1,'blender':bpy.app.version_string,'nominal_unit_metres':1,
          'origin':'module centre; roof wedges rotate around full-cone centre',
          'modules':[{'name':obj.name,'file':'export/'+obj.name+'.fbx',
                      'vertices':len(obj.data.vertices),'polygons':len(obj.data.polygons)} for obj in modules]}
(ROOT/'kit.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
print('TCE_KIT_OK',len(modules))
