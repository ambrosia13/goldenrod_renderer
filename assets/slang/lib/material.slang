module material;

public namespace MaterialType {
    public static const uint LAMBERTIAN = 0;
    public static const uint METAL = 1;
    public static const uint DIELECTRIC = 3;
}

public struct Material {
    public float3 albedo;
    public float roughness;
    public float3 emission;
    public float ior;
    public uint type;
}