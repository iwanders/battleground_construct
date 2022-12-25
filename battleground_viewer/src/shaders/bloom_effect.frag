uniform sampler2D emissive_buffer;

in vec2 uvs;
layout (location = 0) out vec4 color;

const int size = 3;
const float separation = 1.0;
const float amount = 0.75;
const float boost = 1.1;
const float threshold = 0.0001;

void main()
{
    vec2 emissive_buffer_size = textureSize(emissive_buffer, 0);
    // Sample with a kernel around the current fragment
    vec4 accum = vec4(0.0);
    float used_samples = 0.0;
    for(int x = -size; x <= size; x++)
    {
        for(int y = -size; y <= size; y++)
        {
            vec2 sample_uvs = (vec2(x, y) * separation + gl_FragCoord.xy) / emissive_buffer_size;
            vec4 c = texture(emissive_buffer, sample_uvs);
            float grey = max(c.r, max(c.g, c.b));
            if(grey < threshold)
            {
                c = vec4(0.0);
            }
            accum += c;
            used_samples += 1.0;
        }
    }

    accum /= used_samples;
    color = mix(texture(emissive_buffer, uvs), accum * boost, amount);
}
