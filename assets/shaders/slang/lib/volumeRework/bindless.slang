// Current configuration
// Binding  Type                Usage
// -----------------------------------
// 0        Sampler             for material texture support
// 1        Combined            unused
// 2        SampledTexture      for material texture support
// 3        StorageTexture      unused
// 4        UTexelBuffer        unused
// 5        STexelBuffer        unused
// 6        UniformBuffer       unused
// 7        StorageBuffer       for material/volume lookup tables



export T getDescriptorFromHandle<T>(DescriptorHandle<T> handle) where T: IOpaqueDescriptor {
    return defaultGetDescriptorFromHandle(handle, BindlessDescriptorOptions.None);
}