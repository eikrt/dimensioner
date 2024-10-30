shader_type spatial;

uniform float tile_x[64];       // x positions of tiles
uniform float tile_y[4096];       // y positions of tiles
uniform float tile_height[4096];  // heights of tiles     // types of tiles (0 = grass, 1 = water, 2 = sand)

uniform int num_tiles;           // number of tiles

void fragment() {
	vec2 uv = UV;
	vec3 color = vec3(0.3, 0.5, 0.3); // Default color

// Loop through each tile to find the closest one affecting this fragment
	for (int i = 0; i < num_tiles*num_tiles; i++) {
		vec2 tile_pos = vec2(tile_x[i], tile_y[i]);
		float dist = distance(uv, tile_pos);

// Calculate height influence based on distance
		float height_factor = smoothstep(0.3, 0.0, dist) * tile_height[i] / 10.0;

// Determine color based on tile type

		color += vec3(0.1, 0.8, 0.5) * height_factor; // Green

}

// Set the final color
	ALBEDO = vec3(color);
}
void vertex() {
	vec2 uv = UV;
	// Loop through each tile to find the closest one affecting this fragment
	for (int i = 0; i < num_tiles; i++) {
		vec2 tile_pos = vec2(tile_x[i], tile_y[i]);
		float dist = distance(uv, tile_pos);

	// Calculate height influence based on distance
		float height_factor = smoothstep(0.3, 0.0, dist) * tile_height[i] / 10.0;

	// Determine color based on tile type

		VERTEX.y -= 1.0 * height_factor / 1.0; // Green
	}
}
