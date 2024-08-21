#version 330

layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

in VS_OUT {
    vec3 normal;
} gs_in[];

const float MAGNITUDE = 3.0;

uniform mat4 projection;

void main()
{
    gl_Position = projection * (gl_in[0].gl_Position + vec4(gs_in[0].normal, 0.0) * MAGNITUDE);
    EmitVertex();
    gl_Position = projection * (gl_in[1].gl_Position + vec4(gs_in[1].normal, 0.0) * MAGNITUDE);
    EmitVertex();
    gl_Position = projection * (gl_in[2].gl_Position + vec4(gs_in[2].normal, 0.0) * MAGNITUDE);
    EmitVertex();
    EndPrimitive();
}