#version 460

layout(location = 0) in vec4 inColor;
layout(location = 1) in vec3 inNormal;

layout(location = 0) out vec4 outColor;

void main() {
    vec3 normal = normalize(inNormal);

    vec3 lightDir = normalize(vec3(0.5, 1.0, 0.8));
    float diff = dot(normal, lightDir);

    if (inColor.a < 1.0) {
        diff = abs(diff);
    } else {
        diff = max(diff, 0.0);
    }

    float ambient = 0.3;
    float lightIntensity = diff + ambient;

    outColor = vec4(inColor.rgb * lightIntensity, inColor.a);
}
