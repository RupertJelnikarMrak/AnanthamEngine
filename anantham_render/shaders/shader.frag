#version 460

layout(location = 0) in vec4 inColor;
layout(location = 1) in vec3 inPos;

layout(location = 0) out vec4 outColor;

void main() {
    // 1. Calculate the face normal using screen-space derivatives
    // We negate the cross product to account for Vulkan's inverted Y-axis screen space
    vec3 normal = normalize(-cross(dFdx(inPos), dFdy(inPos)));

    // 2. Define the "Sun"
    // Pointing diagonally down. You can eventually pass this in via a Uniform Buffer!
    vec3 lightDir = normalize(vec3(0.5, 1.0, 0.8));

    // 3. Calculate Diffuse Light
    // dot() returns 1.0 if the face looks directly at the sun, 0.0 if perpendicular
    float diff = max(dot(normal, lightDir), 0.0);

    // 4. Add Ambient Light
    // So the faces in the shadows aren't pitch black
    float ambient = 0.3;
    float lightIntensity = diff + ambient;

    // 5. Apply the light to the color!
    outColor = vec4(inColor.rgb * lightIntensity, inColor.a);
}
