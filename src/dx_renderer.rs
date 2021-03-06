// extern crate d3d12_rs;
extern crate winapi;
use crate::{
	dx_descriptor_handles::{
		CD3D12_CPU_DESCRIPTOR_HANDLE,
		CD3D12_GPU_DESCRIPTOR_HANDLE,
	},
	geometry::*,
	transforms,
	weak_ptr::WeakPtr,
	win_window,
};
use cgmath::*;

pub use winapi::shared::winerror::HRESULT;

use winapi::{
	shared::{
		dxgi,
		dxgi1_2,
		dxgi1_3,
		dxgi1_4,
		dxgiformat,
		dxgitype,
		minwindef::{
			FALSE,
			TRUE,
			UINT,
		},
		ntdef::HANDLE,
		winerror,
	},
	um::{
		d3d12,
		d3d12sdklayers,
		d3dcommon,
		d3dcompiler::*,
		synchapi::{
			CreateEventW,
			WaitForSingleObject,
		},
		winbase::INFINITE,
	},
	Interface,
};

use std::{
	assert,
	convert::TryFrom,
	ffi::{
		CStr,
		CString,
		OsString,
	},
	io::Error,
	mem,
	os::windows::ffi::{
		OsStrExt,
		OsStringExt,
	},
	ptr,
	string::String,
};

const G_MAX_FRAME_COUNT : usize = 3;
const G_SINGLE_NODEMASK : u32 = 0;
const G_WIDTH : u32 = 1280;
const G_HEIGHT : u32 = 720;
const FOVY : f32 = 90.0;
const ASPECT_RATIO : f32 = G_WIDTH as f32 / G_HEIGHT as f32;

struct MatrixConstantBuffer
{
	#[allow(dead_code)]
	mvp_transform : cgmath::Matrix4<f32>,
	_padding :      [u8; 192],
}

#[allow(dead_code)]
pub struct Renderer
{
	viewport : d3d12::D3D12_VIEWPORT,
	scissor_rect : d3d12::D3D12_RECT,
	factory : WeakPtr<dxgi1_4::IDXGIFactory4>,
	adapter : WeakPtr<dxgi1_2::IDXGIAdapter2>,
	device : WeakPtr<d3d12::ID3D12Device>,
	command_queue : WeakPtr<d3d12::ID3D12CommandQueue>,
	swap_chain : WeakPtr<dxgi1_4::IDXGISwapChain3>,
	rtv_descriptor_heap : WeakPtr<d3d12::ID3D12DescriptorHeap>,
	rtv_descriptor_size : u32,
	cbv_descriptor_heap : WeakPtr<d3d12::ID3D12DescriptorHeap>,
	command_allocators : [WeakPtr<d3d12::ID3D12CommandAllocator>; G_MAX_FRAME_COUNT],
	command_list : WeakPtr<d3d12::ID3D12GraphicsCommandList>,
	render_targets : [WeakPtr<d3d12::ID3D12Resource>; G_MAX_FRAME_COUNT],
	root_signature : WeakPtr<d3d12::ID3D12RootSignature>,
	pipeline_state : WeakPtr<d3d12::ID3D12PipelineState>,
	frame_count : u32,
	frame_index : usize,
	vertex_buffer : WeakPtr<d3d12::ID3D12Resource>,
	vertex_buffer_view : d3d12::D3D12_VERTEX_BUFFER_VIEW,
	constant_buffer : WeakPtr<d3d12::ID3D12Resource>,
	constant_buffer_gpu_handle : CD3D12_GPU_DESCRIPTOR_HANDLE,
	p_cbv_data : *mut MatrixConstantBuffer,
	fence : WeakPtr<d3d12::ID3D12Fence>,
	fence_values : [u64; G_MAX_FRAME_COUNT],
	fence_event : HANDLE,
	timer : std::time::Instant,
}

fn to_wchar(str : &str) -> Vec<u16>
{
	std::ffi::OsString::from(str).encode_wide().collect()
}

fn to_cstring(str : &str) -> CString
{
	CString::new(str).unwrap()
}

impl Renderer
{
	pub fn new() -> Self
	{
		if cfg!(debug_assertions)
		{
			let mut debug_controller = WeakPtr::<d3d12sdklayers::ID3D12Debug>::null();
			let hr_debug = unsafe {
				winapi::um::d3d12::D3D12GetDebugInterface(
					&d3d12sdklayers::ID3D12Debug::uuidof(),
					debug_controller.mut_void(),
				)
			};
			assert!(winerror::SUCCEEDED(hr_debug), "Unable to get D3D12 debug interface. {:x}", hr_debug);

			unsafe {
				debug_controller.EnableDebugLayer();
			}

			unsafe {
				debug_controller.Release();
			} // Clean Up
		}

		let frame_count : u32 = 2; // number of backbuffers to support. 2 is basic ping-pong buffers.
		assert!(frame_count as usize <= G_MAX_FRAME_COUNT);

		Self {
			viewport : d3d12::D3D12_VIEWPORT {
				TopLeftX : 0.0,
				TopLeftY : 0.0,
				Width :    G_WIDTH as f32,
				Height :   G_HEIGHT as f32,
				MinDepth : d3d12::D3D12_MIN_DEPTH,
				MaxDepth : d3d12::D3D12_MAX_DEPTH,
			},
			scissor_rect : d3d12::D3D12_RECT {
				left :   0,
				top :    0,
				right :  G_WIDTH as i32,
				bottom : G_HEIGHT as i32,
			},
			factory : WeakPtr::<dxgi1_4::IDXGIFactory4>::null(),
			adapter : WeakPtr::<dxgi1_2::IDXGIAdapter2>::null(),
			device : WeakPtr::<d3d12::ID3D12Device>::null(),
			command_queue : WeakPtr::<d3d12::ID3D12CommandQueue>::null(),
			swap_chain : WeakPtr::<dxgi1_4::IDXGISwapChain3>::null(),
			rtv_descriptor_heap : WeakPtr::<d3d12::ID3D12DescriptorHeap>::null(),
			rtv_descriptor_size : 0,
			cbv_descriptor_heap : WeakPtr::<d3d12::ID3D12DescriptorHeap>::null(),
			command_allocators : [WeakPtr::<d3d12::ID3D12CommandAllocator>::null(); G_MAX_FRAME_COUNT],
			command_list : WeakPtr::<d3d12::ID3D12GraphicsCommandList>::null(),
			render_targets : [WeakPtr::null(); G_MAX_FRAME_COUNT],
			root_signature : WeakPtr::<d3d12::ID3D12RootSignature>::null(),
			pipeline_state : WeakPtr::<d3d12::ID3D12PipelineState>::null(),
			frame_count : frame_count,
			frame_index : 0,
			vertex_buffer : WeakPtr::<d3d12::ID3D12Resource>::null(),
			vertex_buffer_view : unsafe { mem::zeroed() },
			constant_buffer : WeakPtr::<d3d12::ID3D12Resource>::null(),
			constant_buffer_gpu_handle : CD3D12_GPU_DESCRIPTOR_HANDLE::new(),
			p_cbv_data : ptr::null_mut(),
			fence : WeakPtr::<d3d12::ID3D12Fence>::null(),
			fence_values : [0; G_MAX_FRAME_COUNT],
			fence_event : ptr::null_mut(),
			timer : std::time::Instant::now(),
		}
	}

