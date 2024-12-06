use std::{collections::HashMap, path::PathBuf};

use adw::prelude::*;
use relm4::{
    adw::{self, ApplicationWindow},
    gtk::{self, StringList},
    Component, ComponentParts, ComponentSender, RelmWidgetExt,
};

use crate::lang;

const EXPORT_FORMATS: &[&str] = &["HTML Document", "Excel Workbook"];
const EXPORT_EXTENSIONS: &[&str] = &["html", "xlsx"];

#[derive(Debug)]
pub enum ExportInput {
    Present(ApplicationWindow),
    _Export,
    _SelectFile,
    _FileSelected(PathBuf),
    _CheckPathValidity,
}

#[derive(Debug)]
pub enum ExportOutput {
    Export { format: String, path: PathBuf },
}

#[derive(Debug)]
pub struct ExportDialogInit {
    /// The name of the package
    pub package_name: String,
    /// The name of the test case beinge exported, or None if the whole package is to be exported.
    pub test_case_name: Option<String>,
}

pub struct ExportDialogModel {
    /// The name of the package
    pub package_name: String,
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
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: &if let Some(name) = &model.test_case_name {
                            lang::lookup_with_args("export-title", {
                                let mut map = HashMap::new();
                                map.insert("target", name.clone().into());
                                map
                            })
                        } else {
                            lang::lookup_with_args("export-title", {
                                let mut map = HashMap::new();
                                map.insert("target", lang::lookup("export-target-package").into());
                                map
                            })
                        }
                    }
                },
                set_width_request: 400,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,
                    set_margin_all: 16,

                    adw::PreferencesGroup {
                        #[name = "format_row"]
                        adw::ComboRow {
                            set_title: &lang::lookup("export-format-label"),
                            set_model: Some(&StringList::new(EXPORT_FORMATS)),
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
                            connect_changed => ExportInput::_CheckPathValidity,
                        },
                    },
                    gtk::Button {
                        set_label: &lang::lookup("export-submit"),
                        add_css_class: "pill",
                        add_css_class: "suggested-action",
                        set_halign: gtk::Align::Center,

                        connect_clicked => ExportInput::_Export,
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let Self::Init {
            package_name,
            test_case_name,
        } = init;
        let model = Self {
            package_name,
            test_case_name,
        };
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
            ExportInput::_CheckPathValidity => {
                let path = widgets.file_row.text().to_string();
                if path.trim().is_empty() {
                    widgets.file_row.add_css_class("error");
                } else {
                    widgets.file_row.remove_css_class("error");
                }
            }
            ExportInput::_Export => {
                let path = widgets.file_row.text().to_string();
                sender.input(ExportInput::_CheckPathValidity);
                if path.trim().is_empty() {
                    return;
                }
                let mut path = PathBuf::from(path);
                // Update extension
                let extension = EXPORT_EXTENSIONS[widgets.format_row.selected() as usize];
                path.set_extension(extension);

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
                    .initial_name(self.test_case_name.as_ref().unwrap_or(&self.package_name))
                    .accept_label(lang::lookup("select"))
                    .build();

                let sender_c = sender.clone();
                let extension = EXPORT_EXTENSIONS[widgets.format_row.selected() as usize];
                dialog.save(
                    Some(&root.toplevel_window().unwrap()),
                    Some(&relm4::gtk::gio::Cancellable::new()),
                    move |res| {
                        if let Ok(file) = res {
                            let mut path = file.path().unwrap();
                            path.set_extension(extension);
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
