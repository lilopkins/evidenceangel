#![cfg_attr(
    all(not(debug_assertions), not(feature = "windows-keep-console-window")),
    windows_subsystem = "windows"
)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::used_underscore_items)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::wildcard_imports)]

use std::{env, fs, path::PathBuf, sync::Mutex};

use clap::Parser;
use directories::ProjectDirs;
use relm4::{
    RelmApp,
    gtk::{
        self,
        gio::ApplicationFlags,
        prelude::{ApplicationExt, ApplicationExtManual},
    },
};
use tracing_subscriber_multi::*;

mod about;
mod app;
mod author_factory;
mod custom_metadata_editor_factory;
mod custom_metadata_factory;
mod dialogs;
mod evidence_factory;
mod filter;
mod lang;
mod nav_factory;
mod util;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The file to work with
    #[arg(index = 1)]
    file: Option<PathBuf>,
}

fn main() {
    let dirs = ProjectDirs::from("uk.hpkns", "AngelSuite", "EvidenceAngel")
        .expect("Failed to get directories");
    if let Err(e) = fs::create_dir_all(dirs.cache_dir()) {
        tracing::warn!("Failed to create cache dir (for logs)! {e}");
    }

    let log_path = dirs.cache_dir().join("evidenceangel.log");
    let subscriber = FmtSubscriber::builder()
        .with_max_level(
            if cfg!(debug_assertions) || env::var("EA_DEBUG").is_ok_and(|v| !v.is_empty()) {
                tracing::Level::TRACE
            } else {
                tracing::Level::INFO
            },
        )
        .with_ansi(true)
        .with_writer(Mutex::new(DualWriter::new(
            std::io::stderr(),
            AnsiStripper::new(RotatingFile::new(
                &log_path,
                AppendCount::new(3),
                ContentLimit::Lines(1000),
                Compression::OnRotate(0),
            )),
        )))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("failed to initialise logger");
    tracing::info!("Log path: {log_path:?}");

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        tracing_panic::panic_hook(panic_info);
        prev_hook(panic_info);
    }));

    let cli = Args::parse();

    let app = RelmApp::new("uk.hpkns.EvidenceAngel");
    relm4::main_application().set_flags(ApplicationFlags::HANDLES_OPEN | ApplicationFlags::NON_UNIQUE);
    relm4::main_application().connect_open(|_app, _files, _hint| {
        // nothing to do, this is handled by clap...
    });

    lang::initialise_i18n();
    relm4_icons::initialize_icons();
    gtk::gio::resources_register_include!("hicolor-icon.gresource").unwrap();
    let display = gtk::gdk::Display::default().unwrap();
    let theme = gtk::IconTheme::for_display(&display);
    theme.add_resource_path("/uk/hpkns/EvidenceAngel/icons/");
    theme.add_resource_path("/uk/hpkns/EvidenceAngel/icons/hicolor/scalable/");
    theme.add_resource_path("/uk/hpkns/EvidenceAngel/icons/scalable/actions/");

    app.run::<app::AppModel>(cli.file);
}
