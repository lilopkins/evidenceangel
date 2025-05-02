use relm4::{adw, gtk, SimpleComponent};

use crate::lang;

pub struct AppAbout;

#[relm4::component(pub)]
impl SimpleComponent for AppAbout {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        #[root]
        #[name = "about"]
        adw::AboutWindow {
            set_application_icon: "evidenceangel",
            set_application_name: &lang::lookup("app-name"),
            set_version: env!("CARGO_PKG_VERSION"),
            set_copyright: "© 2024 Lily Hopkins",
            set_license_type: gtk::License::Gpl30Only,
            set_issue_url: &support_url,
            set_developer_name: "Lily Hopkins",
            set_debug_info: &log_data,

            add_acknowledgement_section: (Some(&lang::lookup("acknowledgements-code-title")), &["Lily Hopkins", "Eden Turner"]),
            add_acknowledgement_section: (Some(&lang::lookup("acknowledgements-testing-title")), &["John Chander", "Eden Turner"]),
            add_acknowledgement_section: (Some(&lang::lookup("acknowledgements-translations-title")), &["Lily Hopkins"]),
            add_legal_section: ("GTK",                      None, gtk::License::Gpl20Only, None),
            add_legal_section: ("Adwaita",                  None, gtk::License::Gpl20Only, None),
            add_legal_section: ("base64",                   None, gtk::License::MitX11,    None),
            add_legal_section: ("build_html",               None, gtk::License::MitX11,    None),
            add_legal_section: ("chrono",                   None, gtk::License::MitX11,    None),
            add_legal_section: ("clap",                     None, gtk::License::MitX11,    None),
            add_legal_section: ("clap_complete",            None, gtk::License::MitX11,    None),
            add_legal_section: ("clap-verbosity-flag",      None, gtk::License::MitX11,    None),
            add_legal_section: ("fluent",                   None, gtk::License::MitX11,    None),
            add_legal_section: ("fluent-templates",         None, gtk::License::MitX11,    None),
            add_legal_section: ("getset",                   None, gtk::License::MitX11,    None),
            add_legal_section: ("html-escape",              None, gtk::License::MitX11,    None),
            add_legal_section: ("infer",                    None, gtk::License::MitX11,    None),
            add_legal_section: ("jsonschema",               None, gtk::License::MitX11,    None),
            add_legal_section: ("once_cell",                None, gtk::License::MitX11,    None),
            add_legal_section: ("parse_datetime",           None, gtk::License::MitX11,    None),
            add_legal_section: ("relm4",                    None, gtk::License::MitX11,    None),
            add_legal_section: ("relm4-icons",              None, gtk::License::MitX11,    None),
            add_legal_section: ("rust_xlsxwriter",          None, gtk::License::MitX11,    None),
            add_legal_section: ("serde",                    None, gtk::License::MitX11,    None),
            add_legal_section: ("serde_json",               None, gtk::License::MitX11,    None),
            add_legal_section: ("sha256",                   None, gtk::License::MitX11,    None),
            add_legal_section: ("sys-locale",               None, gtk::License::MitX11,    None),
            add_legal_section: ("thiserror",                None, gtk::License::MitX11,    None),
            add_legal_section: ("tracing",                  None, gtk::License::MitX11,    None),
            add_legal_section: ("tracing-subscriber",       None, gtk::License::MitX11,    None),
            add_legal_section: ("tracing-subscriber-multi", None, gtk::License::MitX11,    None),
            add_legal_section: ("uuid",                     None, gtk::License::MitX11,    None),
            add_legal_section: ("zip",                      None, gtk::License::MitX11,    None),
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = AppAbout;

        let log_data = format!(
            "Debug data generated at: {}\nSoftware version: {}\nLocale: {} (system wanted: {:?})",
            chrono::Local::now(),
            env!("CARGO_PKG_VERSION"),
            lang::current_locale(),
            sys_locale::get_locales().collect::<Vec<_>>(),
        );
        let support_url = std::env::var("EA_LOCAL_SUPPORT_CONTACT")
            .unwrap_or("https://github.com/lilopkins/evidenceangel".to_string());
        let widgets = view_output!();

        relm4::ComponentParts { model, widgets }
    }
}
