use std::{
    collections::HashMap,
    io::{BufReader, Read},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use adw::prelude::*;
use evidenceangel::{Evidence, EvidenceData, EvidenceKind, EvidencePackage, MediaFile};
use relm4::{
    adw::{self, ApplicationWindow},
    gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt,
};

use crate::{filter, lang};

#[derive(Debug)]
pub enum AddEvidenceInput {
    Present(ApplicationWindow),
    _AddEvidence,
    _SelectFile,
    _FileSelected(PathBuf),
}

#[derive(Debug)]
pub enum AddEvidenceOutput {
    AddEvidence(Evidence),
    Error { title: String, message: String },
}

pub struct AddTextEvidenceDialogModel {}

#[relm4::component(pub)]
impl Component for AddTextEvidenceDialogModel {
    type Input = AddEvidenceInput;
    type Output = AddEvidenceOutput;
    type CommandOutput = ();
    type Init = Arc<RwLock<EvidencePackage>>;

    view! {
        #[root]
        adw::Dialog {
            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: &lang::lookup("add-evidence-title"),
                    }
                },
                set_width_request: 400,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,
                    set_margin_all: 16,

                    adw::PreferencesGroup {
                        #[name = "text_entry"]
                        adw::EntryRow {
                            set_title: &lang::lookup("add-evidence-text-label"),
                            connect_entry_activated => AddEvidenceInput::_AddEvidence,
                        },
                    },
                    gtk::Button {
                        set_label: &lang::lookup("add-evidence-submit"),
                        add_css_class: "pill",
                        add_css_class: "suggested-action",
                        set_halign: gtk::Align::Center,

                        connect_clicked => AddEvidenceInput::_AddEvidence,
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AddTextEvidenceDialogModel {};
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
            AddEvidenceInput::Present(window) => {
                root.present(Some(&window));
            }
            AddEvidenceInput::_AddEvidence => {
                let content = widgets.text_entry.text().to_string();
                let ev = Evidence::new(EvidenceKind::Text, EvidenceData::Text { content });
                let _ = sender.output(AddEvidenceOutput::AddEvidence(ev));
                root.close();
            }
            _ => (),
        }
        self.update_view(widgets, sender)
    }
}

pub struct AddHttpEvidenceDialogModel {}

#[relm4::component(pub)]
impl Component for AddHttpEvidenceDialogModel {
    type Input = AddEvidenceInput;
    type Output = AddEvidenceOutput;
    type CommandOutput = ();
    type Init = Arc<RwLock<EvidencePackage>>;

    view! {
        #[root]
        adw::Dialog {
            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: &lang::lookup("add-evidence-title"),
                    }
                },
                set_width_request: 400,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,
                    set_margin_all: 16,

                    gtk::Box {
                        set_spacing: 8,
                        set_height_request: 300,
                        set_width_request: 500,

                        gtk::ScrolledWindow {
                            set_hexpand: true,

                            gtk::Frame {
                                set_label: Some(&lang::lookup("add-evidence-http-req-label")),
                                #[name = "req_entry"]
                                gtk::TextView {
                                    set_monospace: true,
                                },
                            },
                        },
                        gtk::ScrolledWindow {
                            set_hexpand: true,

                            gtk::Frame {
                                set_label: Some(&lang::lookup("add-evidence-http-res-label")),
                                #[name = "res_entry"]
                                gtk::TextView {
                                    set_monospace: true,
                                },
                            },
                        },
                    },
                    adw::PreferencesGroup {
                        #[name = "caption_entry"]
                        adw::EntryRow {
                            set_title: &lang::lookup("add-evidence-http-caption-label"),
                            connect_entry_activated => AddEvidenceInput::_AddEvidence,
                        },
                    },
                    gtk::Button {
                        set_label: &lang::lookup("add-evidence-submit"),
                        add_css_class: "pill",
                        add_css_class: "suggested-action",
                        set_halign: gtk::Align::Center,

                        connect_clicked => AddEvidenceInput::_AddEvidence,
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AddHttpEvidenceDialogModel {};
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
            AddEvidenceInput::Present(window) => {
                root.present(Some(&window));
            }
            AddEvidenceInput::_AddEvidence => {
                let req_buffer = widgets.req_entry.buffer();
                let req_content = req_buffer
                    .text(&req_buffer.start_iter(), &req_buffer.end_iter(), false)
                    .to_string();
                let res_buffer = widgets.res_entry.buffer();
                let res_content = res_buffer
                    .text(&res_buffer.start_iter(), &res_buffer.end_iter(), false)
                    .to_string();
                let content = format!("{req_content}\n\n\x1e{res_content}");
                let mut ev = Evidence::new(EvidenceKind::Http, EvidenceData::Text { content });
                let caption_text = widgets.caption_entry.text().to_string();
                if !caption_text.trim().is_empty() {
                    ev.set_caption(Some(caption_text.trim().to_string()));
                }
                let _ = sender.output(AddEvidenceOutput::AddEvidence(ev));
                root.close();
            }
            _ => (),
        }
        self.update_view(widgets, sender)
    }
}

