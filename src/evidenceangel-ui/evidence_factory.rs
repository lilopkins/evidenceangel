use std::{collections::HashMap, sync::{Arc, RwLock}};

use adw::prelude::*;
use evidenceangel::{Evidence, EvidenceData, EvidenceKind, EvidencePackage};
#[allow(unused_imports)]
use gtk::prelude::*;
use relm4::{
    adw,
    factory::FactoryView,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    FactorySender, RelmWidgetExt,
};

use crate::lang;
use crate::util::BoxedEvidenceJson;

const EVIDENCE_HEIGHT_REQUEST: i32 = 300;
const HTTP_SEPARATOR: char = '\x1e';

pub struct EvidenceFactoryModel {
    index: DynamicIndex,
    evidence: Arc<RwLock<Evidence>>,
    package: Arc<RwLock<EvidencePackage>>,
}

impl EvidenceFactoryModel {
    fn get_data(&self) -> Vec<u8> {
        log::debug!("Got some {:?} data", self.evidence.read().unwrap().kind());
        match self.evidence.read().unwrap().value() {
            EvidenceData::Text { content } => content.as_bytes().to_vec(),
            EvidenceData::Base64 { data } => data.clone(),
            EvidenceData::Media { hash } => {
                log::debug!("Fetching media with hash {hash}");
                let mut pkg = self.package.write().unwrap();
                log::debug!("Got package instance!");
                let media = pkg.get_media(hash).ok().flatten();
                log::debug!("Got media {media:?}");
                if let Some(media) = media {
                    media.data().clone()
                } else {
                    lang::lookup("invalid-data").as_bytes().to_vec()
                }
            }
        }
    }

    fn get_data_as_string(&self) -> String {
        log::debug!("Converting media to string...");
        String::from_utf8(self.get_data()).unwrap_or(lang::lookup("invalid-data"))
    }

    fn get_data_as_texture(&self) -> Option<gtk::gdk::Texture> {
        log::debug!("Converting media to texture...");
        let glib_bytes = gtk::glib::Bytes::from_owned(self.get_data().clone());
        let r = gtk::gdk::Texture::from_bytes(&glib_bytes).ok();
        log::debug!("Resultant texture: {r:?}");
        r
    }
}

#[derive(Debug)]
pub enum EvidenceFactoryInput {
    /// Set the text for a text evidence object. If not text evidence, ignore.
    TextSetText(String),
    /// Set the HTTP request text. If not HTTP evidence, ignore.
    HttpSetRequest(String),
    /// Set the HTTP response text. If not HTTP evidence, ignore.
    HttpSetResponse(String),
    /// Set the caption for this evidence.
    SetCaption(String),
    MoveUp,
    MoveDown,
    Delete,
}

#[derive(Debug)]
pub enum EvidenceFactoryOutput {
    /// Replace the evidence at the given position. This MUST NOT trigger an update to the interface.
    UpdateEvidence(DynamicIndex, Evidence),
    /// Delete evidence at the given position. This MUST trigger an update to the interface.
    /// Second parameter defines if it is user inititiated
    DeleteEvidence(DynamicIndex, bool),
    /// `InsertEvidenceAt` MUST be followed by a `DeleteEvidence` call as it is only triggered by a data move.
    /// As such, it MUST NOT trigger an update to the interface.
    InsertEvidenceAt(DynamicIndex, isize, Evidence),
}

pub struct EvidenceFactoryInit {
    pub evidence: Evidence,
    pub package: Arc<RwLock<EvidencePackage>>,
}