	pub fn load_pipeline(&mut self, window : win_window::Window)
	{
		if cfg!(debug_assertions)
		{
			let mut debug_controller = WeakPtr::<d3d12sdklayers::ID3D12Debug>::null();
			let hr_debug = unsafe {
				winapi::um::d3d12::D3D12GetDebugInterface(
					&d3d12sdklayers::ID3D12Debug::uuidof(),
					debug_controller.mut_void(),
				)
			};
			assert!(winerror::SUCCEEDED(hr_debug), "Unable to get D3D12 debug interface. {:x}", hr_debug);

			unsafe {
				debug_controller.EnableDebugLayer();
			}

			unsafe {
				debug_controller.Release(); // Clean Up
			}
		}

		let factory_flags = match cfg!(debug_assertions)
		{
			true => dxgi1_3::DXGI_CREATE_FACTORY_DEBUG,
			false => 0,
		};

		let mut factory = WeakPtr::<dxgi1_4::IDXGIFactory4>::null();
		let hr_factory = unsafe {
			dxgi1_3::CreateDXGIFactory2(factory_flags, &dxgi1_4::IDXGIFactory4::uuidof(), factory.mut_void())
		};
		assert!(winerror::SUCCEEDED(hr_factory), "Failed on factory creation. {:x}", hr_factory);
		self.factory = factory;

		let mut adapter_index = 0;
		let _adapter = loop
		{
			let mut adapter1 = WeakPtr::<dxgi::IDXGIAdapter1>::null();
			let hr1 = unsafe { self.factory.EnumAdapters1(adapter_index, adapter1.mut_void() as *mut *mut _) };

			if hr1 == winerror::DXGI_ERROR_NOT_FOUND
			{
				break Err("Failed to enumerate adapters: DXGI_ERROR_NOT_FOUND");
			}

			let (adapter2, hr2) = unsafe { adapter1.cast::<dxgi1_2::IDXGIAdapter2>() };

			unsafe {
				adapter1.destroy();
			} // always clean up

			if !winerror::SUCCEEDED(hr2)
			{
				break Err("Failed to casting to adapter2.");
			}

			adapter_index += 1;

			// Check to see if the adapter supports Direct3D 12, but don't create the
			// actual device yet.
			let mut _device = WeakPtr::<d3d12::ID3D12Device>::null();
			let hr_device = unsafe {
				d3d12::D3D12CreateDevice(
					adapter2.as_mut_ptr() as *mut _,
					d3dcommon::D3D_FEATURE_LEVEL_11_0 as _,
					&d3d12::ID3D12Device::uuidof(),
					_device.mut_void(),
				)
			};

			if !winerror::SUCCEEDED(hr_device)
			{
				unsafe {
					adapter2.destroy();
				}; // always clean up before looping back
				continue;
			}

			break Ok(adapter2);
		};

		self.adapter = _adapter.expect("Failed to find a reasonable adapter.");

		// create the device for real
		let mut device = WeakPtr::<d3d12::ID3D12Device>::null();
		let hr_device = unsafe {
			d3d12::D3D12CreateDevice(
				self.adapter.as_unknown() as *const _ as *mut _,
				d3dcommon::D3D_FEATURE_LEVEL_11_0 as _,
				&d3d12::ID3D12Device::uuidof(),
				device.mut_void(),
			)
		};
		assert!(winerror::SUCCEEDED(hr_device), "Failed to create DX12 device. {:x}", hr_device);
		self.device = device;

		// Describe and Create the command queue.
		let desc = d3d12::D3D12_COMMAND_QUEUE_DESC {
			Type :     d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT as _,
			Priority : d3d12::D3D12_COMMAND_QUEUE_PRIORITY_NORMAL as _,
			Flags :    d3d12::D3D12_COMMAND_QUEUE_FLAG_NONE, // TODO
			NodeMask : G_SINGLE_NODEMASK,
		};

		let mut command_queue = WeakPtr::<d3d12::ID3D12CommandQueue>::null();
		let hr_queue = unsafe {
			self.device.CreateCommandQueue(&desc, &d3d12::ID3D12CommandQueue::uuidof(), command_queue.mut_void())
		};
		assert!(winerror::SUCCEEDED(hr_queue), "error on queue creation: {:x}", hr_queue);
		self.command_queue = command_queue;

		// Create the Swap Chain
		let desc = dxgi1_2::DXGI_SWAP_CHAIN_DESC1 {
			AlphaMode :   dxgi1_2::DXGI_ALPHA_MODE_IGNORE,
			BufferCount : self.frame_count,
			Width :       G_WIDTH,
			Height :      G_HEIGHT,
			Format :      dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
			Flags :       dxgi::DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT,
			BufferUsage : dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
			SampleDesc :  dxgitype::DXGI_SAMPLE_DESC {
				Count :   1,
				Quality : 0,
			},
			Scaling :     dxgi1_2::DXGI_SCALING_STRETCH,
			Stereo :      FALSE,
			SwapEffect :  dxgi::DXGI_SWAP_EFFECT_FLIP_DISCARD,
		};

		self.swap_chain = unsafe {
			let mut swap_chain1 = WeakPtr::<dxgi1_2::IDXGISwapChain1>::null();

			let hr = self.factory.CreateSwapChainForHwnd(
				command_queue.as_mut_ptr() as *mut _,
				window.handle,
				&desc,
				ptr::null(),
				ptr::null_mut(),
				swap_chain1.mut_void() as *mut *mut _,
			);
			assert!(winerror::SUCCEEDED(hr), "error on swapchain creation 0x{:x}", hr);

			let (swap_chain3, hr3) = swap_chain1.cast::<dxgi1_4::IDXGISwapChain3>();
			assert!(winerror::SUCCEEDED(hr), "error on swapchain3 cast 0x{:x}", hr3);

			swap_chain1.destroy();
			swap_chain3
		};

		self.frame_index = unsafe { self.swap_chain.GetCurrentBackBufferIndex() } as usize;

		let heap_type = d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV;

		// Create Descriptor Heaps
		let mut rtv_descriptor_heap = WeakPtr::<d3d12::ID3D12DescriptorHeap>::null();
		let descriptor_heap_desc = d3d12::D3D12_DESCRIPTOR_HEAP_DESC {
			Type :           heap_type as _,
			NumDescriptors : self.frame_count,
			Flags :          d3d12::D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
			NodeMask :       G_SINGLE_NODEMASK,
		};
		let descriptor_heap_hr = unsafe {
			self.device.CreateDescriptorHeap(
				&descriptor_heap_desc,
				&d3d12::ID3D12DescriptorHeap::uuidof(),
				rtv_descriptor_heap.mut_void(),
			)
		};
		assert!(
			winerror::SUCCEEDED(descriptor_heap_hr),
			"error on descriptor_heap creation 0x{:x}",
			descriptor_heap_hr
		);
		self.rtv_descriptor_heap = rtv_descriptor_heap;
		unsafe {
			let buffer_name : String = String::from("rtv descriptor heap");
			let buffer_size = u32::try_from(buffer_name.len()).unwrap();
			self.rtv_descriptor_heap.SetPrivateData(
				&d3dcommon::WKPDID_D3DDebugObjectName,
				buffer_size,
				buffer_name.as_ptr() as *mut _,
			);
		}

		// Create Render Target Views on the RTV Heap
		let rtv_descriptor_size = unsafe { self.device.GetDescriptorHandleIncrementSize(heap_type as _) };
		self.rtv_descriptor_size = rtv_descriptor_size;
		let rtv_heap_cpu_handle = unsafe { rtv_descriptor_heap.GetCPUDescriptorHandleForHeapStart() };
		let _rtv_heap_gpu_handle = unsafe { rtv_descriptor_heap.GetGPUDescriptorHandleForHeapStart() };

		let write_render_targets = &mut self.render_targets[0..(self.frame_count as usize)];

		let mut rtv_cpu_handle =
			CD3D12_CPU_DESCRIPTOR_HANDLE::from_offset(&rtv_heap_cpu_handle, 0, self.rtv_descriptor_size);
		for n in 0..write_render_targets.len()
		{
			unsafe {
				let render_target_ref = &mut write_render_targets[n];
				self.swap_chain.GetBuffer(n as _, &d3d12::ID3D12Resource::uuidof(), render_target_ref.mut_void());
				self.device.CreateRenderTargetView(render_target_ref.as_mut_ptr(), ptr::null(), rtv_cpu_handle.0);
				rtv_cpu_handle.offset(1, rtv_descriptor_size);
			}
		}

		// Create Constant Buffer Descriptor Heap
		let mut cbv_descriptor_heap = WeakPtr::<d3d12::ID3D12DescriptorHeap>::null();
		let cbv_descriptor_heap_desc = d3d12::D3D12_DESCRIPTOR_HEAP_DESC {
			Type :           d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
			NumDescriptors : 1,
			Flags :          d3d12::D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
			NodeMask :       G_SINGLE_NODEMASK,
		};
		let cbv_descriptor_heap_hr = unsafe {
			self.device.CreateDescriptorHeap(
				&cbv_descriptor_heap_desc,
				&d3d12::ID3D12DescriptorHeap::uuidof(),
				cbv_descriptor_heap.mut_void(),
			)
		};
		assert!(
			winerror::SUCCEEDED(cbv_descriptor_heap_hr),
			"error on constant buffer descriptor heap creation 0x{:x}",
			cbv_descriptor_heap_hr
		);
		self.cbv_descriptor_heap = cbv_descriptor_heap;
		unsafe {
			let buffer_name : String = String::from("cbv descriptor heap");
			let buffer_size = u32::try_from(buffer_name.len()).unwrap();
			self.cbv_descriptor_heap.SetPrivateData(
				&d3dcommon::WKPDID_D3DDebugObjectName,
				buffer_size,
				buffer_name.as_ptr() as *mut _,
			);
		}

		// Create Command Allocators
		for n in 0..self.frame_count as usize
		{
			let mut command_allocator = WeakPtr::<d3d12::ID3D12CommandAllocator>::null();
			let hr_command_allocator = unsafe {
				self.device.CreateCommandAllocator(
					d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT as _,
					&d3d12::ID3D12CommandAllocator::uuidof(),
					command_allocator.mut_void(),
				)
			};
			assert!(
				winerror::SUCCEEDED(hr_command_allocator),
				"Failed to create command allocator. 0x{:x}",
				hr_command_allocator
			);
			self.command_allocators[n] = command_allocator;
		}
	}

