fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resources = winres::WindowsResource::new();
        resources.set_icon("assets/app_icon.ico");
        resources
            .compile()
            .expect("Could not embed the Windows icon");
    }
}
