extends CharacterBody3D

# Sensitivity and speed settings
@export var mouse_sensitivity: float = 0.002
@export var move_speed: float = 5.0
@export var jump_force: float = 100.0

# Camera node reference
@onready var camera = $Camera3D
var captured = false
# Internal variables for mouse look
var pitch: float = 0.0

func _ready():
	# Capture the mouse for FPS-style control
	Input.set_mouse_mode(Input.MOUSE_MODE_CAPTURED)

func _physics_process(delta):
	_handle_input(delta)
	move_and_slide()
	if not captured:
		Input.set_mouse_mode(Input.MOUSE_MODE_VISIBLE)
	else:
		Input.set_mouse_mode(Input.MOUSE_MODE_CAPTURED)
func _handle_input(delta):
	# Handle mouse look
	var mouse_delta = Input.get_last_mouse_velocity()
	rotation_degrees.y -= mouse_delta.x * mouse_sensitivity
	pitch -= mouse_delta.y * mouse_sensitivity
	pitch = clamp(pitch, -90.0, 90.0)  # Limit pitch to avoid flipping
	camera.rotation_degrees.x = pitch

	# Handle movement
	velocity = Vector3.ZERO
	if Input.is_action_pressed("move_forward"):
		velocity -= transform.basis.z
	if Input.is_action_pressed("move_backward"):
		velocity += transform.basis.z
	if Input.is_action_pressed("move_left"):
		velocity -= transform.basis.x
	if Input.is_action_pressed("move_right"):
		velocity += transform.basis.x
	
	velocity = velocity.normalized() * move_speed
	velocity.y = velocity.y  # Keep the y-velocity for jumping or falling
	velocity.y = velocity.y - 1 * delta  # Gravity simulation
	# Jumping
	if is_on_floor() and Input.is_action_just_pressed("jump"):
		velocity.y = jump_force
	Globals.player_data.coords[0] = global_position.x * 16 #/ Globals.TILE_SIZE
	Globals.player_data.coords[1] = global_position.z * 16 #/ Globals.TILE_SIZE
	Globals.player_data.coords[2] = global_position.y * 16 #/ Globals.TILE_SIZE
func _input(event):
	if Input.is_action_just_pressed("free_cursor"):
		captured = not captured