	pub fn _get_adapter_name(adapter : WeakPtr<dxgi1_2::IDXGIAdapter2>) -> String
	{
		let mut desc : dxgi1_2::DXGI_ADAPTER_DESC2 = unsafe { mem::zeroed() };
		unsafe {
			adapter.GetDesc2(&mut desc);
		}

		let device_name = {
			let len = desc
				.Description
				.iter()
				.take_while(|&&c| c != 0) // closure: func(&&c) { return c != 0; }
				.count();
			let name = <OsString as OsStringExt>::from_wide(&desc.Description[..len]);
			name.to_string_lossy().into_owned()
		};

		// Handy to know these are available.
		// let _name = _device_name;
		// let _vendor = desc.VendorId as usize;
		// let _device = desc.DeviceId as usize;

		return device_name;
	}

	pub fn _get_additional_device_data(device : WeakPtr<d3d12::ID3D12Device>)
	{
		let mut features_architecture : d3d12::D3D12_FEATURE_DATA_ARCHITECTURE = unsafe { mem::zeroed() };
		let hr_check_feature_support_architecture = unsafe {
			device.CheckFeatureSupport(
				d3d12::D3D12_FEATURE_ARCHITECTURE,
				&mut features_architecture as *mut _ as *mut _, /* take reference, cast to pointer, cast to void
				                                                 * pointer */
				mem::size_of::<d3d12::D3D12_FEATURE_DATA_ARCHITECTURE>() as _,
			)
		};
		assert!(
			winerror::SUCCEEDED(hr_check_feature_support_architecture),
			"Failed to check feature support. 0x{:x}",
			hr_check_feature_support_architecture
		);

		let mut features : d3d12::D3D12_FEATURE_DATA_D3D12_OPTIONS = unsafe { mem::zeroed() };
		let hr_check_feature_support_d3d12_options = unsafe {
			device.CheckFeatureSupport(
				d3d12::D3D12_FEATURE_D3D12_OPTIONS,
				&mut features as *mut _ as *mut _,
				mem::size_of::<d3d12::D3D12_FEATURE_DATA_D3D12_OPTIONS>() as _,
			)
		};
		assert!(
			winerror::SUCCEEDED(hr_check_feature_support_d3d12_options),
			"Failed to check feature support. 0x{:x}",
			hr_check_feature_support_d3d12_options
		);
	}

