#version 430 core

layout(location = 0) in vec3 texCoords;

layout(binding = 0) uniform sampler3D frames;

layout(location = 0) out vec4 outColor;

void main()
{
    outColor = texture(frames, texCoords);

    if (outColor.a < 0.1)
        discard;
}