use relm4::{gtk, RelmApp};

mod about;
mod app;
mod filter;
mod lang;
mod nav_factory;
mod author_factory;
mod dialogs;

fn main() {
    pretty_env_logger::init();

    let app = RelmApp::new("uk.hpkns.EvidenceAngel");

    lang::initialise_i18n();
    relm4_icons::initialize_icons();
    let display = gtk::gdk::Display::default().unwrap();
    let theme = gtk::IconTheme::for_display(&display);
    theme.add_resource_path("/uk/hpkns/EvidenceAngel/icons/");
    theme.add_resource_path("/uk/hpkns/EvidenceAngel/icons/scalable/actions/");

    app.run::<app::AppModel>(());
}