	pub fn load_assets(&mut self)
	{
		// Create an empty Root Signature
		let mut signature_raw = WeakPtr::<d3dcommon::ID3DBlob>::null();
		let mut signature_error = WeakPtr::<d3dcommon::ID3DBlob>::null();

		// TODO Create a root signature consisting of a descriptor table with a single
		// CBV.
		let ranges = [d3d12::D3D12_DESCRIPTOR_RANGE1 {
			RangeType : d3d12::D3D12_DESCRIPTOR_RANGE_TYPE_CBV,
			NumDescriptors : 1,
			BaseShaderRegister : 0,
			RegisterSpace : 0,
			Flags : d3d12::D3D12_DESCRIPTOR_RANGE_FLAG_DATA_STATIC,
			OffsetInDescriptorsFromTableStart : d3d12::D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
		}];

		let root_descriptor_table = d3d12::D3D12_ROOT_DESCRIPTOR_TABLE1 {
			NumDescriptorRanges : ranges.len() as u32,
			pDescriptorRanges :   ranges.as_ptr(),
		};

		let mut root_parameter_desc_table : d3d12::D3D12_ROOT_PARAMETER1 = unsafe { std::mem::zeroed() };
		root_parameter_desc_table.ParameterType = d3d12::D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE;
		root_parameter_desc_table.ShaderVisibility = d3d12::D3D12_SHADER_VISIBILITY_ALL;
		unsafe {
			*root_parameter_desc_table.u.DescriptorTable_mut() = root_descriptor_table;
		}

		let root_parameters = [root_parameter_desc_table];
		let static_samplers : &[d3d12::D3D12_STATIC_SAMPLER_DESC] = &[];
		let root_signature_flags = d3d12::D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT;

		let root_signature_desc_1_1 = d3d12::D3D12_ROOT_SIGNATURE_DESC1 {
			NumParameters : root_parameters.len() as _,
			pParameters : root_parameters.as_ptr() as *const _,
			NumStaticSamplers : static_samplers.len() as _,
			pStaticSamplers : static_samplers.as_ptr() as _,
			Flags : root_signature_flags,
		};

		let mut root_signature_desc : d3d12::D3D12_VERSIONED_ROOT_SIGNATURE_DESC = unsafe { std::mem::zeroed() };
		root_signature_desc.Version = d3d12::D3D_ROOT_SIGNATURE_VERSION_1_1;
		unsafe {
			*root_signature_desc.u.Desc_1_1_mut() = root_signature_desc_1_1;
		}

		let hr_seralize_root_signature = unsafe {
			d3d12::D3D12SerializeVersionedRootSignature(
				// TODO try making this VersionedRootSIgnature instead of non-versioned?
				&root_signature_desc,
				signature_raw.mut_void() as *mut *mut _,
				signature_error.mut_void() as *mut *mut _,
			)
		};

		if !signature_error.is_null()
		{
			println!(
				"Root signature serialization error: {:?}",
				unsafe {
					let data = signature_error.GetBufferPointer();
					CStr::from_ptr(data as *const _ as *const _)
				}
				.to_str()
				.unwrap()
			);
			unsafe {
				signature_error.destroy();
			}
		}

		assert!(
			winerror::SUCCEEDED(hr_seralize_root_signature),
			"Failed to serialize root signature. 0x{:x}",
			hr_seralize_root_signature
		);

		// Create the pipline state, which includes compiling and loading shaders.
		let mut root_signature = WeakPtr::<d3d12::ID3D12RootSignature>::null();
		let root_signature_hr = unsafe {
			self.device.CreateRootSignature(
				G_SINGLE_NODEMASK,
				signature_raw.GetBufferPointer(),
				signature_raw.GetBufferSize(),
				&d3d12::ID3D12RootSignature::uuidof(),
				root_signature.mut_void(),
			)
		};
		assert!(winerror::SUCCEEDED(root_signature_hr), "Failed to create root signature. 0x{:x}", root_signature_hr);
		unsafe {
			signature_raw.destroy();
		}

		unsafe {
			let buffer_name : String = String::from("root signature");
			let buffer_size = u32::try_from(buffer_name.len()).unwrap();
			root_signature.SetPrivateData(
				&d3dcommon::WKPDID_D3DDebugObjectName,
				buffer_size,
				buffer_name.as_ptr() as *mut _,
			);
		}

		self.root_signature = root_signature;

		let compile_flags = if cfg!(debug_assertions)
		{
			D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION
		}
		else
		{
			0
		};

		let shader_path = to_wchar("D:\\Repo\\rust\\rust_raytracer\\src\\shaders.hlsl");

		let vertex_shader_entry_point = to_cstring("VSMain");
		let vertex_shader_compiler_target = to_cstring("vs_5_0");

		let pixel_shader_entry_point = to_cstring("PSMain");
		let pixel_shader_compiler_target = to_cstring("ps_5_0");

		let mut vertex_shader_blob = WeakPtr::<d3dcommon::ID3DBlob>::null();
		let mut vertex_shader_error = WeakPtr::<d3dcommon::ID3DBlob>::null();
		let mut pixel_shader_blob = WeakPtr::<d3dcommon::ID3DBlob>::null();
		let mut pixel_shader_error = WeakPtr::<d3dcommon::ID3DBlob>::null();

		unsafe {
			let hr_vertex_shader_compile = D3DCompileFromFile(
				shader_path.as_ptr(),
				ptr::null() as _,
				ptr::null_mut() as _,
				vertex_shader_entry_point.as_ptr(),
				vertex_shader_compiler_target.as_ptr(),
				compile_flags,
				0,
				vertex_shader_blob.mut_void() as *mut *mut d3dcommon::ID3DBlob,
				vertex_shader_error.mut_void() as *mut *mut d3dcommon::ID3DBlob,
			);

			if !winerror::SUCCEEDED(hr_vertex_shader_compile)
			{
				let error_result = CString::from(CStr::from_ptr(vertex_shader_error.GetBufferPointer() as *const i8));
				let error_result_str = error_result.to_str().unwrap();

				assert!(
					winerror::SUCCEEDED(hr_vertex_shader_compile),
					"Failed to compile vertex shader. HRESULT: 0x{0:x} ; path: {1} ; Error {2} ; Shader Blob Error {3}",
					hr_vertex_shader_compile,
					String::from_utf16(&shader_path).unwrap(),
					std::io::Error::from_raw_os_error(hr_vertex_shader_compile),
					error_result_str
				);
			}

			assert!(
				!vertex_shader_blob.is_null(),
				"Failed to create vertex shader. path: {0}",
				String::from_utf16(&shader_path).unwrap()
			);

			let hr_pixel_shader_compile = D3DCompileFromFile(
				shader_path.as_ptr(),
				ptr::null() as _,
				ptr::null_mut() as _,
				pixel_shader_entry_point.as_ptr(),
				pixel_shader_compiler_target.as_ptr(),
				compile_flags,
				0,
				pixel_shader_blob.mut_void() as *mut *mut d3dcommon::ID3DBlob,
				pixel_shader_error.mut_void() as *mut *mut d3dcommon::ID3DBlob,
			);

			if !winerror::SUCCEEDED(hr_pixel_shader_compile)
			{
				let error_result = CString::from(CStr::from_ptr(pixel_shader_error.GetBufferPointer() as *const i8));
				let error_result_str = error_result.to_str().unwrap();

				assert!(
					winerror::SUCCEEDED(hr_pixel_shader_compile),
					"Failed to compile pixel shader. HRESULT: 0x{0:x} ; path: {1} ; Error {2} ; Shader Blob Error {3}",
					hr_pixel_shader_compile,
					String::from_utf16(&shader_path).unwrap(),
					std::io::Error::from_raw_os_error(hr_pixel_shader_compile),
					error_result_str
				);
			}

			assert!(
				!pixel_shader_blob.is_null(),
				"Failed to create pixel shader. path: {0}",
				String::from_utf16(&shader_path).unwrap()
			);
		}

		let position_semantic = to_cstring("POSITION");
		let color_semnatic = to_cstring("COLOR");

		let input_element_descs : [d3d12::D3D12_INPUT_ELEMENT_DESC; 2] = [
			d3d12::D3D12_INPUT_ELEMENT_DESC {
				SemanticName : position_semantic.as_ptr(),
				SemanticIndex : 0,
				Format : dxgiformat::DXGI_FORMAT_R32G32B32_FLOAT,
				InputSlot : 0,
				AlignedByteOffset : 0,
				InputSlotClass : d3d12::D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
				InstanceDataStepRate : 0,
			},
			d3d12::D3D12_INPUT_ELEMENT_DESC {
				SemanticName : color_semnatic.as_ptr(),
				SemanticIndex : 0,
				Format : dxgiformat::DXGI_FORMAT_R32G32B32_FLOAT,
				InputSlot : 0,
				AlignedByteOffset : 12,
				InputSlotClass : d3d12::D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
				InstanceDataStepRate : 0,
			},
		];

		let vertex_shader = d3d12::D3D12_SHADER_BYTECODE {
			BytecodeLength :  unsafe { vertex_shader_blob.GetBufferSize() },
			pShaderBytecode : unsafe { vertex_shader_blob.GetBufferPointer() },
		};

		let pixel_shader = d3d12::D3D12_SHADER_BYTECODE {
			BytecodeLength :  unsafe { pixel_shader_blob.GetBufferSize() },
			pShaderBytecode : unsafe { pixel_shader_blob.GetBufferPointer() },
		};

		let default_render_target_blend_desc = d3d12::D3D12_RENDER_TARGET_BLEND_DESC {
			BlendEnable : FALSE,
			LogicOpEnable : FALSE,
			SrcBlend : d3d12::D3D12_BLEND_ONE,
			DestBlend : d3d12::D3D12_BLEND_ZERO,
			BlendOp : d3d12::D3D12_BLEND_OP_ADD,
			SrcBlendAlpha : d3d12::D3D12_BLEND_ONE,
			DestBlendAlpha : d3d12::D3D12_BLEND_ZERO,
			BlendOpAlpha : d3d12::D3D12_BLEND_OP_ADD,
			LogicOp : d3d12::D3D12_LOGIC_OP_NOOP,
			RenderTargetWriteMask : d3d12::D3D12_COLOR_WRITE_ENABLE_ALL as u8,
		};

		let default_blendstate = d3d12::D3D12_BLEND_DESC {
			AlphaToCoverageEnable :  FALSE,
			IndependentBlendEnable : TRUE,
			RenderTarget :           [default_render_target_blend_desc; 8],
		};

		let default_rasterizer_state = d3d12::D3D12_RASTERIZER_DESC {
			FillMode : d3d12::D3D12_FILL_MODE_SOLID,
			CullMode : d3d12::D3D12_CULL_MODE_BACK,
			FrontCounterClockwise : FALSE,
			DepthBias : d3d12::D3D12_DEFAULT_DEPTH_BIAS as i32,
			DepthBiasClamp : d3d12::D3D12_DEFAULT_DEPTH_BIAS_CLAMP,
			SlopeScaledDepthBias : d3d12::D3D12_DEFAULT_SLOPE_SCALED_DEPTH_BIAS,
			DepthClipEnable : TRUE,
			MultisampleEnable : FALSE,
			AntialiasedLineEnable : FALSE,
			ForcedSampleCount : 0,
			ConservativeRaster : d3d12::D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
		};

		let default_depth_stencil_op_desc = d3d12::D3D12_DEPTH_STENCILOP_DESC {
			StencilFailOp :      0,
			StencilDepthFailOp : 0,
			StencilPassOp :      0,
			StencilFunc :        0,
		};

		let depth_stencil_state_desc = d3d12::D3D12_DEPTH_STENCIL_DESC {
			DepthEnable :      FALSE,
			DepthWriteMask :   d3d12::D3D12_DEPTH_WRITE_MASK_ZERO,
			DepthFunc :        d3d12::D3D12_COMPARISON_FUNC_NEVER,
			StencilEnable :    FALSE,
			StencilReadMask :  0,
			StencilWriteMask : 0,
			FrontFace :        default_depth_stencil_op_desc,
			BackFace :         default_depth_stencil_op_desc,
		};

		let mut default_rtv_formats = [dxgiformat::DXGI_FORMAT_UNKNOWN; 8];
		default_rtv_formats[0] = dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM;

		// Setup pipeline description
		let pso_desc = d3d12::D3D12_GRAPHICS_PIPELINE_STATE_DESC {
			pRootSignature : self.root_signature.as_mut_ptr(),
			VS : vertex_shader,
			PS : pixel_shader,
			GS : d3d12::D3D12_SHADER_BYTECODE {
				BytecodeLength :  0,
				pShaderBytecode : ptr::null(),
			},
			DS : d3d12::D3D12_SHADER_BYTECODE {
				BytecodeLength :  0,
				pShaderBytecode : ptr::null(),
			},
			HS : d3d12::D3D12_SHADER_BYTECODE {
				BytecodeLength :  0,
				pShaderBytecode : ptr::null(),
			},
			StreamOutput : d3d12::D3D12_STREAM_OUTPUT_DESC {
				pSODeclaration :   ptr::null(),
				NumEntries :       0,
				pBufferStrides :   ptr::null(),
				NumStrides :       0,
				RasterizedStream : 0,
			},
			BlendState : default_blendstate,
			SampleMask : UINT::max_value(),
			RasterizerState : default_rasterizer_state,
			DepthStencilState : depth_stencil_state_desc,
			InputLayout : d3d12::D3D12_INPUT_LAYOUT_DESC {
				pInputElementDescs : input_element_descs.as_ptr(),
				NumElements :        input_element_descs.len() as u32,
			},
			IBStripCutValue : d3d12::D3D12_INDEX_BUFFER_STRIP_CUT_VALUE_DISABLED,
			PrimitiveTopologyType : d3d12::D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
			NumRenderTargets : self.frame_count,
			RTVFormats : default_rtv_formats,
			DSVFormat : dxgiformat::DXGI_FORMAT_UNKNOWN,
			SampleDesc : dxgitype::DXGI_SAMPLE_DESC {
				Count :   1,
				Quality : 0,
			},
			NodeMask : 0,
			CachedPSO : d3d12::D3D12_CACHED_PIPELINE_STATE {
				pCachedBlob :           ptr::null(),
				CachedBlobSizeInBytes : 0,
			},
			Flags : d3d12::D3D12_PIPELINE_STATE_FLAG_NONE,
		};

		// Create Pipeline State
		let mut pipeline = WeakPtr::<d3d12::ID3D12PipelineState>::null();
		unsafe {
			let hr_gpstate = self.device.CreateGraphicsPipelineState(
				&pso_desc,
				&d3d12::ID3D12PipelineState::uuidof(),
				pipeline.mut_void(),
			);

			assert!(winerror::SUCCEEDED(hr_gpstate), "Failed to create graphics pipeline state. 0x{:x}", hr_gpstate);

			let buffer_name : String = String::from("graphics pipeline state");
			let buffer_size = u32::try_from(buffer_name.len()).unwrap();
			pipeline.SetPrivateData(&d3dcommon::WKPDID_D3DDebugObjectName, buffer_size, buffer_name.as_ptr() as *mut _);
		}
		self.pipeline_state = pipeline;

		// Create the Command List
		let mut command_list = WeakPtr::<d3d12::ID3D12GraphicsCommandList>::null();
		unsafe {
			let hr_create_command_list = self.device.CreateCommandList(
				G_SINGLE_NODEMASK,
				d3d12::D3D12_COMMAND_LIST_TYPE_DIRECT,
				self.command_allocators[self.frame_index].as_mut_ptr(),
				pipeline.as_mut_ptr(),
				&d3d12::ID3D12GraphicsCommandList::uuidof(),
				command_list.mut_void(),
			);

			assert!(
				winerror::SUCCEEDED(hr_create_command_list),
				"Failed to create command list. 0x{:x}",
				hr_create_command_list
			);

			// Command lists are created in the recording state, but there is nothing
			// to record yet. The main loop expects it to be closed, so close it now.
			assert!(winerror::SUCCEEDED(command_list.Close()));
		}
		self.command_list = command_list;

		// Create Triangle Assets
		// Upload to Vertex Buffer.
		{
			let mut triangle_vertices = sample_colored_tetrahedron_vertices();
			let triangle_vertices_size = std::mem::size_of_val(&triangle_vertices);
			let triangle_vertices_size_u32 =
				u32::try_from(triangle_vertices_size).expect("Failed Type Conversion: usize -> u32");
			let vertex_size = std::mem::size_of_val(&triangle_vertices[0]);
			let vertex_size_u32 = u32::try_from(vertex_size).expect("Failed Type Conversion: usize -> u32");

			let default_heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
				Type : d3d12::D3D12_HEAP_TYPE_UPLOAD,
				CPUPageProperty : d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
				MemoryPoolPreference : d3d12::D3D12_MEMORY_POOL_UNKNOWN,
				CreationNodeMask : G_SINGLE_NODEMASK,
				VisibleNodeMask : G_SINGLE_NODEMASK,
			};

			let vertex_buffer_resource_desc = d3d12::D3D12_RESOURCE_DESC {
				Dimension : d3d12::D3D12_RESOURCE_DIMENSION_BUFFER,
				Alignment : 0,
				Width : triangle_vertices_size as u64,
				Height : 1,
				DepthOrArraySize : 1,
				MipLevels : 1,
				Format : dxgiformat::DXGI_FORMAT_UNKNOWN,
				SampleDesc : dxgitype::DXGI_SAMPLE_DESC {
					Count :   1,
					Quality : 0,
				},
				Layout : d3d12::D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
				Flags : d3d12::D3D12_RESOURCE_FLAG_NONE,
			};

			let mut vertex_buffer = WeakPtr::<d3d12::ID3D12Resource>::null();

			unsafe {
				let hr_create_committed_resource = self.device.CreateCommittedResource(
					&default_heap_properties,
					d3d12::D3D12_HEAP_FLAG_NONE,
					&vertex_buffer_resource_desc,
					d3d12::D3D12_RESOURCE_STATE_GENERIC_READ,
					ptr::null() as _,
					&d3d12::ID3D12Resource::uuidof(),
					vertex_buffer.mut_void(),
				);

				assert!(
					winerror::SUCCEEDED(hr_create_committed_resource),
					"Failed to create vertex buffer. 0x{:x}",
					hr_create_committed_resource
				);

				let buffer_name : String = String::from("triangle vertex buffer");
				let buffer_size = u32::try_from(buffer_name.len()).unwrap();
				vertex_buffer.SetPrivateData(
					&d3dcommon::WKPDID_D3DDebugObjectName,
					buffer_size,
					buffer_name.as_ptr() as *mut _,
				);
			}

			let mut p_vertex_data_begin = ptr::null_mut::<winapi::ctypes::c_void>();

			let read_range = d3d12::D3D12_RANGE {
				Begin : 0,
				End :   0,
			};
			unsafe {
				let hr_map = vertex_buffer.Map(0, &read_range, &mut p_vertex_data_begin);
				assert!(winerror::SUCCEEDED(hr_map), "Failed to map vertex buffer. 0x{:x}", hr_map);
				assert!(!p_vertex_data_begin.is_null(), "Failed to map vertex buffer. 0x{:x}", hr_map);

				std::ptr::copy_nonoverlapping(
					triangle_vertices.as_mut_ptr(),
					p_vertex_data_begin as *mut ColoredVertex,
					triangle_vertices.len(),
				);

				vertex_buffer.Unmap(0, ptr::null());
			}

			self.vertex_buffer = vertex_buffer;
			self.vertex_buffer_view = d3d12::D3D12_VERTEX_BUFFER_VIEW {
				BufferLocation : unsafe { self.vertex_buffer.GetGPUVirtualAddress() },
				SizeInBytes :    triangle_vertices_size_u32,
				StrideInBytes :  vertex_size_u32,
			};
		}

