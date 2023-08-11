use anyhow::Result;
use imgui_rs_vulkan_renderer::Options;
use std::time::Instant;
use winit::window::Window;
use crate::vulkan_helpers::vh::{Data, self};

#[cfg(feature = "goop_imgui")]
use imgui::*;
#[cfg(feature = "goop_imgui")]
use imgui_winit_support::WinitPlatform;

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

pub struct Renderer
{
	_entry: ash::Entry, // needs to live as long as other vulkan objects
	instance: ash::Instance,
	device: ash::Device,
	surface: ash::extensions::khr::Surface,
	data: Data,

	#[cfg(feature = "goop_imgui")]
    pub renderer: imgui_rs_vulkan_renderer::Renderer,
}

impl Renderer
{
	pub fn init(window: &Window, app_name: &str, imgui: &mut Context) -> Result<Self>
	{
		log::info!("Initializing Renderer........");
		let mut data = Data::default();
		let entry = unsafe { ash::Entry::load()? };
		let instance = vh::create_instance(&entry, window, VALIDATION_ENABLED, &mut data, app_name)?;
		let surface = vh::create_surface(&entry, &instance, window, &mut data)?;
		let device = vh::create_logical_device(&instance, &surface, &mut data)?;
		vh::set_msaa_samples(&instance, &mut data)?;
		vh::create_swapchain(&instance, &device, &surface, window, &mut data)?;
		vh::create_swapchain_image_views(&device, &mut data)?;
		vh::create_render_pass(&instance, &device, &mut data)?;
		vh::create_descriptor_set_layout(&device, &mut data)?;
		vh::create_pipeline(&device, &mut data)?;
		vh::create_command_pools(&instance, &device, &surface, &mut data)?;
		vh::create_color_objects(&instance, &device, &mut data)?;
		vh::create_depth_objects(&instance, &device, &mut data)?;
		vh::create_framebuffers(&device, &mut data)?;
		vh::create_texture_image(&instance, &device, &mut data)?;
		vh::create_texture_image_views(&device, &mut data)?;
		vh::create_texture_sampler(&device, &mut data)?;
		vh::load_model(&mut data)?;
		vh::create_vertex_buffer(&instance, &device, &mut data)?;
		vh::create_index_buffer(&instance, &device, &mut data)?;
		vh::create_uniform_buffers(&instance, &device, &mut data)?;
		vh::create_descriptor_pool(&device, &mut data)?;
		vh::create_descriptor_sets(&device, &mut data)?;
		vh::create_command_buffers(&device, &mut data)?;
		vh::create_sync_objects(&device, &mut data)?;

		log::info!("Renderer Initialized Successfully");

		#[cfg(feature = "goop_imgui")]
		let renderer = imgui_rs_vulkan_renderer::Renderer::with_default_allocator(
			&instance,
			data.physical_device,
			device.clone(),
			data.graphics_queue,
			data.graphics_command_pool,
			data.render_pass,
			imgui,
			Some(Options
				{
					in_flight_frames: 3,
					enable_depth_test: false,
					enable_depth_write: false,
				}
			),
		)?;

		Ok(Renderer
		{
			_entry: entry,
			instance,
			device,
			surface,
			data,
			#[cfg(feature = "goop_imgui")]
			renderer,
		})
	}

	#[cfg(not(feature = "goop_imgui"))]
	pub fn render(&mut self, window: &Window, start: Instant)
	{
		vh::render(
			&self.instance,
			&self.device,
			&self.surface,
			&window,
			&mut self.data,
			&start
		).unwrap();
	}

	#[cfg(feature = "goop_imgui")]
	pub fn render(&mut self, window: &Window, start: Instant, imgui: &mut Context, platform: &mut WinitPlatform)
	{
		platform
			.prepare_frame(imgui.io_mut(), &window)
			.expect("Failed to prepare frame");

		let ui = imgui.frame();
		ui.show_demo_window(&mut true);
		platform.prepare_render(&ui, &window);
		let draw_data = imgui.render();

		vh::render(
			&self.instance,
			&self.device,
			&self.surface,
			&window,
			&mut self.data,
			&start,
			&mut self.renderer,
			&draw_data,
		).unwrap();
	}


	pub fn resize(&mut self)
	{
		self.data.resized = true;
	}
}

impl Drop for Renderer
{
	fn drop(&mut self)
	{
		log::info!("Destroying Renderer........");
		unsafe
		{
			return vh::destroy(&self.instance, &self.device, &self.surface, &self.data);
		}
	}
}
