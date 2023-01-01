uniform sampler2D depthTexture;
uniform mat4 viewProjectionInverse;

uniform vec4 surfaceColor;

in vec3 pos;
layout (location = 0) out vec4 outColor;


// Lifted from shared.frag in three-d
vec3 srgb_from_rgb(vec3 rgb) {
	vec3 a = vec3(0.055, 0.055, 0.055);
	vec3 ap1 = vec3(1.0, 1.0, 1.0) + a;
	vec3 g = vec3(2.4, 2.4, 2.4);
	vec3 ginv = 1.0 / g;
	vec3 select = step(vec3(0.0031308, 0.0031308, 0.0031308), rgb);
	vec3 lo = rgb * 12.92;
	vec3 hi = ap1 * pow(rgb, ginv) - a;
	return mix(lo, hi, select);
}

// Lifted from shared.frag in three-d
vec3 world_pos_from_depth(mat4 viewProjectionInverse, float depth, vec2 uv) {
    vec4 clipSpacePosition = vec4(uv * 2.0 - 1.0, depth * 2.0 - 1.0, 1.0);
    vec4 position = viewProjectionInverse * clipSpacePosition;
    return position.xyz / position.w;
}

void main()
{
    outColor = surfaceColor;

    #ifdef USE_VERTEX_COLORS
    outColor *= col;
    #endif

    ivec2 depthSize = textureSize(depthTexture, 0);
    vec2 uv = gl_FragCoord.xy / depthSize;

    // Determine distance between background and fence
    vec3 backgroundPosition = world_pos_from_depth(viewProjectionInverse, texture(depthTexture, uv).x, uv);
    vec3 fencePosition = pos;
    float d = distance(fencePosition, backgroundPosition);

    // Use distance to blend in fence near things
    // float f = exp(-11.09 * pow(d, 4));
    const float lineWidth = 0.1;
    float fade = exp(-2.0 * (d + 0.2));
    float step = floor(-d + lineWidth) + 1.0;
    outColor.a = max(fade, step);

    // Convert color space
    outColor.rgb = srgb_from_rgb(outColor.rgb);
}
