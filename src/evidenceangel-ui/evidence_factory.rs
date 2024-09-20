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
pub enum EvidenceFactoryInput {}

#[derive(Debug)]
pub enum EvidenceFactoryOutput {}

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

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let EvidenceFactoryInit { evidence, package } = init;
        Self { evidence, package }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        _sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        let main_widget = match self.evidence.kind() {
            EvidenceKind::Text => {
                let ui_box = gtk::Box::default();
                let label = gtk::Label::new(Some(&self.get_data_as_string()));
                label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                label.set_selectable(true);
                ui_box.append(&label);
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

        // Append caption (if set)
        if let Some(caption) = self.evidence.caption() {
            let caption_lbl = gtk::Label::default();
            caption_lbl.set_text(caption);
            caption_lbl.set_selectable(true);
            box_widget.append(&caption_lbl);
        }
        
        box_widget.append(&main_widget);

        // Append separator
        box_widget.append(&gtk::Separator::default());

        root.append(&box_widget);

        widgets
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {}
    }
}
