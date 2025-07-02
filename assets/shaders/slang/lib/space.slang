module space;

public float3x3 getTbnMatrix(float3 normal) {
    float3 reference = abs(normal.y) > 0.99 ? float3(1.0, 0.0, 0.0) : float3(0.0, 1.0, 0.0);

    float3 tangent = cross(reference, normal);
    float3 bitangent = cross(normal, tangent);

    return float3x3(tangent, bitangent, normal);
}