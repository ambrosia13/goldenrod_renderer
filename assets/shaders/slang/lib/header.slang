module header;

public static const float PI = 3.1415926535897932384626433832795;
public static const float HALF_PI = 1.57079632679489661923;
public static const float TAU = 6.2831853071795864769252867665590;

public static const float E = 2.718281828459045235360287471352;

public struct VertexOutput {    
    public float4 clipPosition: SV_Position;

    public float2 uv;
    public float2 texcoord;    
}

public struct Camera {
    public float4x4 viewProjectionMatrix;
    public float4x4 viewMatrix;
    public float4x4 projectionMatrix;

    public float4x4 inverseViewProjectionMatrix;
    public float4x4 inverseViewMatrix;
    public float4x4 inverseProjectionMatrix;

    public float4x4 previousViewProjectionMatrix;
    public float4x4 previousViewMatrix;
    public float4x4 previousProjectionMatrix;

    public float3 position;
    public float3 previousPosition;

    public float3 view;
    public float3 previousView;

    public float3 right;
    public float3 up;

    static float3 fromScreenSpace(float3 screenSpacePos, float4x4 matrix) {
        float3 clipSpacePos = screenSpacePos * 2.0 - 1.0;
        float4 temp = mul(matrix, float4(clipSpacePos, 1.0));
        return temp.xyz / temp.w;
    }

    static float3 toScreenSpace(float3 pos, float4x4 matrix) {
        float4 temp = mul(matrix, float4(pos, 1.0));
        return (temp.xyz / temp.w) * 0.5 + 0.5;
    }

    public float3 screenToScene(float3 screenSpacePos) {
        float3 worldSpacePos = fromScreenSpace(screenSpacePos, this.inverseViewProjectionMatrix);
        return worldSpacePos - this.position;
    }

    public float3 viewDir(float2 texcoord) {
        float3 screenSpacePos = float3(texcoord, 1.0);
        float3 sceneSpacePos = this.screenToScene(screenSpacePos);
        return normalize(sceneSpacePos);
    }
}

public struct View {
    public uint rendererViewportWidth;
    public uint rendererViewportHeight;
    public uint windowWidth;
    public uint windowHeight;

    public float aspectRatio;

    public uint frameCount;
}

public struct Screen {
    public ConstantBuffer<Camera> camera;
    public ConstantBuffer<View> view;

    public bool shouldAccumulate() {
        return 
            all(this.camera.position == this.camera.previousPosition) &&
            all(this.camera.view == this.camera.previousView) && 
            all(this.camera.projectionMatrix[0] == this.camera.previousProjectionMatrix[0]);
    }
}

public float2 getTaaOffset(uint frame) {
    float2 taaOffsets[] = {
        float2( 0.125, -0.375),
        float2(-0.125,  0.375),
        float2( 0.625,  0.125),
        float2( 0.375, -0.625),
        float2(-0.625,  0.625),
        float2(-0.875, -0.125),
        float2( 0.375, -0.875),
        float2( 0.875,  0.875)
    };

    return taaOffsets[frame % 8];
}

public float3 xyzToRgb(float3 xyz) {
    return mul(
        xyz,
        float3x3(
            3.2404542,-0.9692660, 0.0556434,
            -1.5371385, 1.8760108,-0.2040259,
            -0.4985314, 0.0415560, 1.0572252
        )
    );
}

public float pow2(float x) {
    return x * x;
}

public float3 pow2(float3 x) {
    return x * x;
}