#[relm4::factory(pub)]
impl FactoryComponent for EvidenceFactoryModel {
    type ParentWidget = gtk::Box;
    type Input = EvidenceFactoryInput;
    type Output = EvidenceFactoryOutput;
    type Init = EvidenceFactoryInit;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            add_controller = gtk::DragSource {
                set_actions: gtk::gdk::DragAction::MOVE,

                connect_prepare => move |_slf, _x, _y| {
                    let dnd_data = BoxedEvidenceJson::new((*ev.read().unwrap()).clone());
                    log::debug!("Drag data started: {dnd_data:?}");
                    Some(gtk::gdk::ContentProvider::for_value(&dnd_data.to_value()))
                },

                connect_drag_end[sender, index] => move |_slf, _drag, delete_data| {
                    if delete_data {
                        log::debug!("Deleting drag start item");
                        sender.output(EvidenceFactoryOutput::DeleteEvidence(index.clone(), false)).unwrap();
                    }
                }
            },
            add_controller = gtk::DropTarget {
                set_actions: gtk::gdk::DragAction::MOVE,
                set_types: &[BoxedEvidenceJson::static_type()],

                connect_drop[sender, index] => move |_slf, val, _x, _y| {
                    log::debug!("Dropped type: {:?}", val.type_());
                    if let Ok(data) = val.get::<BoxedEvidenceJson>() {
                        let ev = data.inner();
                        log::debug!("Dropped data: {ev:?}");
                        sender.output(EvidenceFactoryOutput::InsertEvidenceAt(index.clone(), 0, ev)).unwrap();
                        return true;
                    }
                    false
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 8,
                set_hexpand: true,

                gtk::Separator {},

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,

                    gtk::Image {
                        set_icon_name: Some(relm4_icons::icon_names::CHEVRON_UP_DOWN_REGULAR),
                    },
                    gtk::Entry {
                        set_placeholder_text: Some(&lang::lookup("test-evidence-caption")),
                        set_hexpand: true,
                        set_text: &self.evidence.read().unwrap().caption().as_ref().unwrap_or(&String::new()),

                        connect_changed[sender] => move |entry| {
                            sender.input(EvidenceFactoryInput::SetCaption(entry.text().to_string()));
                        }
                    },
                    gtk::MenuButton {
                        set_tooltip: &lang::lookup("evidence-menu"),

                        #[wrap(Some)]
                        set_popover = &gtk::Popover {
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 4,

                                gtk::Button {
                                    set_label: &lang::lookup("evidence-move-up"),
                                    add_css_class: "flat",

                                    connect_clicked => EvidenceFactoryInput::MoveUp,
                                },
                                gtk::Button {
                                    set_label: &lang::lookup("evidence-move-down"),
                                    add_css_class: "flat",

                                    connect_clicked => EvidenceFactoryInput::MoveDown,
                                },
                                gtk::Button {
                                    set_label: &lang::lookup("evidence-delete"),
                                    add_css_class: "flat",
                                    add_css_class: "destructive-action",

                                    connect_clicked => EvidenceFactoryInput::Delete,
                                },
                            }
                        }
                    },
                },
                #[name = "evidence_child"]
                gtk::Box {},
            },
        }
    }

    fn init_model(init: Self::Init, index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let EvidenceFactoryInit { evidence, package } = init;
        Self {
            index: index.clone(),
            evidence: Arc::new(RwLock::new(evidence)),
            package,
        }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let ev = self.evidence.clone();
        let widgets = view_output!();
        self.index = index.clone();

        match self.evidence.read().unwrap().kind() {
            EvidenceKind::Text => {
                let scroll_window = gtk::ScrolledWindow::default();
                scroll_window.set_height_request(100);
                scroll_window.set_hexpand(true);

                let frame = gtk::Frame::new(None);

                let text_view = gtk::TextView::new();
                text_view.set_top_margin(8);
                text_view.set_bottom_margin(8);
                text_view.set_left_margin(8);
                text_view.set_right_margin(8);

                text_view.buffer().set_text(&self.get_data_as_string());
                let sender_c = sender.clone();
                text_view.buffer().connect_changed(move |buf| {
                    sender_c.input(EvidenceFactoryInput::TextSetText(
                        buf.text(&buf.start_iter(), &buf.end_iter(), false)
                            .to_string(),
                    ));
                });

                scroll_window.set_child(Some(&text_view));
                frame.set_child(Some(&scroll_window));
                widgets.evidence_child.append(&frame);
            }
            EvidenceKind::Image => {
                let img = gtk::Picture::new();
                img.set_paintable(self.get_data_as_texture().as_ref());
                img.set_hexpand(true);
                img.set_height_request(EVIDENCE_HEIGHT_REQUEST);
                widgets.evidence_child.append(&img);
            }
            EvidenceKind::Http => {
                let data = self.get_data_as_string();
                let data_parts = data
                    .split(HTTP_SEPARATOR)
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                let request = data_parts.first().cloned().unwrap_or_default();
                let response = data_parts.get(1).cloned().unwrap_or_default();

                let frame = gtk::Frame::default();
                frame.set_height_request(EVIDENCE_HEIGHT_REQUEST);
                let scrolled = gtk::ScrolledWindow::new();
                scrolled.set_hexpand(true);
                let txt_request = gtk::TextView::default();
                txt_request.add_css_class("monospace");
                txt_request.buffer().set_text(&request);
                txt_request.set_top_margin(8);
                txt_request.set_bottom_margin(8);
                txt_request.set_left_margin(8);
                txt_request.set_right_margin(8);
                txt_request.set_halign(gtk::Align::Fill);
                txt_request.set_valign(gtk::Align::Fill);
                let sender_c = sender.clone();
                txt_request.buffer().connect_changed(move |buf| {
                    sender_c.input(EvidenceFactoryInput::HttpSetRequest(
                        buf.text(&buf.start_iter(), &buf.end_iter(), false)
                            .to_string(),
                    ));
                });
                scrolled.set_child(Some(&txt_request));
                frame.set_child(Some(&scrolled));

                widgets.evidence_child.set_spacing(8);
                widgets.evidence_child.append(&frame);

                let frame = gtk::Frame::default();
                frame.set_height_request(EVIDENCE_HEIGHT_REQUEST);
                let scrolled = gtk::ScrolledWindow::new();
                scrolled.set_hexpand(true);
                let txt_response = gtk::TextView::default();
                txt_response.add_css_class("monospace");
                txt_response.buffer().set_text(&response);
                txt_response.set_top_margin(8);
                txt_response.set_bottom_margin(8);
                txt_response.set_left_margin(8);
                txt_response.set_right_margin(8);
                txt_response.set_halign(gtk::Align::Fill);
                txt_response.set_valign(gtk::Align::Fill);
                let sender_c = sender.clone();
                txt_response.buffer().connect_changed(move |buf| {
                    sender_c.input(EvidenceFactoryInput::HttpSetResponse(
                        buf.text(&buf.start_iter(), &buf.end_iter(), false)
                            .to_string(),
                    ));
                });
                scrolled.set_child(Some(&txt_response));
                frame.set_child(Some(&scrolled));
                widgets.evidence_child.append(&frame);
            }
            EvidenceKind::File => {
                let label = gtk::Label::default();
                if let Some(filename) = self.evidence.read().unwrap().original_filename() {
                    label.set_markup(&lang::lookup_with_args("test-evidence-file-named", {
                        let mut map = HashMap::new();
                        map.insert("filename", filename.into());
                        map
                    }));
                } else {
                    label.set_markup(&lang::lookup("test-evidence-file-unnamed"));
                }
                widgets.evidence_child.set_halign(gtk::Align::Center);
                widgets.evidence_child.append(&label);
            },
        };

        widgets
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            EvidenceFactoryInput::SetCaption(new_caption) => {
                self.evidence
                    .write()
                    .unwrap()
                    .caption_mut()
                    .replace(new_caption);
                sender
                    .output(EvidenceFactoryOutput::UpdateEvidence(
                        self.index.clone(),
                        self.evidence.read().unwrap().clone(),
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::TextSetText(new_text) => {
                if *self.evidence.read().unwrap().kind() != EvidenceKind::Text {
                    return;
                }
                match self.evidence.write().unwrap().value_mut() {
                    EvidenceData::Text { content } => {
                        *content = new_text;
                    }
                    EvidenceData::Base64 { data } => {
                        *data = new_text.into_bytes();
                    }
                    _ => panic!("cannot handle text of media type"),
                }
                sender
                    .output(EvidenceFactoryOutput::UpdateEvidence(
                        self.index.clone(),
                        self.evidence.read().unwrap().clone(),
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::HttpSetRequest(mut new_req) => {
                if *self.evidence.read().unwrap().kind() != EvidenceKind::Http {
                    return;
                }
                match self.evidence.write().unwrap().value_mut() {
                    EvidenceData::Text { content } => {
                        let data_parts = content
                            .split(HTTP_SEPARATOR)
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>();
                        let response = data_parts.get(1).cloned().unwrap_or_default();

                        new_req.push(HTTP_SEPARATOR);
                        new_req.push_str(&response);
                        *content = new_req;
                    }
                    EvidenceData::Base64 { data } => {
                        let data_parts = String::from_utf8_lossy(data)
                            .split(HTTP_SEPARATOR)
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>();
                        let response = data_parts.get(1).cloned().unwrap_or_default();

                        new_req.push(HTTP_SEPARATOR);
                        new_req.push_str(&response);
                        *data = new_req.into_bytes();
                    }
                    _ => panic!("cannot handle text of media type"),
                }
                sender
                    .output(EvidenceFactoryOutput::UpdateEvidence(
                        self.index.clone(),
                        self.evidence.read().unwrap().clone(),
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::HttpSetResponse(new_res) => {
                if *self.evidence.read().unwrap().kind() != EvidenceKind::Http {
                    return;
                }
                match self.evidence.write().unwrap().value_mut() {
                    EvidenceData::Text { content } => {
                        let data_parts = content
                            .split(HTTP_SEPARATOR)
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>();
                        let mut request = data_parts.first().cloned().unwrap_or_default();

                        request.push(HTTP_SEPARATOR);
                        request.push_str(&new_res);
                        *content = request;
                    }
                    EvidenceData::Base64 { data } => {
                        let data_parts = String::from_utf8_lossy(data)
                            .split(HTTP_SEPARATOR)
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>();
                        let mut request = data_parts.first().cloned().unwrap_or_default();

                        request.push(HTTP_SEPARATOR);
                        request.push_str(&new_res);
                        *data = request.into_bytes();
                    }
                    _ => panic!("cannot handle text of media type"),
                }
                sender
                    .output(EvidenceFactoryOutput::UpdateEvidence(
                        self.index.clone(),
                        self.evidence.read().unwrap().clone(),
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::Delete => {
                sender
                    .output(EvidenceFactoryOutput::DeleteEvidence(
                        self.index.clone(),
                        true,
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::MoveUp => {
                sender
                    .output(EvidenceFactoryOutput::InsertEvidenceAt(
                        self.index.clone(),
                        -1,
                        self.evidence.read().unwrap().clone(),
                    ))
                    .unwrap();
                sender
                    .output(EvidenceFactoryOutput::DeleteEvidence(
                        self.index.clone(),
                        false,
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::MoveDown => {
                sender
                    .output(EvidenceFactoryOutput::InsertEvidenceAt(
                        self.index.clone(),
                        2, // insert after self, which hasn't yet been deleted
                        self.evidence.read().unwrap().clone(),
                    ))
                    .unwrap();
                sender
                    .output(EvidenceFactoryOutput::DeleteEvidence(
                        self.index.clone(),
                        false,
                    ))
                    .unwrap();
            }
        }
    }
}
