public struct LookupTable {
    StructuredBuffer<float>.Handle data;
    float min;
    float max;
    float defaultValue;
}