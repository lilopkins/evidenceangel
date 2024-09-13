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
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::PreferencesGroup {
                    set_title: &lang::lookup("add-evidence-title"),
                    set_margin_all: 16,

                    #[name = "text_entry"]
                    adw::EntryRow {
                        set_title: &lang::lookup("add-evidence-text-label"),
                    },
                },
                gtk::Separator {
                    set_orientation: gtk::Orientation::Horizontal,
                },
                gtk::Button {
                    set_label: &lang::lookup("add-evidence-submit"),
                    add_css_class: "flat",

                    connect_clicked => AddEvidenceInput::_AddEvidence,
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
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::PreferencesGroup {
                    set_title: &lang::lookup("add-evidence-title"),
                    set_margin_all: 16,

                    #[name = "req_entry"]
                    adw::EntryRow {
                        set_title: &lang::lookup("add-evidence-http-req-label"),
                    },
                    #[name = "res_entry"]
                    adw::EntryRow {
                        set_title: &lang::lookup("add-evidence-http-res-label"),
                    },
                },
                gtk::Separator {
                    set_orientation: gtk::Orientation::Horizontal,
                },
                gtk::Button {
                    set_label: &lang::lookup("add-evidence-submit"),
                    add_css_class: "flat",

                    connect_clicked => AddEvidenceInput::_AddEvidence,
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
                let req_content = widgets.req_entry.text().to_string();
                let res_content = widgets.res_entry.text().to_string();
                let content = format!("{req_content}\n\n\x1e{res_content}");
                let ev = Evidence::new(EvidenceKind::Http, EvidenceData::Text { content });
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
    selected_file: Option<PathBuf>,
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
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_width_request: 400,

                adw::PreferencesGroup {
                    set_title: &lang::lookup("add-evidence-title"),
                    set_margin_all: 16,

                    #[name = "file_row"]
                    adw::ActionRow {
                        set_title: &lang::lookup("add-evidence-image-label"),
                        set_activatable: true,
                        connect_activated => AddEvidenceInput::_SelectFile,
                    },
                    #[name = "caption_entry"]
                    adw::EntryRow {
                        set_title: &lang::lookup("add-evidence-image-caption-label"),
                    },
                },
                gtk::Separator {
                    set_orientation: gtk::Orientation::Horizontal,
                },
                gtk::Button {
                    set_label: &lang::lookup("add-evidence-submit"),
                    add_css_class: "flat",

                    connect_clicked => AddEvidenceInput::_AddEvidence,
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AddImageEvidenceDialogModel {
            package: init,
            selected_file: None,
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
            AddEvidenceInput::Present(window) => {
                root.present(Some(&window));
            }
            AddEvidenceInput::_AddEvidence => {
                if self.selected_file.is_none() {
                    root.close();
                    return;
                }
                // Read file data
                let path = self.selected_file.clone().unwrap();
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
                widgets
                    .file_row
                    .set_subtitle(path.to_str().unwrap_or_default());
                self.selected_file = Some(path);
            }
        }
        self.update_view(widgets, sender)
    }
}
