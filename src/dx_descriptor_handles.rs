extern crate winapi;

use winapi::{
	shared::basetsd::SIZE_T,
	um::d3d12,
};

#[repr(transparent)]
#[allow(non_camel_case_types)]
pub struct CD3D12_CPU_DESCRIPTOR_HANDLE(pub d3d12::D3D12_CPU_DESCRIPTOR_HANDLE);

impl CD3D12_CPU_DESCRIPTOR_HANDLE
{
	pub fn new() -> Self
	{
		Self {
			0 : d3d12::D3D12_CPU_DESCRIPTOR_HANDLE {
				ptr : 0,
			},
		}
	}

	pub fn from(other : d3d12::D3D12_CPU_DESCRIPTOR_HANDLE) -> Self
	{
		Self {
			0 : other,
		}
	}

	pub fn offset_cpu_descriptor_handle(
		handle : &d3d12::D3D12_CPU_DESCRIPTOR_HANDLE, offset_index : i32, descriptor_increment_size : u32,
	) -> d3d12::D3D12_CPU_DESCRIPTOR_HANDLE
	{
		let ptr_64 = handle.ptr as i64;
		let offset_index_64 = offset_index as i64;
		let descriptor_increment_size_64 = descriptor_increment_size as i64;
		let result = (ptr_64 + offset_index_64 * descriptor_increment_size_64) as SIZE_T;
		return d3d12::D3D12_CPU_DESCRIPTOR_HANDLE {
			ptr : result,
		};
	}

	pub fn from_offset(
		other : &d3d12::D3D12_CPU_DESCRIPTOR_HANDLE, offset_index : i32, descriptor_increment_size : u32,
	) -> Self
	{
		Self {
			0 : Self::offset_cpu_descriptor_handle(&other, offset_index, descriptor_increment_size),
		}
	}

	pub fn offset(&mut self, offset_index : i32, descriptor_increment_size : u32) -> &mut Self
	{
		self.0 = Self::offset_cpu_descriptor_handle(&self.0, offset_index, descriptor_increment_size);
		self
	}
}

#[repr(transparent)]
#[allow(non_camel_case_types)]
pub struct CD3D12_GPU_DESCRIPTOR_HANDLE(pub d3d12::D3D12_GPU_DESCRIPTOR_HANDLE);

impl CD3D12_GPU_DESCRIPTOR_HANDLE
{
	pub fn new() -> Self
	{
		Self {
			0 : d3d12::D3D12_GPU_DESCRIPTOR_HANDLE {
				ptr : 0,
			},
		}
	}

	pub fn from(other : d3d12::D3D12_GPU_DESCRIPTOR_HANDLE) -> Self
	{
		Self {
			0 : other,
		}
	}

	pub fn offset_gpu_descriptor_handle(
		handle : &d3d12::D3D12_GPU_DESCRIPTOR_HANDLE, offset_index : i32, descriptor_increment_size : u32,
	) -> d3d12::D3D12_GPU_DESCRIPTOR_HANDLE
	{
		let ptr_64 = handle.ptr as i64;
		let offset_index_64 = offset_index as i64;
		let descriptor_increment_size_64 = descriptor_increment_size as i64;
		let result = (ptr_64 + offset_index_64 * descriptor_increment_size_64) as u64;
		return d3d12::D3D12_GPU_DESCRIPTOR_HANDLE {
			ptr : result,
		};
	}

	pub fn from_offset(
		other : &d3d12::D3D12_GPU_DESCRIPTOR_HANDLE, offset_index : i32, descriptor_increment_size : u32,
	) -> Self
	{
		Self {
			0 : Self::offset_gpu_descriptor_handle(&other, offset_index, descriptor_increment_size),
		}
	}

	pub fn offset(&mut self, offset_index : i32, descriptor_increment_size : u32) -> &mut Self
	{
		self.0 = Self::offset_gpu_descriptor_handle(&self.0, offset_index, descriptor_increment_size);
		self
	}
}
