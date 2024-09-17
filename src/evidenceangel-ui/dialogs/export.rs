use std::path::PathBuf;

use adw::prelude::*;
use relm4::{
    adw::{self, ApplicationWindow},
    gtk::{self, StringList},
    Component, ComponentParts, ComponentSender, RelmWidgetExt,
};

use crate::lang;

const EXPORT_FORMATS: &[&str] = &["HTML", "Excel"];

#[derive(Debug)]
pub enum ExportInput {
    Present(ApplicationWindow),
    _Export,
    _SelectFile,
    _FileSelected(PathBuf),
}

#[derive(Debug)]
pub enum ExportOutput {
    Export { format: String, path: PathBuf },
}

#[derive(Debug)]
pub struct ExportDialogInit {
    /// The name of the test case beinge exported, or None if the whole package is to be exported.
    pub test_case_name: Option<String>,
}

pub struct ExportDialogModel {
    /// The name of the test case beinge exported, or None if the whole package is to be exported.
    test_case_name: Option<String>,
}

#[relm4::component(pub)]
impl Component for ExportDialogModel {
    type Input = ExportInput;
    type Output = ExportOutput;
    type CommandOutput = ();
    type Init = ExportDialogInit;

    view! {
        #[root]
        adw::Dialog {
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_width_request: 400,

                adw::PreferencesGroup {
                    set_title: &lang::lookup("export-title"),
                    set_margin_all: 16,

                    #[name = "name"]
                    adw::ActionRow {
                        set_title: &lang::lookup("export-target-label"),
                        set_subtitle: &if let Some(name) = &model.test_case_name {
                            name.clone()
                        } else {
                            lang::lookup("export-target-package")
                        }
                    },
                    #[name = "file_row"]
                    adw::EntryRow {
                        set_title: &lang::lookup("export-file-label"),
                        add_suffix = &gtk::Button {
                            set_icon_name: relm4_icons::icon_names::FOLDER_OPEN_FILLED,
                            add_css_class: "flat",
                            connect_clicked => ExportInput::_SelectFile,
                        },
                        connect_entry_activated => ExportInput::_Export,
                    },
                    #[name = "format_row"]
                    adw::ComboRow {
                        set_title: &lang::lookup("export-format-label"),
                        set_model: Some(&StringList::new(EXPORT_FORMATS)),
                    },
                },
                gtk::Separator {
                    set_orientation: gtk::Orientation::Horizontal,
                },
                gtk::Button {
                    set_label: &lang::lookup("export-submit"),
                    add_css_class: "flat",

                    connect_clicked => ExportInput::_Export,
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let Self::Init { test_case_name } = init;
        let model = Self { test_case_name };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            ExportInput::Present(window) => {
                root.present(Some(&window));
            }
            ExportInput::_Export => {
                let path = widgets.file_row.text().to_string();
                let path = PathBuf::from(path);
                let format = EXPORT_FORMATS[widgets.format_row.selected() as usize].to_lowercase();
                let _ = sender.output(ExportOutput::Export { format, path });
                root.close();
            }
            ExportInput::_SelectFile => {
                // Open file selector
                let dialog = gtk::FileDialog::builder()
                    .modal(true)
                    .title(lang::lookup("header-open"))
                    .initial_folder(&gtk::gio::File::for_path("."))
                    .accept_label(lang::lookup("select"))
                    .build();

                let sender_c = sender.clone();
                dialog.open(
                    Some(&root.toplevel_window().unwrap()),
                    Some(&relm4::gtk::gio::Cancellable::new()),
                    move |res| {
                        if let Ok(file) = res {
                            let path = file.path().unwrap();
                            // Open this package
                            sender_c.input(ExportInput::_FileSelected(path));
                        }
                    },
                );
            }
            ExportInput::_FileSelected(path) => {
                widgets.file_row.set_text(path.to_str().unwrap_or_default());
            }
        }
        self.update_view(widgets, sender)
    }
}
