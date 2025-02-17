use std::{fs, path::PathBuf};

use adw::prelude::*;
use relm4::{
    adw::{self, ApplicationWindow},
    gtk::{self, StringList},
    Component, ComponentParts, ComponentSender, RelmWidgetExt,
};

use crate::{lang, lang_args};

const EXPORT_FORMATS: &[&str] = &["HTML Document", "Excel Workbook", "ZIP Archive of Files"];
const EXPORT_EXTENSIONS: &[&str] = &["html", "xlsx", "zip"];

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
    /// The path of the currently open EVP
    pub package_path: PathBuf,
    /// The name of the test case beinge exported, or None if the whole package is to be exported.
    pub test_case_name: Option<String>,
}

pub struct ExportDialogModel {
    /// The name of the package
    pub package_name: String,
    /// The path of the currently open EVP
    package_directory: PathBuf,
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
                            lang::lookup_with_args("export-title", &lang_args!("target", name.clone()))
                        } else {
                            lang::lookup_with_args("export-title", &lang_args!("target", lang::lookup("export-target-package")))
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
                            connect_selected_notify => ExportInput::_CheckPathValidity,
                        },
                        #[name = "file_row"]
                        adw::EntryRow {
                            set_title: &lang::lookup("export-file-label"),
                            add_suffix = &gtk::Button {
                                set_icon_name: relm4_icons::icon_names::FOLDER_OPEN_FILLED,
                                set_tooltip: &lang::lookup("select"),
                                add_css_class: "flat",
                                connect_clicked => ExportInput::_SelectFile,
                            },
                            connect_entry_activated => ExportInput::_Export,
                            connect_changed => ExportInput::_CheckPathValidity @file_row_changed,
                        },
                    },
                    #[name = "export_btn"]
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
            package_path,
            test_case_name,
        } = init;
        let model = Self {
            package_name,
            package_directory: package_path.parent().unwrap().to_path_buf(),
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

                // evaluate path and determine if a file will be replaced. Update extension if needed.
                let mut path = PathBuf::from(path);

                // Update extension
                let extension = EXPORT_EXTENSIONS[widgets.format_row.selected() as usize];
                path.set_extension(extension);

                // Update text
                widgets.file_row.block_signal(&widgets.file_row_changed);
                if let Some(new_path) = path.as_os_str().to_str() {
                    let cursor_position = widgets.file_row.position();
                    widgets.file_row.set_text(new_path);
                    widgets.file_row.set_position(cursor_position);
                }
                widgets.file_row.unblock_signal(&widgets.file_row_changed);

                // Check if overwriting
                if path.is_relative() {
                    // Make relative to EVP
                    path = self.package_directory.join(path);
                    tracing::debug!("Making path relative to EVP: {path:?}");
                }
                if let Ok(true) = fs::exists(path) {
                    widgets
                        .export_btn
                        .set_label(&lang::lookup("export-submit-replace"));
                    widgets.export_btn.add_css_class("warning");
                } else {
                    widgets.export_btn.set_label(&lang::lookup("export-submit"));
                    widgets.export_btn.remove_css_class("warning");
                }
            }
            ExportInput::_Export => {
                let path = widgets.file_row.text().to_string();
                sender.input(ExportInput::_CheckPathValidity);
                if path.trim().is_empty() {
                    return;
                }
                let mut path = PathBuf::from(path);
                if path.is_relative() {
                    // Make relative to EVP
                    path = self.package_directory.join(path);
                    tracing::debug!("Making path relative to EVP: {path:?}");
                }
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
        self.update_view(widgets, sender);
    }
}
