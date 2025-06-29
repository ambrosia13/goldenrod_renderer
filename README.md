# goldenrod rendering engine

`goldenrod_renderer` is a rendering engine and path tracer written in Rust. It runs on Vulkan via the `wgpu` library, which is a native implementation of the WebGPU specification.

goldenrod is made with high, uncompromising physical accuracy in mind, with a great emphasis on spectral physics, participating media, and ease-of-use.

## build from source

To build goldenrod from source, you will need to set the environment variable `SLANGC` to the path of a `slangc` executable on your system, which you can download [here](https://shader-slang.org/) or through the Vulkan SDK.

For ease of use, you can set environmental variables within your local crate through `.cargo/config.toml`:

```toml
[env]
SLANGC = "absolute/path/to/slangc"
```

You will also need python installed and added to your system's PATH. (todo: make this no longer a requirement)

Once you have everything configured correctly, the slang shaders with entrypoints in `assets/shaders/slang` will be automatically compiled to spir-v every time you build the program.