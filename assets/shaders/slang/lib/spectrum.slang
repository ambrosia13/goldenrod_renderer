module spectrum;

import header;
import noise;

public struct Spectrum {
    RWTexture1D<float4> wavelengthToXyzLut;
    RWTexture1D<float4> rgbToSpectralIntensityLut;

    static const float WAVELENGTH_TO_XYZ_LUT_START = 360.0;
    static const float WAVELENGTH_TO_XYZ_LUT_END = 360.0 + 471.0;
    static const float WAVELENGTH_TO_XYZ_LUT_STEP = 1.0;

    static const float RGB_TO_SPECTRAL_INTENSITY_LUT_START = 380.0;
    static const float RGB_TO_SPECTRAL_INTENSITY_LUT_END = 780.0;
    static const float RGB_TO_SPECTRAL_INTENSITY_LUT_STEP = 5.0;

    static const float WAVELENGTH_RANGE_START = 380.0;
    static const float WAVELENGTH_RANGE_END = 780.0;

    public float generateWavelength(inout Random random) {
        return random.getFloat() * (WAVELENGTH_RANGE_END - WAVELENGTH_RANGE_START) + WAVELENGTH_RANGE_START;
    }

    public float3 spectralRadianceToRgb(float wavelength, float radiance) {
        float translated = rcp(WAVELENGTH_TO_XYZ_LUT_STEP) * clamp(
            (wavelength - WAVELENGTH_TO_XYZ_LUT_START),
            0.0,
            WAVELENGTH_TO_XYZ_LUT_END - WAVELENGTH_TO_XYZ_LUT_START
        );

        int icoord = int(translated);
        float fcoord = fract(translated);

        float3 xyz = lerp(this.wavelengthToXyzLut.Load(icoord).xyz, this.wavelengthToXyzLut.Load(icoord + 1).xyz, fcoord);
        float3 color = xyzToRgb(xyz);

        float luminousEfficacy = 683.0;
        float pdf = 1.0 / (WAVELENGTH_RANGE_END - WAVELENGTH_RANGE_START);

        return color * radiance * luminousEfficacy / pdf;
    }

    public float rgbToSpectralRadiance(float wavelength, float3 radiance) {
        float translated = rcp(RGB_TO_SPECTRAL_INTENSITY_LUT_STEP) * clamp(
            (wavelength - RGB_TO_SPECTRAL_INTENSITY_LUT_START),
            0.0,
            RGB_TO_SPECTRAL_INTENSITY_LUT_END - RGB_TO_SPECTRAL_INTENSITY_LUT_START
        );

        int icoord = int(translated);
        float fcoord = fract(translated);

        float3 sample = lerp(this.rgbToSpectralIntensityLut.Load(icoord).xyz, this.rgbToSpectralIntensityLut.Load(icoord + 1).xyz, fcoord);
        float radiance = dot(radiance, sample);

        return radiance;
    }
}