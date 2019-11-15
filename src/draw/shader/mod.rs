use std::borrow::Cow;
use std::ffi::CStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use vulkano::descriptor::descriptor::{DescriptorDesc, ShaderStages};
use vulkano::descriptor::pipeline_layout::{PipelineLayoutDesc, PipelineLayoutDescPcRange};
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::pipeline::shader::{
    GraphicsEntryPoint, GraphicsShaderType, ShaderInterfaceDef, ShaderInterfaceDefEntry,
    ShaderModule,
};

#[derive(Default, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

vulkano::impl_vertex!(Vertex, position, color);

pub fn open(path: &Path, device: Arc<Device>) -> Arc<ShaderModule> {
    let mut f = File::open(path).expect(&format!("Shader not found: {:#?}", path));
    let mut buf = Vec::with_capacity(50);
    f.read_to_end(&mut buf).unwrap();

    unsafe { ShaderModule::new(device, &buf) }.unwrap()
}

pub struct VertInput;

#[derive(Debug, Copy, Clone)]
pub struct VertInputIter(u16);

unsafe impl ShaderInterfaceDef for VertInput {
    type Iter = VertInputIter;

    fn elements(&self) -> VertInputIter {
        VertInputIter(0)
    }
}

impl Iterator for VertInputIter {
    type Item = ShaderInterfaceDefEntry;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // There are things to consider when giving out entries:
        // * There must be only one entry per one location, you can't have
        //   `color' and `position' entries both at 0..1 locations.  They also
        //   should not overlap.
        // * Format of each element must be no larger than 128 bits.
        if self.0 == 0 {
            self.0 += 1;
            return Some(ShaderInterfaceDefEntry {
                location: 1..2,
                format: Format::R32G32B32Sfloat,
                name: Some(Cow::Borrowed("color")),
            });
        }
        if self.0 == 1 {
            self.0 += 1;
            return Some(ShaderInterfaceDefEntry {
                location: 0..1,
                format: Format::R32G32Sfloat,
                name: Some(Cow::Borrowed("position")),
            });
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // We must return exact number of entries left in iterator.
        let len = (2 - self.0) as usize;
        (len, Some(len))
    }
}

impl ExactSizeIterator for VertInputIter {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct VertOutput;

unsafe impl ShaderInterfaceDef for VertOutput {
    type Iter = VertOutputIter;

    fn elements(&self) -> VertOutputIter {
        VertOutputIter(0)
    }
}

// This structure will tell Vulkan how output entries (those passed to next
// stage) of our vertex shader look like.
#[derive(Debug, Copy, Clone)]
pub struct VertOutputIter(u16);

impl Iterator for VertOutputIter {
    type Item = ShaderInterfaceDefEntry;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            self.0 += 1;
            return Some(ShaderInterfaceDefEntry {
                location: 0..1,
                format: Format::R32G32B32Sfloat,
                name: Some(Cow::Borrowed("v_color")),
            });
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (1 - self.0) as usize;
        (len, Some(len))
    }
}

impl ExactSizeIterator for VertOutputIter {}

// This structure describes layout of this stage.
#[derive(Debug, Copy, Clone)]
pub struct VertLayout(ShaderStages);
unsafe impl PipelineLayoutDesc for VertLayout {
    // Number of descriptor sets it takes.
    fn num_sets(&self) -> usize {
        0
    }
    // Number of entries (bindings) in each set.
    fn num_bindings_in_set(&self, _set: usize) -> Option<usize> {
        None
    }
    // Descriptor descriptions.
    fn descriptor(&self, _set: usize, _binding: usize) -> Option<DescriptorDesc> {
        None
    }
    // Number of push constants ranges (think: number of push constants).
    fn num_push_constants_ranges(&self) -> usize {
        0
    }
    // Each push constant range in memory.
    fn push_constants_range(&self, _num: usize) -> Option<PipelineLayoutDescPcRange> {
        None
    }
}

// Same as with our vertex shader, but for fragment one instead.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FragInput;
unsafe impl ShaderInterfaceDef for FragInput {
    type Iter = FragInputIter;

    fn elements(&self) -> FragInputIter {
        FragInputIter(0)
    }
}
#[derive(Debug, Copy, Clone)]
pub struct FragInputIter(u16);

impl Iterator for FragInputIter {
    type Item = ShaderInterfaceDefEntry;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            self.0 += 1;
            return Some(ShaderInterfaceDefEntry {
                location: 0..1,
                format: Format::R32G32B32Sfloat,
                name: Some(Cow::Borrowed("v_color")),
            });
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (1 - self.0) as usize;
        (len, Some(len))
    }
}

impl ExactSizeIterator for FragInputIter {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FragOutput;
unsafe impl ShaderInterfaceDef for FragOutput {
    type Iter = FragOutputIter;

    fn elements(&self) -> FragOutputIter {
        FragOutputIter(0)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FragOutputIter(u16);

impl Iterator for FragOutputIter {
    type Item = ShaderInterfaceDefEntry;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Note that color fragment color entry will be determined
        // automatically by Vulkano.
        if self.0 == 0 {
            self.0 += 1;
            return Some(ShaderInterfaceDefEntry {
                location: 0..1,
                format: Format::R32G32B32A32Sfloat,
                name: Some(Cow::Borrowed("f_color")),
            });
        }
        None
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (1 - self.0) as usize;
        (len, Some(len))
    }
}

impl ExactSizeIterator for FragOutputIter {}

// Layout same as with vertex shader.
#[derive(Debug, Copy, Clone)]
pub struct FragLayout(ShaderStages);
unsafe impl PipelineLayoutDesc for FragLayout {
    fn num_sets(&self) -> usize {
        0
    }
    fn num_bindings_in_set(&self, _set: usize) -> Option<usize> {
        None
    }
    fn descriptor(&self, _set: usize, _binding: usize) -> Option<DescriptorDesc> {
        None
    }
    fn num_push_constants_ranges(&self) -> usize {
        0
    }
    fn push_constants_range(&self, _num: usize) -> Option<PipelineLayoutDescPcRange> {
        None
    }
}

pub fn get_entry_fragment(
    shader: &ShaderModule,
) -> GraphicsEntryPoint<'_, (), FragInput, FragOutput, FragLayout> {
    unsafe {
        shader.graphics_entry_point(
            CStr::from_bytes_with_nul_unchecked(b"main\0"),
            FragInput,
            FragOutput,
            FragLayout(ShaderStages {
                fragment: true,
                ..ShaderStages::none()
            }),
            GraphicsShaderType::Fragment,
        )
    }
}
pub fn get_entry_vertex(
    shader: &ShaderModule,
) -> GraphicsEntryPoint<'_, (), VertInput, VertOutput, VertLayout> {
    unsafe {
        shader.graphics_entry_point(
            CStr::from_bytes_with_nul_unchecked(b"main\0"),
            VertInput,
            VertOutput,
            VertLayout(ShaderStages {
                vertex: true,
                ..ShaderStages::none()
            }),
            GraphicsShaderType::Vertex,
        )
    }
}
