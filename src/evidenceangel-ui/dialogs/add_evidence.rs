use std::sync::{Arc, RwLock};

use adw::prelude::*;
use evidenceangel::{Evidence, EvidenceData, EvidenceKind, EvidencePackage};
use relm4::{
    adw::{self, ApplicationWindow},
    gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt,
};

use crate::lang;

#[derive(Debug)]
pub enum AddEvidenceInput {
    Present(ApplicationWindow),
    _AddEvidence,
}

#[derive(Debug)]
pub enum AddEvidenceOutput {
    AddEvidence(Evidence),
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
        }
        self.update_view(widgets, sender)
    }
}

pub struct AddImageEvidenceDialogModel {}

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

                adw::PreferencesGroup {
                    set_title: &lang::lookup("add-evidence-title"),
                    set_margin_all: 16,

                    adw::ActionRow {
                        set_title: &lang::lookup("add-evidence-image-label"),
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
        let model = AddImageEvidenceDialogModel {};
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
                // TODO Add media to package
                // TODO Return media hash
                //let content = widgets.text_entry.text().to_string();
                //let ev = Evidence::new(EvidenceKind::Image, EvidenceData::Text { content });
                //let _ = sender.output(AddEvidenceOutput::AddEvidence(ev));
                root.close();
            }
        }
        self.update_view(widgets, sender)
    }
}
