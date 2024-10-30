extends MeshInstance3D

var mat = self.get_active_material(0)
# Called when the node enters the scene tree for the first time.
var tile_x = []
var tile_y = []
var tile_height = []
var tile_type = []

var width = 64
var depth = 64
# Function to request and parse chunk data
func fetch_chunk_from_server(x: float, y: float, index: int) -> Dictionary:
	var net = Net.new()
	var result = net.fetch_chunk(x, y, index)
	var json = JSON.new()
	return json.parse_string(result)
func _ready() -> void:
	var chunk = fetch_chunk_from_server(0,0,0)
	var tiles = chunk.tiles
	self.mesh = generate_mesh_from_height_map(chunk.tiles, 1.0)
# Pass the arrays to the shader
	
func _process(delta: float) -> void:
	pass
func generate_mesh_from_height_map(height_map: Array, tile_size: float) -> ArrayMesh:
	var array_mesh = ArrayMesh.new()
# Vertex arrays to hold mesh data
	var vertices = PackedVector3Array()
	var indices = []
	var normals = PackedVector3Array()
	var uvs = PackedVector2Array()
	for x in range(width - 1):
		for z in range(depth - 1):
			# Define the four corners of the tile (each corner has an x, y, z position)
			var v0 = Vector3(x * tile_size, get_height(height_map, x, z), z * tile_size)
			var v1 = Vector3((x + 1) * tile_size, get_height(height_map, x + 1, z), z * tile_size)
			var v2 = Vector3(x * tile_size, get_height(height_map, x, z + 1), (z + 1) * tile_size)
			var v3 = Vector3((x + 1) * tile_size, get_height(height_map, x + 1, z + 1), (z + 1) * tile_size)

# Add the vertices in clockwise or counterclockwise order
			var current_vertex_count = vertices.size()
			vertices.append(v0)
			vertices.append(v1)
			vertices.append(v2)
			vertices.append(v3)
			indices.append(current_vertex_count)       # Triangle 1
			indices.append(current_vertex_count + 1)
			indices.append(current_vertex_count + 2)
			indices.append(current_vertex_count + 2)   # Triangle 2
			indices.append(current_vertex_count + 1)
			indices.append(current_vertex_count + 3)

			# Calculate normals (simple approach using cross product)
			var normal_1 = (v1 - v0).cross(v2 - v0).normalized()
			var normal_2 = (v2 - v3).cross(v1 - v3).normalized()
			normals.append(normal_1)
			normals.append(normal_1)
			normals.append(normal_2)
			normals.append(normal_2)

			# Set basic UV coordinates for each corner
			uvs.append(Vector2(0, 0))
			uvs.append(Vector2(1, 0))
			uvs.append(Vector2(0, 1))
			uvs.append(Vector2(1, 1))
	var mesh_arrays = []
	var packed_indices = PackedInt32Array(indices)
	mesh_arrays.resize(Mesh.ARRAY_MAX)
	mesh_arrays[Mesh.ARRAY_VERTEX] = vertices
	mesh_arrays[Mesh.ARRAY_INDEX] = packed_indices
	mesh_arrays[Mesh.ARRAY_NORMAL] = normals
	mesh_arrays[Mesh.ARRAY_TEX_UV] = uvs

# Add the data to the ArrayMesh
	array_mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, mesh_arrays)
	return array_mesh
	
func get_height(height_map, x: int, z: int) -> float:
	if x >= 0 and x < width and z >= 0 and z < depth:
		return height_map[x + z * width].height / 10
	else:
		print("Index out of bounds for height map at position: ", x, z)
		return 0  # Default to 0 if out of bounds
