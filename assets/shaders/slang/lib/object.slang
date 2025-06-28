module object;

import intersect;
import material;

public struct Objects: Hittable {
    public StructuredBuffer<Material> materials;
    public StructuredBuffer<Sphere> spheres;
    public StructuredBuffer<Aabb> aabbs;
    public StructuredBuffer<Triangle> triangles;

    public bool isUnhittable() {
        return true;
    }

    public Hit getHit(Ray ray) {
        Hit hit;

        // Start iterating from index 1, because the 0th element is garbage data

        for (int i = 1; i < this.spheres.getCount(); ++i) {
            // If we encounter a null value, we break out of the loop
            if (spheres[i].isUnhittable()) {
                break;
            }

            hit = Hit.merge(hit, ray.intersect(spheres[i]));
        }

        for (int i = 1; i < this.aabbs.getCount(); ++i) {
            if (aabbs[i].isUnhittable()) {
                break;
            }

            hit = Hit.merge(hit, ray.intersect(aabbs[i]));
        }

        for (int i = 1; i < this.triangles.getCount(); ++i) {
            if (triangles[i].isUnhittable()) {
                break;
            }

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

    // For when we want a sphere not related to the material path tracing
    public __init(float3 position, float radius) {
        this.position = position;
        this.radius = radius;
        this.materialIndex = uint(-1);
    }

    public bool isUnhittable() {
        return this.radius <= 0.0 || this.materialIndex == uint.maxValue;
    }

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
    public float3 boundsMin;
    public float3 boundsMax;
    public uint materialIndex;

    public bool isUnhittable() {
        return any(this.boundsMax <= this.boundsMin) || this.materialIndex == uint.maxValue;
    }

    public Hit getHit(Ray ray) {
        Hit hit;
        hit.materialIndex = this.materialIndex;

        bool inside = all(clamp(ray.pos, this.boundsMin, this.boundsMax) == ray.pos);

        float3 tMin = (this.boundsMin - ray.pos) / ray.dir;
        float3 tMax = (this.boundsMax - ray.pos) / ray.dir;

        float3 t1 = min(tMin, tMax);
        float3 t2 = max(tMin, tMax);

        float tNear = max3(t1.x, t1.y, t1.z);
        float tFar = min3(t2.x, t2.y, t2.z);

        if (!inside) {
            hit.success = !(tNear >= tFar || tFar <= 0.0);
            hit.distance = tNear;

            bool3 eq = t1 == float3(tNear);
            hit.normal = float3(float(eq.x), float(eq.y), float(eq.z)) * sign(-ray.dir);
        } else {
            hit.success = true;
            hit.distance = tFar;
            
            bool3 eq = t2 == float3(tFar);
            hit.normal = float3(float(eq.x), float(eq.y), float(eq.z)) * sign(-ray.dir);
        }

        hit.position = ray.pos + ray.dir * hit.distance;

        return hit;
    }
}

public struct Triangle: Hittable {
    public float3 a;
    public float3 b;
    public float3 c;
    public uint materialIndex;

    public bool isUnhittable() {
        return all(this.a == this.b) || all(this.b == this.c) || all(this.a == this.c) || this.materialIndex == uint.maxValue;
    }

    public Hit getHit(Ray ray) {
        Hit hit;
        return hit;
    }
}