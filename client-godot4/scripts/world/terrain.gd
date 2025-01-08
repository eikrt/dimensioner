extends Node3D

var mesh_instance
var chunk_size = 16
var thread: Thread = null
var mi: MeshInstance3D = MeshInstance3D.new()
func _ready():
	add_child(mi)
	generate_chunk() # Replace with function body.
func generate_chunk():
	if Globals.current_chunks == [{}]:
		return
	var mesh = ArrayMesh.new()
	var plane_mesh = PlaneMesh.new()
	plane_mesh.size = Vector2(chunk_size, chunk_size)

	plane_mesh.subdivide_depth = chunk_size
	plane_mesh.subdivide_width = chunk_size
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, plane_mesh.get_mesh_arrays())
	var mdt = MeshDataTool.new()
	mdt.create_from_surface(mesh, 0)
	for i in range(mdt.get_vertex_count()):
		var vertex = mdt.get_vertex(i)
		if not Globals.current_chunks == [{}] and not Globals.current_chunks == null:
			print(Globals.current_chunks[0].coords.x)
			vertex.y += Globals.current_chunks[0].tiles[i / 3].coords.z / 10
		mdt.set_vertex(i, vertex)
	mesh.clear_surfaces()
	mdt.commit_to_surface(mesh)
	for c in Globals.current_chunks:
		var mi = MeshInstance3D.new()
		mi.mesh = mesh
		mi.material_override = preload("res://materials/terrain/terrain.tres")
		mi.create_trimesh_collision()
		mi.position = Vector3(8 + c.coords.x * 16,0,8 + c.coords.y * 16)
		call_deferred("set_mesh", mi)
	
func set_mesh(mi):
	self.mi = mi
	add_child(mi)
func _process(delta):
	if get_child_count() > 9:
		for i in get_children().size():
			var child = get_child(i)
		var child = get_child(1)
		remove_child(child)
	generate_chunk_async()

# Function to start the thread
func generate_chunk_async() -> void:
	if thread == null:
		thread = Thread.new()
		thread.start(_process_chunks_in_thread.bind())

# Function executed in a separate thread
func _process_chunks_in_thread() -> void:
	generate_chunk()
	# Once processing is finished, notify the main thread
	call_deferred("_on_chunks_processed")

# This function is called once the thread has completed its task
func _on_chunks_processed() -> void:
	#print("All chunks processed!")
	# Ensure that the thread has finished processing before resetting
	if thread:
		thread.wait_to_finish()  # Wait for the thread to finish processing
		thread = null  # Clean up the thread reference
