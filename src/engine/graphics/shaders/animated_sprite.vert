#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;

struct Sprite {
    vec3 position;
    vec4 dimensions;
    uint frame;
};

layout(set = 0, binding = 1) uniform InputData {
    mat4 view;
    mat4 projection;
};

layout(set = 0, std430, binding = 2) buffer spriteSSBO {
    int spriteCount;
    Sprite sprites[];
};

layout(location = 0) out vec3 texCoords;

void main()
{
    Sprite sprite = sprites[gl_InstanceIndex];

    vec2 anchor = sprite.dimensions.xy;
    vec2 scale = sprite.dimensions.zw;

    vec3 offsetPos = position - vec3(anchor, 0);

    // Map 0 to -1 and 1 to 1.
    // Offset direction based on the position of the mesh vertex.
    // This makes sure the half-texel offset stays inside the bounds of the texture.
    // The half-texel offset prevents texture bleed at the edges of a sprtite due to floating point precision issues.
    // This is inteded for a square mesh with vertices (0, 0), (0, 1), (1, 0), (1, 1).
    // Any other mesh will have unexpected resutls
    vec2 offsetDirection = (position.xy - 0.5) * 2;

    vec3 translation = sprite.position;

    mat4 scaleMatrix = transpose(mat4(
        scale.x,    0,          0,  0,
        0,          scale.y,    0,  0,
        0,          0,          1,  0,
        0,          0,          0,  1
    ));

    // Inverse of view matrix
    mat4 rotationMatrix = mat4(inverse(mat3(view)));

    mat4 translationMatrix = transpose(mat4(
        1,          0,          0,  translation.x,
        0,          1,          0,  translation.y,
        0,          0,          1,  translation.z,
        0,          0,          0,  1
    ));

    mat4 model = translationMatrix * rotationMatrix * scaleMatrix;

    mat4 final_matrix = projection * view * model;

    gl_Position = final_matrix * vec4(offsetPos, 1);
    gl_Position.z = (gl_Position.z + gl_Position.w) * 0.5;

    texCoords = vec3(uv, sprite.frame);
}