use std::sync::{Arc, RwLock};

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
    /// Set the text for a text evidence object. If not text evidence, panic.
    TextSetText(String),
    /// Set the caption for this evidence.
    SetCaption(String),
    Delete,
}

#[derive(Debug)]
pub enum EvidenceFactoryOutput {
    /// Replace the evidence at the given position. This MUST NOT trigger an update to the interface.
    UpdateEvidence(DynamicIndex, Evidence),
    /// Delete evidence at the given position. This MUST trigger an update to the interface.
    DeleteEvidence(DynamicIndex),
    /// `InsertEvidenceBefore` MUST be followed by a `DeleteEvidence` call as it is only triggered by a data move.
    /// As such, it MUST NOT trigger an update to the interface.
    InsertEvidenceBefore(DynamicIndex, Evidence),
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
                        sender.output(EvidenceFactoryOutput::DeleteEvidence(index.clone())).unwrap();
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
                        sender.output(EvidenceFactoryOutput::InsertEvidenceBefore(index.clone(), ev)).unwrap();
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

                                    // TODO connect_clicked => EvidenceFactoryInput::MoveUp,
                                },
                                gtk::Button {
                                    set_label: &lang::lookup("evidence-move-down"),
                                    add_css_class: "flat",

                                    // TODO connect_clicked => EvidenceFactoryInput::MoveDown,
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

                let text_view = gtk::TextView::new();
                text_view.set_top_margin(4);
                text_view.set_bottom_margin(4);
                text_view.set_left_margin(4);
                text_view.set_right_margin(4);

                text_view.buffer().set_text(&self.get_data_as_string());
                let sender_c = sender.clone();
                text_view.buffer().connect_changed(move |buf| {
                    sender_c.input(EvidenceFactoryInput::TextSetText(
                        buf.text(&buf.start_iter(), &buf.end_iter(), false)
                            .to_string(),
                    ));
                });

                scroll_window.set_child(Some(&text_view));
                widgets.evidence_child.append(&scroll_window);
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
                    .split("\x1e")
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                let request = data_parts.first().cloned().unwrap_or_default();
                let response = data_parts.get(1).cloned().unwrap_or_default();

                let frame = gtk::Frame::default();
                frame.set_height_request(EVIDENCE_HEIGHT_REQUEST);
                let scrolled = gtk::ScrolledWindow::new();
                scrolled.set_hexpand(true);
                let label = gtk::Label::default();
                label.set_markup(&format!(
                    "<tt>{}</tt>",
                    request
                        .replace("&", "&amp;")
                        .replace("<", "&lt;")
                        .replace(">", "&gt;")
                ));
                label.set_margin_all(8);
                label.set_selectable(true);
                label.set_halign(gtk::Align::Start);
                label.set_valign(gtk::Align::Start);
                scrolled.set_child(Some(&label));
                frame.set_child(Some(&scrolled));

                widgets.evidence_child.set_spacing(8);
                widgets.evidence_child.append(&frame);

                let frame = gtk::Frame::default();
                frame.set_height_request(EVIDENCE_HEIGHT_REQUEST);
                let scrolled = gtk::ScrolledWindow::new();
                scrolled.set_hexpand(true);
                let label = gtk::Label::default();
                label.set_markup(&format!(
                    "<tt>{}</tt>",
                    response
                        .replace("&", "&amp;")
                        .replace("<", "&lt;")
                        .replace(">", "&gt;")
                ));
                label.set_margin_all(8);
                label.set_selectable(true);
                label.set_halign(gtk::Align::Start);
                label.set_valign(gtk::Align::Start);
                scrolled.set_child(Some(&label));
                frame.set_child(Some(&scrolled));
                widgets.evidence_child.append(&frame);
            }
            EvidenceKind::File => (),
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
                if let EvidenceData::Text { content } = self.evidence.write().unwrap().value_mut() {
                    *content = new_text;
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
                    .output(EvidenceFactoryOutput::DeleteEvidence(self.index.clone()))
                    .unwrap();
            }
        }
    }
}
