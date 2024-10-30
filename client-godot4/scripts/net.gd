extends Net


# Called when the node enters the scene tree for the first time.

# Function to request and parse chunk data
func fetch_chunk_from_server(x: float, y: float, index: int) -> Dictionary:
	var net = Net.new()
	var result = net.fetch_chunk(x, y, index)
	var json = JSON.new()
	# Ensure result is valid JSON before parsing
	if result is String:
		var json_result = json.parse(result)
		if json_result.error == OK:
			return json_result.result
		else:
			push_error("Failed to parse JSON: %s" % json_result.error_string)
			return {}
	else:
		push_error("Received invalid data from fetch_chunk.")
		return {}
func _ready() -> void:
	pass
	
# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass
