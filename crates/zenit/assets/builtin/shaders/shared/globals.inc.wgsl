
// Keep this uniform buffer in sync with <TODO>
struct Globals {
    f32 time;
    /// Framebuffer size in pixels.
    /// Note, that this doesn't have to be the same size as the camera that's currently being
    /// rendered to, for that access the camera's uniform buffer.
    vec2<u32> screen_size;
}
