extends Node

var tcp_client : StreamPeerTCP
var connected : bool = false

func _ready():
	tcp_client = StreamPeerTCP.new()
	connect_to_server("127.0.0.1", 3000)


func connect_to_server(address: String, port: int) -> void:
	var err = tcp_client.connect_to_host(address, port)
	tcp_client.poll()
	if err == OK:
		connected = true
		print("Connected to server!")
	else:
		print("Failed to connect to server: ", err)


func send_message(message: String) -> void:
	if connected:
		tcp_client.put_utf8_string(message)
		print("Message sent: ", message)
	else:
		print("Not connected to server.")

func receive_message() -> void:
	if connected:
		var received = tcp_client.get_available_bytes()
		if received > 0:
			var message = tcp_client.get_utf8_string(received)
			print("Message received: ", message)

func _process(delta: float) -> void:
	if connected:
		receive_message()

func disconnect_from_server() -> void:
	if connected:
		tcp_client.close()
		connected = false
		print("Disconnected from server.")
