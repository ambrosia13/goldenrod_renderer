namespace DeltaTrackCollision {
    public static const uint NULL = 0;
    public static const uint BOUNDARY = 1;
    public static const uint RAYLEIGH_SCATTERING = 2;
    public static const uint MIE_SCATTERING = 3;
    public static const uint OZONE_ABSORPTION = 4;
}

enum Collision {
    NULL, BOUNDARY, RAYLEIGH, MIE, OZONE,
}

import header;
import intersect;
import object;
import noise;
import space;

public struct Atmosphere {
    public AtmosphereBody body;
    public AtmosphereConstituent[] constituents;
}

public struct AtmosphereBody {
    public float3 planetPos;
    public float planetRadius;
    public float atmosphereRadius;

    public float albedo;

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

public struct CelestialLight {
    public Sphere shell;
    public Texture1D<float> irradiances;
}

public struct AtmosphereConstituent {
    public uint crossSectionScatteringCoefficients;
    public uint crossSectionAbsorptionCoefficients;
    public uint numberDensities;

    public float getCrossSectionScatteringCoefficient(float wavelength) {
        return 1.0;
    }

    public float getCrossSectionAbsorptionCoefficient(float wavelength) {
        return 1.0;
    }

    public float getNumberDensity(float altitude) {
        return 1.0;
    }
}

public struct AtmosphereComposition {
    public StructuredBuffer<Texture1D<float>> luts;
    public StructuredBuffer<AtmosphereConstituent> constituents;
}

static const float earthAlbedo = 0.15;

static const float earthRadius = 6.360e6;
static const float atmosphereRadius = 6.460e6;

// 500 meters above the ground
static const float3 planetPos = float3(0.0, -earthRadius - 500.0, 0.0);

static const Sphere earthShell = Sphere(planetPos, earthRadius);
static const Sphere atmosphereShell = Sphere(planetPos, atmosphereRadius);

float altitude(float3 position) {
    return distance(position, planetPos) - earthRadius;
}

float getRayleighPhase(float cosTheta) {
    return (1.0 / (4.0 * PI)) * (3.0 / 4.0) * (1.0 - pow2(cosTheta));
}

float getRayleighDensity(float altitude) {
    return exp(-pow(max(0.0, altitude), 1.13102) / 28377.2);
}

// TODO: find reference
float3 getRayleighDirection(float3 direction, inout Random random) {
    float3x3 tbn = getTbnMatrix(direction);

    float cosTheta;
    float u;

    do {
        cosTheta = lerp(-1.0, 1.0, random.getFloat());
        u = random.getFloat();
    } while (u > 3.0 / 4.0 * (1.0 + cosTheta * cosTheta));

    float sinTheta = sqrt(1.0 - cosTheta * cosTheta);
    float phi = 2.0 * PI * random.getFloat();

    float3 dir = float3(
        cos(phi) * sinTheta,
        cosTheta,
        sin(phi) * sinTheta
    );

    return mul(dir, tbn);
}

float getMiePhase(float cosTheta) {
    float g = 0.75;
    return 1.0 / (4.0 * PI) * (1.0 - g * g) / pow(1.0 + g * g - 2.0 * g * cosTheta, 1.5);
}

float getMieDensity(float altitude) {
    return exp(-max(0.0, altitude) / 1400.0);
}

float3 getMieDirection(float3 direction, inout Random random) {
    float3x3 tbn = getTbnMatrix(direction);

    float g = 0.75;

	float t = (1.0 - g * g) / (1.0 - g + 2.0 * g * random.getFloat());
	float cos_theta = (1.0 + g * g - t) / (2.0 * g);
	float sin_theta = sqrt(1.0 - cos_theta * cos_theta);
	float phi = 2.0 * PI * random.getFloat();
	
	float3 dir = float3(
        cos(phi) * sin_theta, 
        cos_theta, 
        sin(phi) * sin_theta
    );
    
	return mul(dir, tbn);
}

// TODO: find reference
float getOzoneDensity(float altitude) {
    float x = (max(0.0, altitude) - 22000.0) / 6092.04;
    return 2.0 / (exp(-x) + exp(x));
}

// TODO: find reference
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

    // https://en.wikipedia.org/wiki/Rayleigh_scattering
    static Coefficients rayleigh(float wavelength) {
        // refractive index of air
        const float n = 1.000279;
        const float n2 = n * n;

        //float scattering = (8.0 * PI / 3.0) * pow(2.0 * PI / wavelength, 4.0) * pow((n2 - 1) / (n2 + 2), 2.0);

        float scattering = 2.504e-5 * 3.7307e10 / pow(wavelength - 13.6111, 4.0);
        float absorption = 0.0;

        // sigma_s
        return Coefficients(scattering, absorption);
    }

