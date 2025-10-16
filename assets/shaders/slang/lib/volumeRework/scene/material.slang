public enum MaterialType: uint {
    Lambertian = 0,
    Metal,
    Dielectric,
}

public struct Material {
    public MaterialType type;
    public uint hasAlbedoTexture;
    public uint hasRoughnessTexture;
    public uint hasEmissionTexture;

    public float4 albedo; // alpha has opacity

    public float roughness;
    public float emission;
    public float ior;

    public Texture2D<float4>.Handle albedoTexture; // alpha has opacity
    public Texture2D<float4>.Handle normalTexture; // alpha has roughness
}