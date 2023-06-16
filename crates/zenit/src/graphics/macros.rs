//! Helpful macros for the Zenit renderer.

/// Similarly to [`wgpu::vertex_attr_array`], this macro generates a list of [`wgpu::BindGroupLayoutEntry`].
///
/// Output has type: `[BindGroupLayoutEntry; _]`. Usage is as follows:
/// ```
/// # use zenit::bind_group_layout_array;
/// use wgpu::*;
/// let entries = bind_group_layout![
///     0 => (
///         VERTEX | FRAGMENT,
///         BindingType::Buffer {
///             ty: BufferBindingType::Uniform,
///             has_dynamic_offset: false,
///             min_binding_size: NonZeroU64::new(123),
///         }
///     ),
///     1 => (
///         FRAGMENT,
///         BindingType::Texture {
///             sample_type: TextureSampleType::Float {
///                 filterable: true,
///             },
///             view_dimension: TextureViewDimension::D2,
///             multisampled: false,
///         }
///     ),
/// ];
/// ```
///
/// Note, that at this time, this macro always sets [`BindGroupLayoutEntry`]`::count` to [`None`].
#[macro_export]
macro_rules! bind_group_layout_array {
    (
        $(
            $binding:expr => (
                $visibility:expr,
                $ty:expr $(,)?
            )
        ),* $(,)?
    ) => {
        [
            $(
                ::wgpu::BindGroupLayoutEntry {
                    binding: $binding,
                    visibility: {
                        const VERTEX: ::wgpu::ShaderStages = ::wgpu::ShaderStages::VERTEX;
                        const FRAGMENT: ::wgpu::ShaderStages = ::wgpu::ShaderStages::FRAGMENT;

                        let _ = VERTEX;
                        let _ = FRAGMENT;

                        $visibility
                    },
                    ty: $ty,
                    count: None,
                },
            )*
        ]
    }
}
