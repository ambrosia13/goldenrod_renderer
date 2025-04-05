module object;

import intersect;
import material;

public struct Objects: Hittable {
    public StructuredBuffer<Material> materials;
    public StructuredBuffer<Sphere> spheres;
    public StructuredBuffer<Aabb> aabbs;
    public StructuredBuffer<Triangle> triangles;

    public Hit getHit(Ray ray) {
        Hit hit;

        for (int i = 0; i < this.spheres.getCount(); ++i) {
            hit = Hit.merge(hit, ray.intersect(spheres[i]));
        }

        for (int i = 0; i < this.aabbs.getCount(); ++i) {
            hit = Hit.merge(hit, ray.intersect(aabbs[i]));
        }

        for (int i = 0; i < this.triangles.getCount(); ++i) {
            hit = Hit.merge(hit, ray.intersect(triangles[i]));
        }

        return hit;
    }
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

            float solutions[2] = { 
                (-b - sqrtDeterminant) / a, 
                (-b + sqrtDeterminant) / a 
            };

            float t = solutions[0] > 0.0 ? solutions[0] : solutions[1];

            if (t > 0.0) {
                float3 hitPosition = ray.pos + ray.dir * t;
                float3 outwardNormal = normalize(hitPosition - this.position);

                float dirDotNormal = dot(ray.dir, outwardNormal);
                bool frontFace = dirDotNormal < 0.0;

                float3 normal = outwardNormal * -sign(dirDotNormal);

                hit.success = true;
                hit.position = hitPosition;
                hit.normal = normal;
                hit.distance = t;
            }
        }

        return hit;
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