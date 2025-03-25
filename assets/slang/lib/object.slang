module object;

import intersect;
import material;

public struct Objects {
    public StructuredBuffer<Material> materials;
    public StructuredBuffer<Sphere> spheres;
    public StructuredBuffer<Aabb> aabbs;
    public StructuredBuffer<Triangle> triangles;
}

public struct ObjectCounts {
    public uint sphereCount;
    public uint aabbCount;
    public uint triangleCount;
}

public struct Sphere: Hittable {
    public float3 position;
    public float radius;
    public uint materialIndex;

    public Hit getHit(Ray ray) {
        Hit hit;
        hit.materialIndex = this.materialIndex;

        float3 originToCenter = ray.pos - this.position;

        float b = dot(originToCenter, ray.dir);
        float a = dot(ray.dir, ray.dir);
        float c = dot(originToCenter, originToCenter) - this.radius * this.radius;

        let determinant = b * b - a * c;

        if (determinant >= 0.0) {
            float sqrtDeterminant = sqrt(determinant);
            float nearT = (-b - sqrtDeterminant) / a;
            float farT = (-b + sqrtDeterminant) / a;

            let thing = float(b > 0.0);
        }
    }
}

public struct Aabb: Hittable {
    public float3 min;
    public float3 max;
    public uint materialIndex;

    public Hit getHit(Ray ray) {
        Hit hit;
        return hit;
    }
}

public struct Triangle: Hittable {
    public float3 a;
    public float3 b;
    public float3 c;
    public uint materialIndex;

    public Hit getHit(Ray ray) {
        Hit hit;
        return hit;
    }
}