module material;

import header;
import intersect;
import space;
import noise;

float3 ggxNormal(float3 normal, float roughness, inout Random random) {
    float r1 = random.getFloat();
    float r2 = random.getFloat();

    float a = roughness * roughness;

    float phi = 2.0 * PI * r1;

    float cosTheta = sqrt((1.0 - r2) / (1.0 + (a * a - 1.0) * r2));
    float sinTheta = sqrt(1.0 - cosTheta * cosTheta);

    float x = sinTheta * cos(phi);
    float y = sinTheta * sin(phi);
    float z = cosTheta;

    float3 roughNormal = mul(float3(x, y, z), getTbnMatrix(normal));

    return roughNormal;
}

public namespace MaterialType {
    public static const uint LAMBERTIAN = 0;
    public static const uint METAL = 1;
    public static const uint DIELECTRIC = 3;
}

public struct Material {
    public float3 albedo;
    public float roughness;
    public float emission;
    public float ior;
    public uint type;

    public float3 getAlbedo() {
        return this.albedo * step(this.emission, 0.0001);
    }

    public float3 getEmission() {
        return this.albedo * this.emission;
    }

    public float3 evaluateBrdf(in Hit hit, inout Random random, out Ray nextRay) {
        if (this.type == MaterialType.LAMBERTIAN) {
            float3 brdf = this.getAlbedo() / PI;
            nextRay = Ray(hit.position + hit.normal * 0.0001, ggxNormal(hit.normal, 1.0, random));

            return brdf;
        } else {
            nextRay = Ray(float3(0.0), float3(0.0));
            return float3(0.0);
        }
    }
}