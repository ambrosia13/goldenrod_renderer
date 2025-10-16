import lib.volumeRework.scene.object;
import lib.volumeRework.scene.handle;
import lib.volumeRework.scene.geometry;
import lib.volumeRework.scene.material;
import lib.volumeRework.scene.volume;

import lib.intersect;

public struct Scene {
    public ConstantBuffer<SceneMetadata> metadata;

    public StructuredBuffer<Object> objects;
    
    public StructuredBuffer<Sphere> spheres;
    public StructuredBuffer<Aabb> aabbs;
    public StructuredBuffer<Triangle> triangles;

    public StructuredBuffer<Material> materials;
    public StructuredBuffer<VolumeParticle> volumeParticles;
}

public struct SceneMetadata {
    public VolumeHandle baseVolume;
    public uint useBvhSphere;
    public uint useBvhAabb;
    public uint useBvhTriangle;
}

public struct SceneHit {
    public Hit hit = Hit.NO_HIT;

    public MaterialHandle material;
    public VolumeHandle innerVolume;
    public VolumeHandle outerVolume;

    public __init(Hit hit, MaterialHandle material, VolumeHandle innerVolume, VolumeHandle outerVolume) {
        this.hit = hit;
        this.material = material;
        this.innerVolume = innerVolume;
        this.outerVolume = outerVolume;
    }
}