pub struct AddImageEvidenceDialogModel {
    package: Arc<RwLock<EvidencePackage>>,
}

#[relm4::component(pub)]
impl Component for AddImageEvidenceDialogModel {
    type Input = AddEvidenceInput;
    type Output = AddEvidenceOutput;
    type CommandOutput = ();
    type Init = Arc<RwLock<EvidencePackage>>;

    view! {
        #[root]
        adw::Dialog {
            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: &lang::lookup("add-evidence-title"),
                    }
                },
                set_width_request: 400,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,
                    set_margin_all: 16,

                    adw::PreferencesGroup {
                        #[name = "file_row"]
                        adw::EntryRow {
                            set_title: &lang::lookup("add-evidence-image-label"),
                            add_suffix = &gtk::Button {
                                set_icon_name: relm4_icons::icon_names::FOLDER_OPEN_FILLED,
                                add_css_class: "flat",
                                connect_clicked => AddEvidenceInput::_SelectFile,
                            },
                            connect_entry_activated => AddEvidenceInput::_AddEvidence,
                        },
                        #[name = "caption_entry"]
                        adw::EntryRow {
                            set_title: &lang::lookup("add-evidence-image-caption-label"),
                            connect_entry_activated => AddEvidenceInput::_AddEvidence,
                        },
                    },
                    gtk::Button {
                        set_label: &lang::lookup("add-evidence-submit"),
                        add_css_class: "pill",
                        add_css_class: "suggested-action",
                        set_halign: gtk::Align::Center,

                        connect_clicked => AddEvidenceInput::_AddEvidence,
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
        let model = AddImageEvidenceDialogModel { package: init };
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
            AddEvidenceInput::Present(window) => {
                root.present(Some(&window));
            }
            AddEvidenceInput::_AddEvidence => {
                let path = widgets.file_row.text().to_string();
                // Read file data
                let read_data = move || {
                    use std::fs::File;

                    let mut buf = vec![];
                    let mut br = BufReader::new(File::open(path)?);
                    br.read_to_end(&mut buf)?;

                    Ok(buf)
                };
                let data: Result<Vec<u8>, std::io::Error> = read_data();
                if let Err(e) = data {
                    sender
                        .output(AddEvidenceOutput::Error {
                            title: lang::lookup("add-evidence-image-failed"),
                            message: lang::lookup_with_args("add-evidence-image-failed-message", {
                                let mut map = HashMap::new();
                                map.insert("error", e.to_string().into());
                                map
                            }),
                        })
                        .unwrap();
                    return;
                }
                let data = data.unwrap();

                // Add media to package
                let mut pkg = self.package.write().unwrap();
                let media = MediaFile::from(data);
                let hash = media.hash();
                if let Err(e) = pkg.add_media(media) {
                    sender
                        .output(AddEvidenceOutput::Error {
                            title: lang::lookup("add-evidence-image-failed"),
                            message: lang::lookup_with_args("add-evidence-image-failed-message", {
                                let mut map = HashMap::new();
                                map.insert("error", e.to_string().into());
                                map
                            }),
                        })
                        .unwrap();
                    return;
                }

                // Return media hash
                let mut ev = Evidence::new(EvidenceKind::Image, EvidenceData::Media { hash });
                let caption_text = widgets.caption_entry.text().to_string();
                if !caption_text.trim().is_empty() {
                    ev.set_caption(Some(caption_text.trim().to_string()));
                }
                let _ = sender.output(AddEvidenceOutput::AddEvidence(ev));
                root.close();
            }
            AddEvidenceInput::_SelectFile => {
                // Open file selector
                let dialog = gtk::FileDialog::builder()
                    .modal(true)
                    .title(lang::lookup("header-open"))
                    .filters(&filter::filter_list(vec![filter::images()]))
                    .initial_folder(&gtk::gio::File::for_path("."))
                    .build();

                let sender_c = sender.clone();
                dialog.open(
                    Some(&root.toplevel_window().unwrap()),
                    Some(&relm4::gtk::gio::Cancellable::new()),
                    move |res| {
                        if let Ok(file) = res {
                            let path = file.path().unwrap();
                            // Open this package
                            sender_c.input(AddEvidenceInput::_FileSelected(path));
                        }
                    },
                );
            }
            AddEvidenceInput::_FileSelected(path) => {
                widgets.file_row.set_text(path.to_str().unwrap_or_default());
            }
        }
        self.update_view(widgets, sender)
    }
}
