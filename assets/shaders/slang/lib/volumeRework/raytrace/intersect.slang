import lib.volumeRework.scene.handle;

public struct Ray {
    public float3 pos;
    public float3 dir;

    public Hit intersect<T>(T object)
        where T: Hittable 
    {
        return object.getHit(this);
    }

    public float3 getPoint(float t) {
        return this.pos + this.dir * t;
    }
}

public struct Hit {
    public float3 position;
    public float distance;
    public float3 normal;
    public bool backFace;

    public bool success;

    public static const Hit NULL = Hit();

    private __init() {
        success = false;
    }

    public __init(float3 position, float distance, float3 normal, bool backFace) {
        success = true;
        this.position = position;
        this.distance = distance;
        this.normal = normal;
        this.backFace = backFace;
    }

    public float3 biasPosition() {
        return this.position + this.normal * 0.001;
    }

    public float3 biasPositionNegative() {
        return this.position - this.normal * 0.001;
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
    bool isUnhittable() {
        return false;
    }

    Hit getHit(Ray ray);
}