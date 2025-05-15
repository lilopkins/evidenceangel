use std::{any::Any, sync::Arc};

use adw::prelude::*;
use evidenceangel::{Evidence, EvidenceData, EvidenceKind, EvidencePackage};
#[allow(unused_imports)]
use gtk::prelude::*;
use parking_lot::RwLock;
use relm4::{
    Component, ComponentController, FactorySender, RelmWidgetExt, adw,
    factory::FactoryView,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
};

use crate::lang;
use crate::util::BoxedEvidenceJson;

mod file;
mod http;
mod image;
mod rich_text;
mod text;

const EVIDENCE_HEIGHT_REQUEST: i32 = 300;
const HTTP_SEPARATOR: char = '\x1e';

pub struct EvidenceFactoryModel {
    sub_component: Box<dyn Any>,
    index: DynamicIndex,
    evidence: Arc<RwLock<Evidence>>,
    package: Arc<RwLock<EvidencePackage>>,
}

impl EvidenceFactoryModel {
    fn get_data(&self) -> Vec<u8> {
        tracing::debug!("Got some {:?} data", self.evidence.read().kind());
        let mut pkg = self.package.write();
        self.evidence.read().data(&mut pkg)
    }

    fn get_data_as_string(&self) -> String {
        tracing::debug!("Converting media to string...");
        String::from_utf8(self.get_data()).unwrap_or(lang::lookup("invalid-data"))
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
                    let dnd_data = BoxedEvidenceJson::new((*ev.read()).clone());
                    tracing::debug!("Drag data started: {dnd_data:?}");
                    Some(gtk::gdk::ContentProvider::for_value(&dnd_data.to_value()))
                },