    // TODO: find reference
    static Coefficients mie(float wavelength) {
        float turbidity = 1.0;

        float A = 0.6544 * turbidity - 0.6510;
        float B = (166.504891453 - 0.833005669719 * wavelength) / (1.0 - 0.00546759 * wavelength);

        float scattering = A * B / (wavelength * wavelength);
        float absorption = 0.11 * scattering;

        // sigma_s
        return Coefficients(scattering, absorption);
    }

    // TODO: find reference
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

        float r = random.getFloat() * freePathCoefficient;

        if (r < totalExtinction) {
            if (r < sample.rayleigh.scattering) {
                return DeltaTrackCollision.RAYLEIGH_SCATTERING;
            }

            if (r < sample.rayleigh.scattering + sample.mie.scattering) {
                return DeltaTrackCollision.MIE_SCATTERING;
            }

            return DeltaTrackCollision.OZONE_ABSORPTION;
        }

        // if (r < sample.rayleigh.extinction()) {
        //     return DeltaTrackCollision.RAYLEIGH_SCATTERING;
        // } else if (r < sample.mie.extinction()) {
        //     return DeltaTrackCollision.MIE_SCATTERING;
        // } else {
        //     return DeltaTrackCollision.OZONE_ABSORPTION;
        // }
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

float sunTransmittance(Ray sunRay, AtmosphereSample baseSample, float wavelength, inout Random random) {
    float transmittance = 1.0;
    for (uint i = 0; i < 256; i++) {
        Hit earthHit = sunRay.intersect(earthShell);
        Hit atmosphereHit = sunRay.intersect(atmosphereShell);
        Hit closestHit = Hit.merge(earthHit, atmosphereHit);

        if (!closestHit.success) {
            return transmittance;
        }

        if (earthHit.success) {
            return 0.0;
        }

        float _t;
        transmittance *= ratioTrack(sunRay, baseSample, wavelength, random, closestHit.distance, _t);
        sunRay.pos += sunRay.dir * closestHit.distance - closestHit.normal;

        if (transmittance < 1e-6) {
            break;
        }
    }

    return transmittance;
}

public float pathtraceAtmosphere(Ray ray, float wavelength, inout Random random) {
    // skip path tracing if the ray is inside the earth bc wtf
    if (distance(ray.pos, earthShell.position) < earthShell.radius) {
        return 0.0;
    }

    const float eps = 1.0;

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

        // float _t = 0.0;
        // float sunTransmittance = ratioTrack(sunRay, baseSample, wavelength, random, closestHit.distance, _t);
        // Ray sunRay = Ray(ray.pos, normalize(float3(0.1, 0.4, 0.3)));
        // float sunTransmittance = sunTransmittance(sunRay, baseSample, wavelength, random);
        // float sunEmission = sunIrradiance * sunTransmittance;

        if (collisionType == DeltaTrackCollision.BOUNDARY) { // hit planet or atmosphere boundary
            if (earthHit.success) { // hit planet, using lambertian material                
                // bounce off the earth surface
                nextRay.pos = earthHit.position + earthHit.normal * eps;
                nextRay.dir = random.getCosineVector(earthHit.normal);

                Ray sunRay = Ray(nextRay.pos, normalize(float3(0.1, 0.4, 0.3)));
                float sunTransmittance = sunTransmittance(sunRay, baseSample, wavelength, random);
                float sunEmission = sunIrradiance * sunTransmittance;                

                radiance += earthAlbedo * throughput * sunEmission * max(0.0, dot(earthHit.normal, sunRay.dir)) / PI;
                throughput *= earthAlbedo; // * brdf, = 1 / PI implicitly multiplied

            } else { // hit atmosphere edge, atmosphereHit.success
                nextRay.pos = ray.pos + ray.dir * t - closestHit.normal * eps;
                nextRay.dir = ray.dir;
            }
        } else if (
            // any scattering
            collisionType == DeltaTrackCollision.RAYLEIGH_SCATTERING || 
            collisionType == DeltaTrackCollision.MIE_SCATTERING
        ) {
            nextRay.pos = ray.pos + ray.dir * t;

            Ray sunRay = Ray(nextRay.pos, normalize(float3(0.1, 0.4, 0.3)));
            float sunTransmittance = sunTransmittance(sunRay, baseSample, wavelength, random);
            float sunEmission = sunIrradiance * sunTransmittance;                

            float cosTheta = dot(ray.dir, sunRay.dir);

            float phase = 0.0;

            if (collisionType == DeltaTrackCollision.RAYLEIGH_SCATTERING) {
                phase = getRayleighPhase(cosTheta);
                nextRay.dir = getRayleighDirection(ray.dir, random);
            } else if (collisionType == DeltaTrackCollision.MIE_SCATTERING) {
                phase = getMiePhase(cosTheta);
                nextRay.dir = getMieDirection(ray.dir, random);
            }

            radiance += sunEmission * phase * throughput;
        } else {
            break;
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

    return radiance;
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