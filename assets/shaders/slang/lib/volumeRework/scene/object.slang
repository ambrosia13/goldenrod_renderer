import lib.volumeRework.scene.handle;

public struct Object {
    public GeometryHandle geometry;
    public MaterialHandle mainMaterial;
    public MaterialHandle coatMaterial;
    public VolumeHandle innerVolume;
    public VolumeHandle outerVolume;
}