		// Create constant buffer.
		{
			let constant_buffer_size = std::mem::size_of::<MatrixConstantBuffer>(); // CB size is required to be 256-byte aligned.

			let default_heap_properties = d3d12::D3D12_HEAP_PROPERTIES {
				Type : d3d12::D3D12_HEAP_TYPE_UPLOAD,
				CPUPageProperty : d3d12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
				MemoryPoolPreference : d3d12::D3D12_MEMORY_POOL_UNKNOWN,
				CreationNodeMask : G_SINGLE_NODEMASK,
				VisibleNodeMask : G_SINGLE_NODEMASK,
			};

			let constant_buffer_resource_desc = d3d12::D3D12_RESOURCE_DESC {
				Dimension : d3d12::D3D12_RESOURCE_DIMENSION_BUFFER,
				Alignment : 0,
				Width : constant_buffer_size as u64,
				Height : 1,
				DepthOrArraySize : 1,
				MipLevels : 1,
				Format : dxgiformat::DXGI_FORMAT_UNKNOWN,
				SampleDesc : dxgitype::DXGI_SAMPLE_DESC {
					Count :   1,
					Quality : 0,
				},
				Layout : d3d12::D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
				Flags : d3d12::D3D12_RESOURCE_FLAG_NONE,
			};

			let mut constant_buffer = WeakPtr::<d3d12::ID3D12Resource>::null();

			unsafe {
				let hr_create_constant_buffer = self.device.CreateCommittedResource(
					&default_heap_properties,
					d3d12::D3D12_HEAP_FLAG_NONE,
					&constant_buffer_resource_desc,
					d3d12::D3D12_RESOURCE_STATE_GENERIC_READ,
					ptr::null(),
					&d3d12::ID3D12Resource::uuidof(),
					constant_buffer.mut_void(),
				);

				assert!(
					winerror::SUCCEEDED(hr_create_constant_buffer),
					"Failed to create constant buffer. 0x{:x}",
					hr_create_constant_buffer
				);

				let buffer_name : String = String::from("constant buffer b0");
				let buffer_size = u32::try_from(buffer_name.len()).unwrap();
				constant_buffer.SetPrivateData(
					&d3dcommon::WKPDID_D3DDebugObjectName,
					buffer_size,
					buffer_name.as_ptr() as *mut _,
				);
			}

			// Describe and create a constant buffer view.
			unsafe {
				let cbv_desc = d3d12::D3D12_CONSTANT_BUFFER_VIEW_DESC {
					BufferLocation : constant_buffer.GetGPUVirtualAddress(),
					SizeInBytes :    u32::try_from(constant_buffer_size).unwrap(),
				};

				self.device
					.CreateConstantBufferView(&cbv_desc, self.cbv_descriptor_heap.GetCPUDescriptorHandleForHeapStart());

				let cbv_srv_gpu_handle =
					CD3D12_GPU_DESCRIPTOR_HANDLE::from(self.cbv_descriptor_heap.GetGPUDescriptorHandleForHeapStart());
				self.constant_buffer_gpu_handle = cbv_srv_gpu_handle;
			}

			// Map the constant buffers and cache their heap pointers.
			// We don't unmap this until the app closes. Keeping things mapped for the
			// lifetime of the resource is okay.
			unsafe {
				let mut p_constant_buffer_data_begin = ptr::null_mut::<winapi::ctypes::c_void>();
				let read_range = d3d12::D3D12_RANGE {
					Begin : 0,
					End :   0,
				};
				let hr_map = constant_buffer.Map(0, &read_range, &mut p_constant_buffer_data_begin);
				assert!(winerror::SUCCEEDED(hr_map), "Failed to map constant buffer. 0x{:x}", hr_map);
				assert!(!p_constant_buffer_data_begin.is_null(), "Failed to map constant buffer. 0x{:x}", hr_map);

				self.p_cbv_data = p_constant_buffer_data_begin as *mut MatrixConstantBuffer;

				// Initialize the constant buffer.
				let initial_matrix = <Matrix4<f32> as cgmath::Transform<cgmath::Point3<f32>>>::one();
				std::ptr::copy_nonoverlapping(&initial_matrix, self.p_cbv_data as *mut Matrix4<f32>, 1);
			}

			self.constant_buffer = constant_buffer;
		}

