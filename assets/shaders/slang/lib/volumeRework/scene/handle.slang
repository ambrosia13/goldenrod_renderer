public enum GeometryType: uint {
    Sphere = 0,
    Aabb,
    Triangle,
} 

public struct GeometryHandle {
    public const GeometryHandle NULL = GeometryHandle(uint.maxValue);

    // 8 most significant bits are for the geometry type
    // 24 least significant bits are for the index
    private uint data;

    private __init(uint data) {
        this.data = data;
    }

    public GeometryType getType() {
        return GeometryType((this.data >> 24) & 0xFF);
    }

    public uint getIndex() {
        return this.data & 0xFFFFFF;
    }

    public bool isNull() {
        return this.data == NULL.data;
    }
}

public struct MaterialHandle {
    public const MaterialHandle NULL = MaterialHandle(uint.maxValue);

    public uint index;

    public bool isNull() {
        return this.index == NULL.index;
    }
}

public struct VolumeHandle {
    public const VolumeHandle NULL = VolumeHandle(uint.maxValue);

    // 8 most significant bits are for the particle count
    // 24 least significant bits are for the particle starting index
    private uint data;

    private __init(uint data) {
        this.data = data;
    }

    public uint getParticleStartIndex() {
        return this.data & 0xFFFFFF;
    }

    public uint getParticleCount() {
        return (this.data >> 24) & 0xFF;
    }

    public bool isNull() {
        return this.data == NULL.data;
    }
}