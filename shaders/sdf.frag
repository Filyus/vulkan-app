#version 450

layout(location = 0) in vec2 fragTexCoord;
layout(location = 1) in vec3 fragWorldPos;

layout(location = 0) out vec4 outColor;

// SDF Shape types
#define SPHERE 0
#define BOX 1
#define PLANE 2
#define TORUS 3
#define CYLINDER 4

// SDF Shape data (will be replaced by uniform buffer from ECS)
struct SDFShapeData {
    int shapeType;
    vec3 position;
    float size;
    vec4 params; // Additional parameters
    vec3 color;
    float metallic;
    float roughness;
    float emission;
    int padding;
};

// Light data
struct LightData {
    vec3 position;
    vec3 color;
    float intensity;
    int padding;
};

// Temporary hardcoded data for testing
SDFShapeData shapes[3];
LightData lights[1];

// SDF distance functions
float sdSphere(vec3 p, float r) {
    return length(p) - r;
}

float sdBox(vec3 p, vec3 b) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

float sdPlane(vec3 p, vec4 n) {
    // n.xyz must be normalized
    return dot(p, n.xyz) + n.w;
}

float sdTorus(vec3 p, vec2 t) {
    vec2 q = vec2(length(p.xz) - t.x, p.y);
    return length(q) - t.y;
}

float sdCylinder(vec3 p, vec2 h) {
    vec2 d = abs(vec2(length(p.xz), p.y)) - h;
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}

// Scene SDF function
float map(vec3 p) {
    float minDist = 1000.0;
    
    for (int i = 0; i < 3; i++) {
        vec3 localPos = p - shapes[i].position;
        float dist = 1000.0;
        
        switch (shapes[i].shapeType) {
            case SPHERE:
                dist = sdSphere(localPos, shapes[i].size);
                break;
            case BOX:
                dist = sdBox(localPos, vec3(shapes[i].size));
                break;
            case PLANE:
                dist = sdPlane(localPos, vec4(0.0, 1.0, 0.0, 0.0));
                break;
            case TORUS:
                dist = sdTorus(localPos, vec2(shapes[i].size, shapes[i].params.x));
                break;
            case CYLINDER:
                dist = sdCylinder(localPos, vec2(shapes[i].size, shapes[i].params.x));
                break;
        }
        
        minDist = min(minDist, dist);
    }
    
    return minDist;
}

// Get normal using gradient
vec3 getNormal(vec3 p) {
    vec2 e = vec2(0.001, 0.0);
    vec3 n = vec3(
        map(p + e.xyy) - map(p - e.xyy),
        map(p + e.yxy) - map(p - e.yxy),
        map(p + e.yyx) - map(p - e.yyx)
    );
    return normalize(n);
}

// Ray marching
float rayMarch(vec3 ro, vec3 rd, float maxDist) {
    float precis = 0.001;
    float h = precis * 2.0;
    float t = 0.0;
    
    for (int i = 0; i < 100; i++) {
        if (abs(h) < precis || t > maxDist) break;
        h = map(ro + rd * t);
        t += h;
    }
    
    return t;
}

// Calculate lighting
vec3 calculateLighting(vec3 pos, vec3 normal, vec3 viewDir, vec3 color, float metallic, float roughness) {
    vec3 finalColor = vec3(0.0);
    
    for (int i = 0; i < 1; i++) {
        vec3 lightDir = normalize(lights[i].position - pos);
        vec3 lightColor = lights[i].color * lights[i].intensity;
        
        // Check for shadows
        float shadowDist = rayMarch(pos + normal * 0.01, lightDir, 10.0);
        float shadow = shadowDist < length(lights[i].position - pos) ? 0.3 : 1.0;
        
        // Diffuse
        float diff = max(dot(normal, lightDir), 0.0);
        vec3 diffuse = diff * lightColor * color;
        
        // Specular (simplified)
        vec3 reflectDir = reflect(-lightDir, normal);
        float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32.0);
        vec3 specular = spec * lightColor * metallic;
        
        finalColor += (diffuse + specular) * shadow;
    }
    
    // Ambient
    vec3 ambient = color * 0.1;
    
    return ambient + finalColor;
}

// Push constants for window data
layout(push_constant) uniform PushConstants {
    vec2 uResolution;
    float uTime;
    float uAspectRatio;
} pushConstants;

void main() {
    // Initialize shapes (temporary - will come from ECS)
    shapes[0] = SDFShapeData(SPHERE, vec3(0.0, 0.0, 0.0), 0.5, vec4(0.0), vec3(1.0, 0.0, 0.0), 0.0, 0.5, 0.0, 0);
    shapes[1] = SDFShapeData(BOX, vec3(-1.5, 0.0, 0.0), 0.3, vec4(0.0), vec3(0.0, 1.0, 0.0), 0.0, 0.5, 0.0, 0);
    shapes[2] = SDFShapeData(SPHERE, vec3(1.5, 0.0, 0.0), 0.4, vec4(0.0), vec3(0.0, 0.0, 1.0), 0.0, 0.5, 0.0, 0);
    
    // Initialize lights (temporary - will come from ECS)
    lights[0] = LightData(vec3(2.0, 2.0, 2.0), vec3(1.0, 1.0, 1.0), 1.0, 0);
    
    // Use aspect ratio from push constants
    float aspectRatio = pushConstants.uAspectRatio;
    
    // Proper ray setup for SDF rendering with aspect ratio correction
    vec2 uv = fragTexCoord * 2.0 - 1.0; // Convert to [-1, 1] range
    uv.x *= aspectRatio; // Apply aspect ratio correction to prevent stretching
    
    vec3 ro = vec3(0.0, 0.0, -2.0); // Fixed camera position
    vec3 rd = normalize(vec3(uv, 1.0)); // Ray direction with aspect ratio correction
    
    // Ray marching
    float maxDist = 10.0;
    float t = rayMarch(ro, rd, maxDist);
    
    vec3 color = vec3(0.1, 0.1, 0.2); // Background color
    
    if (t < maxDist) {
        vec3 pos = ro + rd * t;
        vec3 normal = getNormal(pos);
        vec3 viewDir = normalize(-rd);
        
        // Find which shape we hit and get its material
        for (int i = 0; i < 3; i++) {
            vec3 localPos = pos - shapes[i].position;
            float dist = 1000.0;
            
            switch (shapes[i].shapeType) {
                case SPHERE:
                    dist = sdSphere(localPos, shapes[i].size);
                    break;
                case BOX:
                    dist = sdBox(localPos, vec3(shapes[i].size));
                    break;
            }
            
            if (abs(dist) < 0.01) {
                color = calculateLighting(pos, normal, viewDir, shapes[i].color, shapes[i].metallic, shapes[i].roughness);
                break;
            }
        }
    }
    
    outColor = vec4(color, 1.0);
}