		// Create synchronization objects and wait until assets have been uploaded to
		// the GPU.
		unsafe {
			let hr_create_fence = self.device.CreateFence(
				self.fence_values[self.frame_index],
				d3d12::D3D12_FENCE_FLAG_NONE,
				&d3d12::ID3D12Fence::uuidof(),
				self.fence.mut_void(),
			);
			assert!(winerror::SUCCEEDED(hr_create_fence), "Failed to create fence. 0x{:x}", hr_create_fence);

			self.fence_values[self.frame_index] += 1;

			// Create an event handle to use for frame synchronization.
			self.fence_event = CreateEventW(ptr::null_mut(), FALSE, FALSE, ptr::null());
			assert!(self.fence_event != ptr::null_mut(), "Failed to create fence. 0x{:?}", Error::last_os_error());

			// Wait for the command list to execute
			self.wait_for_gpu()
		}
	}

	pub fn update(&mut self)
	{
		let time_elapsed = self.timer.elapsed().as_secs_f32();
		let model = Matrix4::from_angle_y(Rad::from(Deg(time_elapsed * 90.0)));

		let eye = Point3::new(0.0, 0.66, -2.5);
		let target = Point3::new(0.0, 0.66, 0.0);
		let up = Vector3::unit_y();
		let view_lh = transforms::look_at_lh(eye, target, up);

		let perspective = PerspectiveFov {
			fovy :   cgmath::Rad(FOVY.to_radians()),
			aspect : ASPECT_RATIO,
			near :   0.1,
			far :    100.0,
		};

		let proj_lh = transforms::perspective_lh(perspective);

		let buffer_data = MatrixConstantBuffer {
			mvp_transform : proj_lh * view_lh * model,
			_padding :      unsafe { std::mem::zeroed() },
		};

		unsafe {
			std::ptr::copy_nonoverlapping(&buffer_data, self.p_cbv_data, 1);
		}
	}

	pub fn render(&mut self) -> i32
	{
		self.populate_command_list();

		let vec_command_lists = [self.command_list.as_mut_ptr() as *mut d3d12::ID3D12CommandList];
		unsafe {
			self.command_queue
				.ExecuteCommandLists(u32::try_from(vec_command_lists.len()).unwrap(), vec_command_lists.as_ptr())
		};

		let sync_interval : u32 = 1; // 0 is no-vsync. 1-4 is vsync by N frames.
		let present_flags : u32 = 0;
		let present_parameters = dxgi1_2::DXGI_PRESENT_PARAMETERS {
			DirtyRectsCount : 0,               // update the whole frame
			pDirtyRects :     ptr::null_mut(), // these parameters are ignored when updating the whole frame.
			pScrollRect :     ptr::null_mut(),
			pScrollOffset :   ptr::null_mut(),
		};
		let hr_swap_backbuffer = unsafe { self.swap_chain.Present1(sync_interval, present_flags, &present_parameters) };
		assert!(winerror::SUCCEEDED(hr_swap_backbuffer), "Failed to swap backbuffer. 0x{:x}", hr_swap_backbuffer);

		self.move_to_next_frame();

		return 0;
	}

	pub fn _destroy(&mut self)
	{
		// Ensure that the GPU is no longer referencing resources that are about to be
		// cleaned up by the destructor.
		self.wait_for_gpu();

		unsafe {
			winapi::um::handleapi::CloseHandle(self.fence_event);
		}

		unsafe { self.constant_buffer.Unmap(0, ptr::null()) };
		self.p_cbv_data = ptr::null_mut();
	}

	pub fn populate_command_list(&mut self)
	{
		unsafe {
			let hr_allocator_reset = self.command_allocators[self.frame_index].Reset();
			assert!(
				winerror::SUCCEEDED(hr_allocator_reset),
				"Failed to reset command allocator. 0x{:x}",
				hr_allocator_reset
			);

			let hr_command_reset = self
				.command_list
				.Reset(self.command_allocators[self.frame_index].as_mut_ptr(), self.pipeline_state.as_mut_ptr());
			assert!(winerror::SUCCEEDED(hr_command_reset), "Failed to reset command list. 0x{:x}", hr_command_reset);

			self.command_list.SetGraphicsRootSignature(self.root_signature.as_mut_ptr());

			let mut descriptor_heaps = [self.cbv_descriptor_heap.as_mut_ptr()];
			self.command_list.SetDescriptorHeaps(descriptor_heaps.len() as u32, descriptor_heaps.as_mut_ptr());
			const CBV_SLOT : u32 = 0;
			self.command_list.SetGraphicsRootDescriptorTable(
				CBV_SLOT,
				self.cbv_descriptor_heap.GetGPUDescriptorHandleForHeapStart(),
			);

			self.command_list.RSSetViewports(1, &self.viewport);
			self.command_list.RSSetScissorRects(1, &self.scissor_rect);

			let mut resource_barrier_start = d3d12::D3D12_RESOURCE_BARRIER {
				Type : d3d12::D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
				Flags : d3d12::D3D12_RESOURCE_BARRIER_FLAG_NONE,
				..mem::zeroed()
			};
			*resource_barrier_start.u.Transition_mut() = d3d12::D3D12_RESOURCE_TRANSITION_BARRIER {
				pResource :   self.render_targets[self.frame_index].as_mut_ptr(),
				Subresource : d3d12::D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
				StateBefore : d3d12::D3D12_RESOURCE_STATE_PRESENT,
				StateAfter :  d3d12::D3D12_RESOURCE_STATE_RENDER_TARGET,
			};
			let resource_barrier_start_d3d = std::mem::transmute::<
				&d3d12::D3D12_RESOURCE_BARRIER,
				*const d3d12::D3D12_RESOURCE_BARRIER,
			>(&resource_barrier_start);
			self.command_list.ResourceBarrier(1, resource_barrier_start_d3d);

			let rtv_handle = CD3D12_CPU_DESCRIPTOR_HANDLE::from_offset(
				&self.rtv_descriptor_heap.GetCPUDescriptorHandleForHeapStart(),
				self.frame_index as i32,
				self.rtv_descriptor_size,
			);
			self.command_list.OMSetRenderTargets(1, &rtv_handle.0, FALSE, ptr::null());

			let clear_color : [f32; 4] = [0.0, 0.2, 0.4, 1.0];
			self.command_list.ClearRenderTargetView(rtv_handle.0, &clear_color, 0, ptr::null());
			self.command_list.IASetPrimitiveTopology(d3dcommon::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
			self.command_list.IASetVertexBuffers(0, 1, &self.vertex_buffer_view);
			let vertex_count = 12; // TODO: Make this not hardcoded.
			let instance_count = 1;
			let start_vertex_location = 0;
			let start_instance_location = 0;
			self.command_list.DrawInstanced(
				vertex_count,
				instance_count,
				start_vertex_location,
				start_instance_location,
			);

			let mut resource_barrier_end = d3d12::D3D12_RESOURCE_BARRIER {
				Type : d3d12::D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
				Flags : d3d12::D3D12_RESOURCE_BARRIER_FLAG_NONE,
				..mem::zeroed()
			};
			*resource_barrier_end.u.Transition_mut() = d3d12::D3D12_RESOURCE_TRANSITION_BARRIER {
				pResource :   self.render_targets[self.frame_index].as_mut_ptr(),
				Subresource : d3d12::D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
				StateBefore : d3d12::D3D12_RESOURCE_STATE_RENDER_TARGET,
				StateAfter :  d3d12::D3D12_RESOURCE_STATE_PRESENT,
			};
			let resource_barrier_end_d3d = std::mem::transmute::<
				&d3d12::D3D12_RESOURCE_BARRIER,
				*const d3d12::D3D12_RESOURCE_BARRIER,
			>(&resource_barrier_end);
			self.command_list.ResourceBarrier(1, resource_barrier_end_d3d);

			let hr_command_close = self.command_list.Close();
			assert!(winerror::SUCCEEDED(hr_command_close), "Failed to close command list. 0x{:x}", hr_command_close);
		}
	}

	// Wait for pending GPU work to complete.
	pub fn wait_for_gpu(&mut self)
	{
		let current_fence_index = self.frame_index;
		let current_fence_value = self.fence_values[current_fence_index];

		// Schedule a Signal command in the queue.
		unsafe {
			let hr_signal = self.command_queue.Signal(self.fence.as_mut_ptr(), current_fence_value); // when command queue triggers this, it will set the fense to the given value.
			assert!(winerror::SUCCEEDED(hr_signal), "Failed to signal the comment queue. 0x{:x}", hr_signal);
		}

		// Wait until the fence has been processed.
		unsafe {
			let hr_on_completed = self.fence.SetEventOnCompletion(current_fence_value, self.fence_event); // when the fence is set to this value, trigger the fence_event
			assert!(winerror::SUCCEEDED(hr_on_completed), "Failed to SetEventOnCompletion. 0x{:x}", hr_on_completed);

			// Wait for the fence event (end of command queue)
			let wait_result = WaitForSingleObject(self.fence_event, INFINITE); // wait for the fence event to trigger.
			match wait_result
			{
				0x00000080 => println!("wait_for_previous_frame: WAIT_ABANDONED"),
				0x00000000 => (), // println!("wait_for_previous_frame: WAIT_OBJECT_0"), // SUCCESS
				0x00000102 => println!("wait_for_previous_frame: WAIT_TIMEOUT"),
				0xFFFFFFFF =>
				{
					println!("wait_for_previous_frame: WAIT_FAILED");
					panic!("wait_for_previous_frame failed")
				}
				_ => unreachable!(),
			}
		}

		// Increment the fence value for the current frame.
		self.fence_values[current_fence_index] += 1;
	}

	// Prepare to render the next frame.
	pub fn move_to_next_frame(&mut self)
	{
		// The fence value for this frame was set at the end of the previous call to
		// this function.
		let current_fence_index = self.frame_index;
		let current_fence_value = self.fence_values[current_fence_index];

		// Schedule a Signal command in the queue.
		unsafe {
			let hr_signal = self.command_queue.Signal(self.fence.as_mut_ptr(), current_fence_value); // when command queue triggers this, it will set the fense to the given value.
			assert!(winerror::SUCCEEDED(hr_signal), "Failed to signal the comment queue. 0x{:x}", hr_signal);
		}

		// Update the frame index to the current one.
		self.frame_index = unsafe { self.swap_chain.GetCurrentBackBufferIndex() } as usize;
		let next_fence_index = self.frame_index;
		let next_fence_value = self.fence_values[next_fence_index];

		// If the next frame is not ready to be rendered yet, wait until it is ready.
		if unsafe { self.fence.GetCompletedValue() } < next_fence_value
		{
			let hr_on_completed = unsafe { self.fence.SetEventOnCompletion(next_fence_value, self.fence_event) };
			assert!(winerror::SUCCEEDED(hr_on_completed), "Failed to SetEventOnCompletion. 0x{:x}", hr_on_completed);

			let wait_result = unsafe { WaitForSingleObject(self.fence_event, INFINITE) };
			match wait_result
			{
				0x00000080 => println!("wait_for_previous_frame: WAIT_ABANDONED"),
				0x00000000 => (), // println!("wait_for_previous_frame: WAIT_OBJECT_0"),
				0x00000102 => println!("wait_for_previous_frame: WAIT_TIMEOUT"),
				0xFFFFFFFF =>
				{
					println!("wait_for_previous_frame: WAIT_FAILED");
					panic!("wait_for_previous_frame failed")
				}
				_ => unreachable!(),
			}
		}

		// Ready to begin the next frame. Set the fence value for the next frame.
		self.fence_values[next_fence_index] = current_fence_value + 1;
	}
}
