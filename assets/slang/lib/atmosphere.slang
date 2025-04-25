namespace DeltaTrackCollision {
    public static const uint NULL = 0;
    public static const uint BOUNDARY = 1;
    public static const uint RAYLEIGH_SCATTERING = 2;
    public static const uint MIE_SCATTERING = 3;
    public static const uint OZONE_ABSORPTION = 4;
}

import header;
import intersect;
import object;
import noise;
import space;

public struct Atmosphere {
    public ConstantBuffer<CelestialBody> celestialBody;
    public AtmosphereComposition atmosphereComposition;
    public RWTexture1D<float> solarIrradiance;

    public float getSolarIrradiance(float wavelength) {
        int coord = int(wavelength - 380.0);
        return solarIrradiance.Load(coord);
    }
}

public struct CelestialBody {
    public float3 planetPos;
    public float planetRadius;
    public float atmosphereRadius;

    public float albedo;

    public float sunAngularRadius;

    public Sphere getPlanetShell() {
        return Sphere(planetPos, planetRadius);
    }

    public Sphere getAtmosphereShell() {
        return Sphere(planetPos, atmosphereRadius);
    }

    public float getAltitude(float3 position) {
        return distance(position, this.planetPos) - this.planetRadius;
    }

    public float getNormalizedAltitude(float3 position) {
        return clamp(getAltitude(position) / (atmosphereRadius - planetRadius), 0.0, 1.0);
    }

    public float3 sampleSunDirection(Ray ray, inout Random random, out float pdf) {
        float r1 = random.getFloat();
        float r2 = random.getFloat();

        
    }
}

public struct AtmosphereComposition {
    // These are read-write textures only because we want them to compile to storage textures on WGSL.

    // row 0: rayleigh
    // row 1: mie
    // row 2: ozone
    // columns: normalized, 0 means sea level, 1 means edge of atmosphere
    RWTexture2D<float> densities;

    // row 0: rayleigh scattering
    // row 1: rayleigh absorption
    // row 2: mie scattering
    // row 3: mie absorption
    // row 4: ozone absorption (zero if none)
    // columns: 400 columns, starting from wavelength 380 nm, ending at 780 nm
    RWTexture2D<float> coefficients;

    public AtmosphereSample getBaseSample(float wavelength) {
        // where to sample in the coefficients texture
        int sampleColumn = clamp(int(wavelength) - 380, 0, 400);

        float rayleighScatteringCoefficient = coefficients.Load(int2(sampleColumn, 0));
        float rayleighAbsorptionCoefficient = coefficients.Load(int2(sampleColumn, 1));
        float mieScatteringCoefficient = coefficients.Load(int2(sampleColumn, 2));
        float mieAbsorptionCoefficient = coefficients.Load(int2(sampleColumn, 3));
        float ozoneAbsorptionCoefficient = coefficients.Load(int2(sampleColumn, 4));

        Coefficients rayleigh = Coefficients(rayleighScatteringCoefficient, rayleighAbsorptionCoefficient);
        Coefficients mie = Coefficients(mieScatteringCoefficient, mieAbsorptionCoefficient);
        Coefficients ozone = Coefficients(0.0, ozoneAbsorptionCoefficient);

        return AtmosphereSample(rayleigh, mie, ozone);
    }

    public void scale(inout AtmosphereSample sample, in float normalizedAltitude) {
        int numColumns;
        int _numRows;
        int _numLevels;
        densities.GetDimensions(0, numColumns, _numRows, _numLevels);

        int sampleColumn = int(normalizedAltitude * float(numColumns));

        float rayleighDensity = densities.Load(int2(sampleColumn, 0));
        float mieDensity = densities.Load(int2(sampleColumn, 1));
        float ozoneDensity = densities.Load(int2(sampleColumn, 2));

        sample.rayleigh.scattering *= rayleighDensity;
        sample.rayleigh.absorption *= rayleighDensity;
        sample.mie.scattering *= mieDensity;
        sample.mie.absorption *= mieDensity;
        sample.ozone.absorption *= ozoneDensity;
    }
}

static const float earthAlbedo = 0.3;

static const float earthRadius = 6.360e6;
static const float atmosphereRadius = 6.460e6;

// 500 meters above the ground
static const float3 planetPos = float3(0.0, -earthRadius - 500.0, 0.0);

static const Sphere earthShell = Sphere(float3(0.0), earthRadius);
static const Sphere atmosphereShell = Sphere(float3(0.0), atmosphereRadius);

float altitude(float3 position) {
    return distance(position, planetPos) - earthRadius;
}

float getRayleighPhase(float cosTheta) {
    return (1.0 / (4.0 * PI)) * (3.0 / 4.0) * (1.0 - pow2(cosTheta));
}

float getRayleighDensity(float altitude) {
    const float scale = 8000.0;
    return exp(-altitude / scale);
}

float getMieDensity(float altitude) {
    const float scale = 1200.0;
    return exp(-altitude / scale);
}

