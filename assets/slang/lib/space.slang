module space;

public float3 fromScreenSpace(float3 screenSpacePos, float4x4 matrix) {
    float3 clipSpacePos = screenSpacePos * 2.0 - 1.0;
    float4 temp = mul(matrix, float4(clipSpacePos, 1.0));
    return temp.xyz / temp.w;
}

public float3 toScreenSpace(float3 pos, float4x4 matrix) {
    float4 temp = mul(matrix, float4(pos, 1.0));
    return (temp.xyz / temp.w) * 0.5 + 0.5;
}

public float3x3 getTbnMatrix(float3 normal) {
    float3 reference = normalize(float3(1.0, 0.5, 0.25));

    float3 tangent = cross(reference, normal);
    float3 bitangent = cross(normal, tangent);

    return float3x3(tangent, bitangent, normal);
}