use crate::engine::Engine;
use log::info;
use vulkano::{
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceExtensions, Features,
    },
    instance::{ApplicationInfo, Instance},
    Version,
};
use winit::window::Window;

pub struct Renderer {}

impl Renderer {
    pub fn new(_engine: &mut Engine, window: &Window) -> Self {
        info!("Initializing the renderer...");

        let required_extensions = vulkano_win::required_extensions();

        let instance = Instance::new(
            Some(&ApplicationInfo {
                engine_name: Some("Zenit Engine".into()),
                engine_version: Some(Version::major_minor(0, 1)),
                ..Default::default()
            }),
            Version::V1_0,
            &required_extensions,
            None,
        )
        .expect("Couldn't create Vulkan instance");

        let surface = vulkano_win::create_vk_surface(window, instance.clone())
            .expect("Couldn't create Vulkan surface");

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        let (physical_device, queue_family) = PhysicalDevice::enumerate(&instance)
            .filter(|&p| p.supported_extensions().is_superset_of(&device_extensions))
            .filter_map(|p| {
                p.queue_families()
                    .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
                    .map(|q| (p, q))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
            })
            .expect("Couldn't find a suitable device!");

        {
            let name = physical_device.properties().device_name.as_str();
            let driver = physical_device
                .properties()
                .driver_name
                .as_ref()
                .map(|x| x.as_str())
                .unwrap_or("unknown");
            let kind = physical_device.properties().device_type;
            info!("GPU properties:");
            info!(" * device name: {}", name);
            info!(" * device driver: {}", driver);
            info!(" * device type: {:?}", kind);
        }

        let (device, mut queues) = Device::new(
            physical_device,
            &Features::none(),
            &physical_device
                .required_extensions()
                .union(&device_extensions),
            [(queue_family, 0.5)].iter().cloned(),
        )
        .expect("Couldn't create a logical device");

        let queue = queues.next().expect("Couldn't get the queue");

        todo!()
    }
}