float getOzoneDensity(float altitude) {
    float x = (max(0.0, altitude) - 22000.0) / 6092.04;
    return 2.0 / (exp(-x) + exp(x));
}

float getSunIrradiance(float wavelength) {
    return pow(1510.51 / wavelength, 5.0) / (exp(2444.32 / wavelength) - 1.0);
}

public struct Coefficients {
    float scattering;
    float absorption;

    public __init(float scattering, float absorption) {
        this.scattering = scattering;
        this.absorption = absorption;
    }

    static Coefficients rayleigh(float wavelength) {
        float scattering = 2.504e-5 * 3.7307e10 / pow(wavelength - 13.6111, 4.0);
        float absorption = 0.0;

        // sigma_s
        return Coefficients(scattering, absorption);
    }

    static Coefficients mie(float wavelength) {
        float turbidity = 1.0;

        float A = 0.6544 * turbidity - 0.6510;
        float B = (166.504891453 - 0.833005669719 * wavelength) / (1.0 - 0.00546759 * wavelength);

        float scattering = A * B / (wavelength * wavelength);
        float absorption = 0.11 * scattering;

        // sigma_s
        return Coefficients(scattering, absorption);
    }

    static Coefficients ozone(float wavelength) {
        float scattering = 0.0;
        float absorption = 1.2 * 4.86e18 * 1e-4 * 4.76296e-21 * exp(-pow((wavelength - 592.281) / 76.7895, 2.0));

        return Coefficients(scattering, absorption);
    }

    // sigma_t
    float extinction() {
        return this.scattering + this.absorption;
    }
}

public struct AtmosphereSample {
    Coefficients rayleigh;
    Coefficients mie;
    Coefficients ozone;

    float rayleighDensity;
    float mieDensity;
    float ozoneDensity;

    static AtmosphereSample seaLevel(float wavelength) {
        return AtmosphereSample(
            Coefficients.rayleigh(wavelength), 
            Coefficients.mie(wavelength), 
            Coefficients.ozone(wavelength)
        );
    }

    // Assumes densities to be 1.0
    public __init(Coefficients rayleigh, Coefficients mie, Coefficients ozone) {
        this.rayleigh = rayleigh;
        this.mie = mie;
        this.ozone = ozone;

        this.rayleighDensity = 1.0;
        this.mieDensity = 1.0;
        this.ozoneDensity = 1.0;
    }

    __init(float wavelength, float altitude) {
        var baseSample = AtmosphereSample.seaLevel(wavelength);
        baseSample.scale(altitude);

        return baseSample;
    }

    [mutating]
    void scale(float altitude) {
        this.rayleighDensity = getRayleighDensity(altitude);
        this.rayleigh.scattering *= this.rayleighDensity;
        this.rayleigh.absorption *= this.rayleighDensity;

        this.mieDensity = getMieDensity(altitude);
        this.mie.scattering *= this.mieDensity;
        this.mie.absorption *= this.mieDensity;

        this.ozoneDensity = getOzoneDensity(altitude);
        this.ozone.scattering *= this.ozoneDensity;
        this.ozone.absorption *= this.ozoneDensity;
    }

    float extinction() {
        return rayleigh.extinction() + mie.extinction() + ozone.extinction();
    }
}

float getInteractionDistance(float majorantExtinction, inout Random random) {
    return -log(1.0 - random.getFloat()) / majorantExtinction;
}

uint deltaTrack(Ray ray, AtmosphereSample baseSample, float wavelength, inout Random random, float tMax, out float t) {
    float freePathCoefficient = baseSample.rayleigh.extinction() + baseSample.mie.extinction() + baseSample.ozone.extinction();

    for (int i = 0; i < 256; ++i) {
        // march along ray
        float dt = getInteractionDistance(freePathCoefficient, random);
        t += dt;

        if (t > tMax) {
            return DeltaTrackCollision.BOUNDARY;
        }

        float3 pos = ray.pos + ray.dir * t;

        float altitude = altitude(pos);

        AtmosphereSample sample = baseSample;
        sample.scale(altitude);

        float totalExtinction = sample.extinction();

        if (random.getFloat() > totalExtinction / freePathCoefficient) {
            // null collision, keep ray going
            continue;
        }

        float r = random.getFloat() * totalExtinction;

        if (r < sample.rayleigh.extinction()) {
            return DeltaTrackCollision.RAYLEIGH_SCATTERING;
        } else if (r < sample.mie.extinction()) {
            return DeltaTrackCollision.MIE_SCATTERING;
        } else {
            return DeltaTrackCollision.OZONE_ABSORPTION;
        }
    }

    return DeltaTrackCollision.NULL;
}

