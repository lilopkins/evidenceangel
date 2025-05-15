use std::{fs, sync::Arc};

use evidenceangel::{Evidence, EvidencePackage};
use gtk::prelude::*;
use parking_lot::RwLock;
use relm4::{
    Component, ComponentParts, ComponentSender,
    gtk::{self, prelude::OrientableExt},
};
use tempfile::NamedTempFile;

use crate::{lang, lang_args};

pub struct ComponentModel {
    package: Arc<RwLock<EvidencePackage>>,
    evidence: Evidence,
    temp_files: Vec<NamedTempFile>,
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
    Preview,
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

            gtk::Label {
                set_ellipsize: gtk::pango::EllipsizeMode::Middle,
                set_markup: &if let Some(filename) = model.evidence.original_filename() {
                    lang::lookup_with_args(
                        "test-evidence-file-named",
                        &lang_args!("filename", filename),
                    )
                } else {
                    lang::lookup("test-evidence-file-unnamed")
                },
            },
            gtk::Button {
                set_label: &lang::lookup("expand-file"),
                add_css_class: "flat",
                set_halign: gtk::Align::Center,

                connect_clicked => ComponentInput::Internal(ComponentInputInternal::Preview),
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let ComponentInit {
            package, evidence, ..
        } = init;
        let model = ComponentModel {
            evidence,
            package,
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
            ComponentInput::Internal(ComponentInputInternal::Preview) => {
                let mut pkg = self.package.write();
                // Create temporary file
                let maybe_extension =
                    infer::get(&self.evidence.data(&mut pkg)).map(|ty| ty.extension());
                let maybe_tempfile = if let Some(ext) = maybe_extension {
                    NamedTempFile::with_suffix(format!(".{ext}"))
                } else {
                    NamedTempFile::new()
                };
                tracing::debug!("Temp file: {maybe_tempfile:?}");
                if let Ok(file) = maybe_tempfile {
                    if let Err(e) = fs::write(&file, self.evidence.data(&mut pkg).clone()) {
                        tracing::error!("Failed to write image data to temp file! ({e})");
                        return;
                    }

                    // Trigger OS to open
                    open::that_in_background(file.path());

                    self.temp_files.push(file);
                }
            }
        }
    }
}
