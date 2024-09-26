use std::sync::{Arc, RwLock};

use evidenceangel::{Evidence, EvidenceData, EvidenceKind, EvidencePackage};
use gtk::prelude::*;
use relm4::{
    factory::FactoryView,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    FactorySender, RelmWidgetExt,
};

use crate::lang;

const EVIDENCE_HEIGHT_REQUEST: i32 = 300;

pub struct EvidenceFactoryModel {
    index: DynamicIndex,
    evidence: Evidence,
    package: Arc<RwLock<EvidencePackage>>,
}

impl EvidenceFactoryModel {
    fn get_data(&self) -> Vec<u8> {
        log::debug!("Got some {:?} data", self.evidence.kind());
        match self.evidence.value() {
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
    UpdateEvidence(DynamicIndex, Evidence),
    DeleteEvidence(DynamicIndex),
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
        gtk::Box {}
    }

    fn init_model(init: Self::Init, index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let EvidenceFactoryInit { evidence, package } = init;
        Self { index: index.clone(), evidence, package }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();
        self.index = index.clone();

        let main_widget = match self.evidence.kind() {
            EvidenceKind::Text => {
                let ui_box = gtk::Box::default();
                let ml = gtk::TextView::new();
                ml.buffer().set_text(&self.get_data_as_string());
                let sender_c = sender.clone();
                ml.buffer().connect_changed(move |buf| {
                    sender_c.input(EvidenceFactoryInput::TextSetText(buf.text(&buf.start_iter(), &buf.end_iter(), false).to_string()));
                });
                ml.set_hexpand(true);
                ui_box.append(&ml);
                ui_box
            }
            EvidenceKind::Image => {
                let ui_box = gtk::Box::default();

                let img = gtk::Picture::new();
                img.set_paintable(self.get_data_as_texture().as_ref());
                img.set_hexpand(true);
                img.set_height_request(EVIDENCE_HEIGHT_REQUEST);
                ui_box.append(&img);

                ui_box
            }
            EvidenceKind::Http => {
                let ui_box = gtk::Box::default();
                ui_box.set_spacing(8);

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
                ui_box.append(&frame);

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
                ui_box.append(&frame);

                ui_box
            }
            EvidenceKind::File => gtk::Box::default(),
        };

        let box_widget = gtk::Box::new(gtk::Orientation::Vertical, 8);
        box_widget.set_hexpand(true);

        // Append separator
        box_widget.append(&gtk::Separator::default());

        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        // Append caption
        let caption_txt = gtk::Entry::new();
        caption_txt.set_placeholder_text(Some(&lang::lookup("test-evidence-caption")));
        caption_txt.set_hexpand(true);
        if let Some(caption) = self.evidence.caption() {
            caption_txt.set_text(caption);
        }
        let sender_c = sender.clone();
        caption_txt.connect_changed(move |entry| {
            sender_c.input(EvidenceFactoryInput::SetCaption(entry.text().to_string()));
        });
        row_box.append(&caption_txt);

        // Append delete button
        let btn_delete = gtk::Button::new();
        btn_delete.set_icon_name(relm4_icons::icon_names::DELETE_FILLED);
        btn_delete.set_tooltip(&lang::lookup("evidence-delete"));
        btn_delete.add_css_class("circle");
        btn_delete.add_css_class("destructive-action");
        let sender_c = sender.clone();
        btn_delete.connect_clicked(move |_| {
            sender_c.input(EvidenceFactoryInput::Delete);
        });
        row_box.append(&btn_delete);

        box_widget.append(&row_box);
        box_widget.append(&main_widget);

        root.append(&box_widget);

        widgets
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            EvidenceFactoryInput::SetCaption(new_caption) => {
                self.evidence.caption_mut().replace(new_caption);
                sender.output(EvidenceFactoryOutput::UpdateEvidence(self.index.clone(), self.evidence.clone())).unwrap();
            }
            EvidenceFactoryInput::TextSetText(new_text) => {
                if let EvidenceData::Text { content } = self.evidence.value_mut() {
                    *content = new_text;
                }
                sender.output(EvidenceFactoryOutput::UpdateEvidence(self.index.clone(), self.evidence.clone())).unwrap();
            }
            EvidenceFactoryInput::Delete => {
                sender.output(EvidenceFactoryOutput::DeleteEvidence(self.index.clone())).unwrap();
            }
        }
    }
}
