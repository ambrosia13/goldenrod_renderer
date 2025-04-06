module space;

public float3x3 getTbnMatrix(float3 normal) {
    float3 reference = normalize(float3(1.0, 0.5, 0.25));

    float3 tangent = cross(reference, normal);
    float3 bitangent = cross(normal, tangent);

    return float3x3(tangent, bitangent, normal);
}