module object;

public interface Hittable {
    Hit getHit(float3 rayOrigin, float3 rayDirection);
}

public struct Hit {
    public float3 position;
    public float distance;
    public float3 normal;
    public uint materialIndex;
}

public struct Sphere {
    public float3 position;
    public float radius;
    public uint materialIndex;
}

public struct Aabb {
    public float3 min;
    public float3 max;
    public uint materialIndex;
}

public struct Triangle {
    public float3 a;
    public float3 b;
    public float3 c;
    public uint materialIndex;
}