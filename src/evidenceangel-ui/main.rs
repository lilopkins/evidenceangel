#![cfg_attr(
    all(not(debug_assertions), not(feature = "windows-keep-console-window")),
    windows_subsystem = "windows"
)]

use std::path::PathBuf;

use clap::Parser;
use relm4::{
    gtk::{
        self,
        gio::ApplicationFlags,
        prelude::{ApplicationExt, ApplicationExtManual},
    },
    RelmApp,
};

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
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now(),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("evidenceangel", log::LevelFilter::Debug)
        .level_for("evidenceangel_ui", log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("evidenceangel.log").expect("Couldn't open log file."))
        .apply()
        .expect("Couldn't start logger!");

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
