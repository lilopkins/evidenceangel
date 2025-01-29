#[cfg(windows)]
fn main() {
    println!("cargo::rerun-if-changed=icon.png");

    ico_builder::IcoBuilder::default()
        .add_source_file("icon.png")
        .build_file("icon.ico")
        .unwrap();

    let mut res = winres::WindowsResource::new();
    res.set_icon("icon.ico");
    res.compile().unwrap();
}

#[cfg(not(windows))]
fn main() {}
