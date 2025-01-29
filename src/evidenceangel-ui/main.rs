#![cfg_attr(
    all(not(debug_assertions), not(feature = "windows-keep-console-window")),
    windows_subsystem = "windows"
)]

use std::{env, path::PathBuf, sync::Mutex};

use clap::Parser;
use relm4::{
    gtk::{
        self,
        gio::ApplicationFlags,
        prelude::{ApplicationExt, ApplicationExtManual},
    },
    RelmApp,
};
use tracing_subscriber_multi::*;

mod about;
mod app;
mod author_factory;
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
                env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                    .unwrap_or(PathBuf::from("."))
                    .join("evidenceangel.log"),
                AppendCount::new(3),
                ContentLimit::Lines(1000),
                Compression::OnRotate(0),
            )),
        )))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("failed to initialise logger");

    let cli = Args::parse();

    let app = RelmApp::new("uk.hpkns.EvidenceAngel");
    relm4::main_application().set_flags(ApplicationFlags::HANDLES_OPEN);
    relm4::main_application().connect_open(|_app, _files, _hint| {
        // nothing to do, this is handled by clap...
    });

    lang::initialise_i18n();
    relm4_icons::initialize_icons();
    let display = gtk::gdk::Display::default().unwrap();
    let theme = gtk::IconTheme::for_display(&display);
    theme.add_resource_path("/uk/hpkns/EvidenceAngel/icons/");
    theme.add_resource_path("/uk/hpkns/EvidenceAngel/icons/scalable/actions/");

    app.run::<app::AppModel>(cli.file);
}
