module material;

import header;
import intersect;
import space;
import noise;
import spectrum;

float3 cosineSample(float3 normal, inout Random random) {
    float r1 = random.getFloat();
    float r2 = random.getFloat();

    float phi = 2.0 * PI * r1;

    float cosTheta = sqrt(1.0 - r2);
    float sinTheta = sqrt(r2);

    float x = sinTheta * cos(phi);
    float y = sinTheta * sin(phi);
    float z = cosTheta;

    return mul(float3(x, y, z), getTbnMatrix(normal));
}

float3 ggxSample(float3 normal, float roughness, inout Random random) {
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
    public static const uint DIELECTRIC = 2;
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

    public float evaluateBrdf(Hit hit, float wavelength, in Spectrum spectrum, inout Random random, out Ray nextRay) {
        if (this.type == MaterialType.LAMBERTIAN) {
            float brdf = spectrum.rgbToSpectralRadiance(wavelength, this.getAlbedo());
            nextRay = Ray(hit.position + hit.normal * 0.0001, random.getCosineVector(hit.normal));

            // cosine sampling implicitly multiplies by the pdf, so no need to divide by pi
            return brdf;
        } else {
            nextRay = Ray(float3(0.0), float3(0.0));
            return 0.0;
        }
    }
}