use adw::prelude::*;
use relm4::{
    Component, ComponentParts, ComponentSender, RelmWidgetExt,
    adw::{self, ApplicationWindow},
    gtk,
};

use crate::lang;

pub struct CustomMetadataDialogModel {}

#[derive(Debug)]
pub enum CustomMetadataDialogInput {
    Editing { name: String, description: String },
    Present(ApplicationWindow, Option<String>),
    _Save,
}

#[derive(Debug)]
pub enum CustomMetadataDialogOutput {
    SaveField {
        key: Option<String>,
        name: String,
        description: String,
    },
}

#[relm4::component(pub)]
impl Component for CustomMetadataDialogModel {
    type Input = CustomMetadataDialogInput;
    type Output = CustomMetadataDialogOutput;
    type CommandOutput = ();
    type Init = ();

    view! {
        #[root]
        adw::Dialog {
            set_width_request: 400,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: &lang::lookup("metadata-edit-title"),
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,
                    set_margin_all: 16,

                    adw::PreferencesGroup {
                        #[name = "id"]
                        adw::EntryRow {
                            set_title: &lang::lookup("metadata-edit-id"),
                        },

                        #[name = "name"]
                        adw::EntryRow {
                            set_title: &lang::lookup("metadata-edit-name"),
                            connect_entry_activated => CustomMetadataDialogInput::_Save,
                        },

                        #[name = "description"]
                        adw::EntryRow {
                            set_title: &lang::lookup("metadata-edit-description"),
                            connect_entry_activated => CustomMetadataDialogInput::_Save,
                        }
                    },

                    gtk::Button {
                        set_label: &lang::lookup("metadata-edit-submit"),
                        add_css_class: "pill",
                        add_css_class: "suggested-action",
                        set_halign: gtk::Align::Center,

                        connect_clicked => CustomMetadataDialogInput::_Save,
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = CustomMetadataDialogModel {};
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
            CustomMetadataDialogInput::Editing { name, description } => {
                widgets.name.set_text(&name);
                widgets.description.set_text(&description);
            }
            CustomMetadataDialogInput::Present(window, id) => {
                if let Some(id) = id {
                    widgets.id.set_text(&id);
                    widgets.id.set_visible(false);
                }
                root.present(Some(&window));
            }
            CustomMetadataDialogInput::_Save => {
                let id = widgets.id.text().to_string();
                let name = widgets.name.text().to_string();
                let description = widgets.description.text().to_string();
                if !name.trim().is_empty() {
                    let key = if id.trim().is_empty() {
                        Some(name.to_ascii_lowercase().replace(' ', "_"))
                    } else {
                        Some(id)
                    };
                    sender
                        .output(CustomMetadataDialogOutput::SaveField {
                            key,
                            name,
                            description,
                        })
                        .unwrap();
                }
                root.close();
            }
        }
        self.update_view(widgets, sender);
    }
}
