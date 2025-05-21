use std::{fs, sync::Arc};

use evidenceangel::{Evidence, EvidencePackage};
use gtk::prelude::*;
use parking_lot::RwLock;
use relm4::{Component, ComponentParts, ComponentSender, gtk};
use tempfile::TempDir;

use crate::lang;

pub struct ComponentModel {
    texture: Option<gtk::gdk::Texture>,
    package: Arc<RwLock<EvidencePackage>>,
    evidence: Evidence,
    temp_files: Vec<TempDir>,
}

#[derive(Debug)]
pub enum ComponentInput {
    /// An internal message was triggered
    #[allow(
        private_interfaces,
        reason = "These messages should only be produced by this component."
    )]
    Internal(ComponentInputInternal),
}

#[derive(Debug)]
enum ComponentInputInternal {
    Enlarge,
}

#[derive(Debug)]
pub enum ComponentOutput {}

pub struct ComponentInit {
    pub evidence: Evidence,
    pub package: Arc<RwLock<EvidencePackage>>,
}

#[relm4::component(pub)]
impl Component for ComponentModel {
    type CommandOutput = ();
    type Input = ComponentInput;
    type Output = ComponentOutput;
    type Init = ComponentInit;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 4,

            gtk::Picture {
                set_paintable: model.texture.as_ref(),
                set_hexpand: true,
                set_height_request: super::EVIDENCE_HEIGHT_REQUEST,
            },
            gtk::Button {
                set_label: &lang::lookup("expand-image"),
                add_css_class: "flat",
                set_halign: gtk::Align::Center,

                connect_clicked => ComponentInput::Internal(ComponentInputInternal::Enlarge),
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let ComponentInit {
            evidence, package, ..
        } = init;
        let glib_bytes = gtk::glib::Bytes::from_owned(evidence.data(&mut package.write()));
        let texture = gtk::gdk::Texture::from_bytes(&glib_bytes).ok();
        let model = ComponentModel {
            texture,
            package,
            evidence,
            temp_files: vec![],
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ComponentInput::Internal(ComponentInputInternal::Enlarge) => {
                let data = self.evidence.data(&mut self.package.write());

                // Create temporary directory
                if let Ok(target_dir) = tempfile::tempdir() {
                    let target_file = target_dir.path().join(
                        self.evidence
                            .original_filename()
                            .clone()
                            .unwrap_or_else(|| {
                                let maybe_extension = infer::get(&data).map(|ty| ty.extension());
                                format!(
                                    "image{}{}",
                                    if maybe_extension.is_some() { "." } else { "" },
                                    maybe_extension.unwrap_or_default()
                                )
                            }),
                    );

                    if let Err(e) = fs::write(&target_file, data) {
                        tracing::error!("Failed to write image data to temp file! ({e})");
                        return;
                    }

                    // Trigger OS to open
                    // SAFETY: file MUST be present in a directory
                    open::that_in_background(target_dir.path());

                    self.temp_files.push(target_dir);
                }
            }
        }
    }
}
