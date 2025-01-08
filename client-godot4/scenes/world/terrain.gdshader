shader_type spatial;

// Adjustable parameters
uniform float yellow_threshold = 100.0; // Height where terrain starts transitioning to green
uniform float green_threshold = 0.0; // Height where terrain starts transitioning to grey
uniform float blend_sharpness = 5.0; // Controls the smoothness of blending between colors

void vertex() {
    // Optional: Add custom vertex logic here
}

void fragment() {
    // Get the world position's height
    float height = VERTEX.y;

    // Calculate blend weights for colors
    float yellow_weight = smoothstep(yellow_threshold - blend_sharpness, yellow_threshold, height);
    float green_weight = smoothstep(yellow_threshold, green_threshold, height);
    float grey_weight = smoothstep(green_threshold, green_threshold + blend_sharpness, height);

    // Normalize weights
    float total_weight = yellow_weight + green_weight + grey_weight;
    yellow_weight /= total_weight;
    green_weight /= total_weight;
    grey_weight /= total_weight;

    // Define colors
    vec3 yellow_color = vec3(1.0, 1.0, 0.0); // Yellow
    vec3 green_color = vec3(0.0, 1.0, 0.0);  // Green
    vec3 grey_color = vec3(0.5, 0.5, 0.5);   // Grey

    // Blend colors based on weights
    vec3 albedo = yellow_color * yellow_weight +
                  green_color * green_weight +
                  grey_color * grey_weight;

    // Lighting (basic diffuse lighting)
    vec3 light_dir = normalize(vec3(0.5, 1.0, 0.5));
    vec3 normal = normalize(NORMAL);
    float diffuse = max(dot(normal, light_dir), 0.0);

    // Output
    ALBEDO = albedo;
    //DIFFUSE_LIGHT = diffuse;
}