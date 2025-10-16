import lib.volumeRework.raytrace.intersect;

public struct Sphere: Hittable {
    public float3 position;
    public float radius;

    public override bool isUnhittable() {
        return this.radius <= 0.0;
    }

    public Hit getHit(Ray ray) {
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
                bool backFace = dirDotNormal > 0.0;

                return Hit(hitPosition, t, normal, backFace);
            }
        }

        return Hit.NULL;
    }
}

public struct Aabb: Hittable {
    public float3 minBounds;
    public float3 maxBounds;

    public override bool isUnhittable() {
        return any(this.maxBounds <= this.minBounds);
    }

    public Hit getHit(Ray ray) {
        bool insideAabb = all(clamp(ray.pos, this.minBounds, this.maxBounds) == ray.pos);

        float3 tMin = (this.minBounds - ray.pos) / ray.dir;
        float3 tMax = (this.maxBounds - ray.pos) / ray.dir;

        float3 t1 = min(tMin, tMax);
        float3 t2 = max(tMin, tMax);

        float tNear = max3(t1.x, t1.y, t1.z);
        float tFar = min3(t2.x, t2.y, t2.z);

        Hit hit = Hit.NULL;

        if (insideAabb) {
            // if inside the box, all rays will hit the box
            hit.success = true;
            hit.distance = tFar;
            hit.normal = float3(t2 == float3(tFar)) * -sign(ray.dir);
        } else {
            hit.success = !(tNear >= tFar || tFar <= 0.0);
            hit.distance = tNear;
            hit.normal = float3(t2 == float3(tNear)) * -sign(ray.dir);
        }

        hit.position = ray.pos + ray.dir * hit.distance;
        hit.backFace = insideAabb;
    }
}

public struct Triangle: Hittable {
    public float3 vertices[3];

    public override bool isUnhittable() {
        return 
            all(vertices[0] == vertices[1]) ||
            all(vertices[1] == vertices[2]) ||
            all(vertices[2] == vertices[0]);
    }

    public Hit getHit(Ray ray) {
        return Hit.NULL;
    }
}