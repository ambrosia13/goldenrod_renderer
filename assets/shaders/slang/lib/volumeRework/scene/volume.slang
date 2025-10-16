public enum VolumeDensityProfile: uint {
    Uniform,
    Noise,
    FadedNoise,
    Radial,
}

public struct VolumeParticle {
    // Bindless storage buffer handles
    public StructuredBuffer<float>.Handle scatteringCrossSectionCoefficients;
    public StructuredBuffer<float>.Handle absorptionCrossSectionCoefficients;

    // Henyey-greenstein G 
    public float phaseAsymmetry;

    public VolumeDensityProfile densityProfile;
    public float baseDensity;

    // used only for radial density profile, uses normalized radius
    public StructuredBuffer<float>.Handle densities;
}

public struct VolumeParticleSample {
    public float scatteringCrossSectionCoefficient;
    public float absorptionCrossSectionCoefficient;

    public float phaseAsymmetry;
    public float density;
}