                connect_drag_end[sender, index] => move |_slf, _drag, delete_data| {
                    if delete_data {
                        tracing::debug!("Deleting drag start item");
                        sender.output(EvidenceFactoryOutput::DeleteEvidence(index.clone(), false)).unwrap();
                    }
                }
            },
            add_controller = gtk::DropTarget {
                set_actions: gtk::gdk::DragAction::MOVE,
                set_types: &[BoxedEvidenceJson::static_type()],

                connect_drop[sender, index] => move |_slf, val, _x, _y| {
                    tracing::debug!("Dropped type: {:?}", val.type_());
                    if let Ok(data) = val.get::<BoxedEvidenceJson>() {
                        let ev = data.inner();
                        tracing::debug!("Dropped data: {ev:?}");
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
                        set_text: &self.evidence.read().caption().as_ref().unwrap_or(&String::new()),

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
                adw::Bin {},
            },
        }
    }

    fn init_model(init: Self::Init, index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let EvidenceFactoryInit { evidence, package } = init;
        Self {
            sub_component: Box::new(()),
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

        match self.evidence.read().kind() {
            EvidenceKind::Text => {
                let component = text::ComponentModel::builder()
                    .launch(text::ComponentInit {
                        text: self.get_data_as_string(),
                    })
                    .forward(sender.input_sender(), |msg| match msg {
                        text::ComponentOutput::TextChanged { new_text } => {
                            EvidenceFactoryInput::TextSetText(new_text)
                        }
                    });
                widgets.evidence_child.set_child(Some(component.widget()));
                self.sub_component = Box::new(component);
            }
            EvidenceKind::RichText => {
                let component = rich_text::ComponentModel::builder()
                    .launch(rich_text::ComponentInit {
                        text: self.get_data_as_string(),
                    })
                    .forward(sender.input_sender(), |msg| match msg {
                        rich_text::ComponentOutput::TextChanged { new_text } => {
                            EvidenceFactoryInput::TextSetText(new_text)
                        }
                    });
                widgets.evidence_child.set_child(Some(component.widget()));
                self.sub_component = Box::new(component);
            }
            EvidenceKind::Image => {
                let component = image::ComponentModel::builder()
                    .launch(image::ComponentInit {
                        data: self.get_data(),
                    })
                    .forward(sender.input_sender(), |msg| match msg {});
                widgets.evidence_child.set_child(Some(component.widget()));
                self.sub_component = Box::new(component);
            }
            EvidenceKind::Http => {
                let data = self.get_data_as_string();
                let data_parts = data
                    .split(HTTP_SEPARATOR)
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                let request = data_parts.first().cloned().unwrap_or_default();
                let response = data_parts.get(1).cloned().unwrap_or_default();

                let component = http::ComponentModel::builder()
                    .launch(http::ComponentInit { request, response })
                    .forward(sender.input_sender(), |msg| match msg {
                        http::ComponentOutput::RequestChanged { new_text } => {
                            EvidenceFactoryInput::HttpSetRequest(new_text)
                        }
                        http::ComponentOutput::ResponseChanged { new_text } => {
                            EvidenceFactoryInput::HttpSetResponse(new_text)
                        }
                    });
                widgets.evidence_child.set_child(Some(component.widget()));
                self.sub_component = Box::new(component);
            }
            EvidenceKind::File => {
                let component = file::ComponentModel::builder()
                    .launch(file::ComponentInit {
                        evidence: self.evidence.read().clone(),
                        package: self.package.clone(),
                    })
                    .forward(sender.input_sender(), |msg| match msg {});
                widgets.evidence_child.set_halign(gtk::Align::Center);
                widgets.evidence_child.set_child(Some(component.widget()));
                self.sub_component = Box::new(component);
            }
        };

        widgets
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            EvidenceFactoryInput::SetCaption(new_caption) => {
                self.evidence.write().caption_mut().replace(new_caption);
                sender
                    .output(EvidenceFactoryOutput::UpdateEvidence(
                        self.index.clone(),
                        self.evidence.read().clone(),
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::TextSetText(new_text) => {
                if ![EvidenceKind::Text, EvidenceKind::RichText]
                    .contains(self.evidence.read().kind())
                {
                    return;
                }
                match self.evidence.write().value_mut() {
                    EvidenceData::Text { content } => {
                        *content = new_text;
                    }
                    EvidenceData::Base64 { data } => {
                        *data = new_text.into_bytes();
                    }
                    EvidenceData::Media { .. } => panic!("cannot handle text of media type"),
                }
                sender
                    .output(EvidenceFactoryOutput::UpdateEvidence(
                        self.index.clone(),
                        self.evidence.read().clone(),
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::HttpSetRequest(mut new_req) => {
                if *self.evidence.read().kind() != EvidenceKind::Http {
                    return;
                }
                match self.evidence.write().value_mut() {
                    EvidenceData::Text { content } => {
                        let data_parts = content
                            .split(HTTP_SEPARATOR)
                            .map(ToString::to_string)
                            .collect::<Vec<_>>();
                        let response = data_parts.get(1).cloned().unwrap_or_default();

                        new_req.push(HTTP_SEPARATOR);
                        new_req.push_str(&response);
                        *content = new_req;
                    }
                    EvidenceData::Base64 { data } => {
                        let data_parts = String::from_utf8_lossy(data)
                            .split(HTTP_SEPARATOR)
                            .map(ToString::to_string)
                            .collect::<Vec<_>>();
                        let response = data_parts.get(1).cloned().unwrap_or_default();

                        new_req.push(HTTP_SEPARATOR);
                        new_req.push_str(&response);
                        *data = new_req.into_bytes();
                    }
                    EvidenceData::Media { .. } => panic!("cannot handle text of media type"),
                }
                sender
                    .output(EvidenceFactoryOutput::UpdateEvidence(
                        self.index.clone(),
                        self.evidence.read().clone(),
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::HttpSetResponse(new_res) => {
                if *self.evidence.read().kind() != EvidenceKind::Http {
                    return;
                }
                match self.evidence.write().value_mut() {
                    EvidenceData::Text { content } => {
                        let data_parts = content
                            .split(HTTP_SEPARATOR)
                            .map(ToString::to_string)
                            .collect::<Vec<_>>();
                        let mut request = data_parts.first().cloned().unwrap_or_default();

                        request.push(HTTP_SEPARATOR);
                        request.push_str(&new_res);
                        *content = request;
                    }
                    EvidenceData::Base64 { data } => {
                        let data_parts = String::from_utf8_lossy(data)
                            .split(HTTP_SEPARATOR)
                            .map(ToString::to_string)
                            .collect::<Vec<_>>();
                        let mut request = data_parts.first().cloned().unwrap_or_default();

                        request.push(HTTP_SEPARATOR);
                        request.push_str(&new_res);
                        *data = request.into_bytes();
                    }
                    EvidenceData::Media { .. } => panic!("cannot handle text of media type"),
                }
                sender
                    .output(EvidenceFactoryOutput::UpdateEvidence(
                        self.index.clone(),
                        self.evidence.read().clone(),
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
                        self.evidence.read().clone(),
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
                        self.evidence.read().clone(),
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
