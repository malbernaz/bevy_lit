use bevy::{
    math::{IVec2, UVec2},
    render::render_resource::ShaderType,
};

/// A wrapper for [`IVec2`] that ensures 16-byte alignment for use in uniform buffers.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, ShaderType)]
pub(crate) struct IVec2Uniform {
    inner: IVec2,
    _padding: IVec2,
}
impl IVec2Uniform {
    pub(crate) fn new(x: i32, y: i32) -> Self {
        Self {
            inner: IVec2::new(x, y),
            _padding: IVec2::ZERO,
        }
    }
}

/// A wrapper for [`UVec2`] that ensures 16-byte alignment for use in uniform buffers.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, ShaderType)]
pub(crate) struct UVec2Uniform {
    inner: UVec2,
    _padding: UVec2,
}
impl UVec2Uniform {
    pub(crate) fn new(x: u32, y: u32) -> Self {
        Self {
            inner: UVec2::new(x, y),
            _padding: UVec2::ZERO,
        }
    }
}
