extends Net


# Called when the node enters the scene tree for the first time.

# Function to request and parse chunk data
func send():
	var json = JSON.new()
	var p = json.stringify(Globals.player_data)
	var chunks = self.transfer(p)
	var c = json.parse(chunks)
	var json_data = json.data
	if json_data == null:
		return null
	for e in json_data[0].entities:
		if e.index == Globals.player_data.id:
			Globals.player_data.ccoords[0] = e.ccoords.x
			Globals.player_data.ccoords[2] = e.ccoords.y
			Globals.player_data.ccoords[1] = e.ccoords.z
	return json_data
func _ready():
	send()
# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	var chunks = send()
	if chunks == null:
		return
	Globals.current_chunks = chunks
	
