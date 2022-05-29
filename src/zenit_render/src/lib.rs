use vulkano::instance::{Instance, InstanceCreateInfo};

pub fn init() {
    let instance = Instance::new(InstanceCreateInfo {
        application_name: Some("Zenit Engine".to_string()),
        application_version: vulkano::Version::major_minor(0, 1),
        enabled_extensions: todo!(),
        engine_name: Some("Zenit Engine".to_string()),
        engine_version: vulkano::Version::major_minor(0, 1),
        max_api_version: Some(vulkano::Version::V1_0),
        ..Default::default()
    }).expect("couldn't initialize Vulkan");
}
