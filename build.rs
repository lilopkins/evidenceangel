fn main() {
    if cfg!(feature = "cli") || cfg!(feature = "ui") {
        // Build documentation
        println!("cargo::rerun-if-changed=docs/book.toml");
        println!("cargo::rerun-if-changed=docs/src");
        let docs_book =
            mdbook::MDBook::load("docs").expect("Failed to load documentation for EvidenceAngel");
        docs_book
            .build()
            .expect("Failed to build documentation for EvidenceAngel");

        // Build icon
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
