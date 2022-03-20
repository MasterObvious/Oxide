use oxide::HelloTriangleApplication;

fn main() {
    env_logger::init();

    let mut triangle_app = match HelloTriangleApplication::new() {
        Err(error) => panic!("Failed to create application. Cause: '{}'", error),
        Ok(app) => app,
    };
    triangle_app.run();
}