float ratioTrack(Ray ray, AtmosphereSample baseSample, float wavelength, inout Random random, float tMax, out float t) {
    float transmittance = 1.0;

    Coefficients baseRayleigh = Coefficients.rayleigh(wavelength);
    Coefficients baseMie = Coefficients.mie(wavelength);
    Coefficients baseOzone = Coefficients.ozone(wavelength);

    float freePathCoefficient = baseRayleigh.extinction() + baseMie.extinction() + baseOzone.extinction();

    for (int i = 0; i < 256; ++i) {
        // march along ray
        float dt = getInteractionDistance(freePathCoefficient, random);
        t += dt;

        if (t > tMax) {
            break;
        }

        float3 pos = ray.pos + ray.dir * t;

        float altitude = altitude(pos);

        AtmosphereSample sample = baseSample;
        sample.scale(altitude);

        float totalExtinction = sample.extinction();

        transmittance *= 1.0 - totalExtinction / freePathCoefficient;

    }

    return transmittance;
}

float pathtraceAtmosphere(Ray ray, float wavelength, inout Random random) {
    // skip path tracing if the ray is inside the earth bc wtf
    if (distance(ray.pos, earthShell.position) < earthRadius) {
        return 0.0;
    }

    bool isInsideAtmosphere = distance(ray.pos, atmosphereShell.position) < atmosphereRadius;

    float throughput = 1.0;
    float radiance = 0.0;

    AtmosphereSample baseSample = AtmosphereSample.seaLevel(wavelength);
    float sunIrradiance = getSunIrradiance(wavelength);

    const int maxBounces = 100;

    for (int i = 0; i < maxBounces; ++i) {
        Hit earthHit = ray.intersect(earthShell);
        Hit atmosphereHit = ray.intersect(atmosphereShell);
        Hit closestHit = Hit.merge(earthHit, atmosphereHit);

        // if we don't hit any part of the atmosphere, don't path trace
        if (!closestHit.success) {
            break;
        }

        Ray nextRay;

        float tMax = closestHit.distance;
        float t = 0.0;
        uint collisionType = deltaTrack(ray, baseSample, wavelength, random, tMax, t);

        float _t = 0.0;
        Ray sunRay = Ray(ray.pos, normalize(float3(0.1, 0.4, 0.3)));
        float sunTransmittance = ratioTrack(sunRay, baseSample, wavelength, random, tMax, _t);
        float sunEmission = sunIrradiance * sunTransmittance;
        
        if (collisionType == DeltaTrackCollision.NULL) {
            break; // no collision with any particle
        } else if (collisionType == DeltaTrackCollision.BOUNDARY) { // hit planet or atmosphere boundary
            if (earthHit.success) { // hit planet, using lambertian material                
                throughput *= earthAlbedo; // * brdf, = 1 / PI implicitly multiplied
                radiance += throughput * sunEmission;

                // bounce off the earth surface
                nextRay.pos = earthHit.position + earthHit.normal * 0.0001;
                nextRay.dir = random.getCosineVector(earthHit.normal);
            } else { // hit atmosphere edge, atmosphereHit.success
                if (isInsideAtmosphere) { // if we're exiting the atmosphere, exit the loop
                    break;
                } else {
                    // enter the atmosphere
                    // we have already confirmed there was an atmosphere hit
                    nextRay.pos = atmosphereHit.position - atmosphereHit.normal * 0.0001;
                    nextRay.dir = ray.dir;
                }
            }
        } else if (
            // any scattering
            collisionType == DeltaTrackCollision.RAYLEIGH_SCATTERING || 
            collisionType == DeltaTrackCollision.MIE_SCATTERING
        ) {
            float3 collisionPos = ray.pos + ray.dir * t;
            float cosTheta = dot(ray.dir, sunRay.dir);


        }

        // russian roulette path termination
        // probability is higher the lower the ray energy (throughput) is
        float inverseTerminationProbability = clamp(throughput, 0.0, 1.0);
        if (random.getFloat() > inverseTerminationProbability) {
            break;
        }

        // compensate for the energy lost with the terminated paths
        throughput *= 1.0 / inverseTerminationProbability;

        ray = nextRay;
    }
}

// float pathtraceAtmosphere(Ray ray, float wavelength, inout Random random) {
//     float throughput = 1.0;
//     float radiance = 0.0;

//     for (int i = 0; i < 5; ++i) {
//         // todo: russian roulette path termination

//         // add sun radiance * sun transmittance * throughput to radiance

//         switch (deltaTrack(ray, wavelength, random)) {
//             case DeltaTrackCollision.GROUND: {
//                 // bounce ray off ground surface, multiply throughput by brdf of earth material
//             }
//             case DeltaTrackCollision.RAYLEIGH_SCATTERING: {
//                 // sample ray direction from rayleigh phase, multiply throughput by pdf
//                 // update ray with new direction
//             }
//             case DeltaTrackCollision.MIE_SCATTERING: {
//                 // sample ray direction from mie phase, multiply throughput by pdf
//                 // update ray with new direction
//             }
//             // ozone, idk
//             // null, do nothing? break out of loop because no more collision?
//         }
//     }

//     return radiance;
// }