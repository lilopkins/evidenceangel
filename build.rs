fn main() {
    if cfg!(feature = "cli") || cfg!(feature = "ui") {
        println!("cargo::rerun-if-changed=icon.png");

        #[cfg(windows)]
        {
            ico_builder::IcoBuilder::default()
                .add_source_file("icon.png")
                .build_file("icon.ico")
                .unwrap();

            let mut res = winresource::WindowsResource::new();
            res.set_icon("icon.ico");
            res.compile().unwrap();
        }
    }
}
