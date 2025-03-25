module intersect;

public struct Ray {
    public float3 pos;
    public float3 dir;

    public Hit intersect<T>(T object)
        where T: Hittable 
    {
        return object.getHit(this);
    }
}

public struct Hit {
    public bool success;
    public float3 position;
    public float distance;
    public float3 normal;
    public uint materialIndex;

    public __init() {
        success = false;
        position = float3(0.0);
        distance = 0.0;
        normal = float3(0.0);
        materialIndex = 0;
    }

    public static Hit merge(Hit a, Hit b) {
        Hit hit;

        if (a.success && b.success) {
            if (a.distance <= b.distance) {
                hit = a;
            } else {
                hit = b;
            }
        } else if (a.success && !b.success) {
            hit = a;
        } else if (b.success && !a.success) {
            hit = b;
        }

        return hit;
    }
}

public interface Hittable {
    Hit getHit(Ray ray);
}
