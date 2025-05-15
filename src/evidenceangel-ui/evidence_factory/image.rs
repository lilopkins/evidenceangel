use std::fs;

use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};
use tempfile::NamedTempFile;

use crate::lang;

pub struct ComponentModel {
    data: Vec<u8>,
    texture: Option<gtk::gdk::Texture>,
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
    Enlarge,
}

#[derive(Debug)]
pub enum ComponentOutput {}

pub struct ComponentInit {
    pub data: Vec<u8>,
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
        let ComponentInit { data } = init;
        let glib_bytes = gtk::glib::Bytes::from_owned(data.clone());
        let texture = gtk::gdk::Texture::from_bytes(&glib_bytes).ok();
        let model = ComponentModel {
            data,
            texture,
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
                // Create temporary file for image
                let maybe_extension = infer::get(&self.data).map(|ty| ty.extension());
                let maybe_tempfile = if let Some(ext) = maybe_extension {
                    NamedTempFile::with_suffix(format!(".{ext}"))
                } else {
                    NamedTempFile::new()
                };
                tracing::debug!("Temp file: {maybe_tempfile:?}");
                if let Ok(file) = maybe_tempfile {
                    if let Err(e) = fs::write(&file, self.data.clone()) {
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
