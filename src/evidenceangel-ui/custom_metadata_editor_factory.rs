use adw::prelude::*;
use evidenceangel::CustomMetadataField;
use relm4::{
    Component, ComponentController, Controller, FactorySender, RelmWidgetExt, adw,
    factory::FactoryView,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
};

use crate::{
    dialogs::custom_metadata_field::{
        CustomMetadataDialogInput, CustomMetadataDialogModel, CustomMetadataDialogOutput,
    },
    lang,
};

pub struct CustomMetadataEditorFactoryInit {
    pub root: adw::ApplicationWindow,
    pub key: String,
    pub field: CustomMetadataField,
}

pub struct CustomMetadataEditorFactoryModel {
    root: adw::ApplicationWindow,
    index: DynamicIndex,
    key: String,
    name: String,
    description: String,
    primary: bool,
    latest_new_custom_metadata_dlg: Option<Controller<CustomMetadataDialogModel>>,
}

#[derive(Debug, Clone)]
pub enum CustomMetadataEditorFactoryInput {
    UpdatePrimary(bool),
    UpdateCustomField { name: String, description: String },
    MakeSelfPrimary,
    EditSelf,
    DeleteSelf,
}

#[derive(Debug)]
pub enum CustomMetadataEditorFactoryOutput {
    UpdateCustomField {
        key: String,
        name: String,
        description: String,
    },
    DeleteCustomField {
        index: DynamicIndex,
        key: String,
    },
    MakeFieldPrimary {
        /// None to unset primary
        index: Option<DynamicIndex>,
        /// None to unset primary
        key: Option<String>,
    },
}

#[relm4::factory(pub)]
impl FactoryComponent for CustomMetadataEditorFactoryModel {
    type ParentWidget = adw::PreferencesGroup;
    type Input = CustomMetadataEditorFactoryInput;
    type Output = CustomMetadataEditorFactoryOutput;
    type Init = CustomMetadataEditorFactoryInit;
    type CommandOutput = ();

    view! {
        #[root]
        adw::ActionRow {
            #[watch]
            set_title: &self.name,
            #[watch]
            set_subtitle: &self.description,
            set_use_markup: false,

            #[name = "box_widget"]
            add_suffix = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 4,

                gtk::Button {
                    set_icon_name: relm4_icons::icon_names::DOCUMENT_EDIT,
                    set_tooltip: &lang::lookup("custom-metadata-edit"),
                    add_css_class: "flat",

                    connect_clicked[sender] => move |_| {
                        sender.input(CustomMetadataEditorFactoryInput::EditSelf);
                    }
                },

                gtk::Button {
                    #[watch]
                    set_icon_name: if self.primary {
                        relm4_icons::icon_names::CHECKMARK_STARBURST_FILLED
                    } else {
                        relm4_icons::icon_names::CHECKMARK_STARBURST_REGULAR
                    },
                    set_tooltip: &lang::lookup("custom-metadata-promote"),
                    add_css_class: "flat",

                    connect_clicked[sender] => move |_| {
                        sender.input(CustomMetadataEditorFactoryInput::MakeSelfPrimary);
                    }
                },

                gtk::Button {
                    set_icon_name: relm4_icons::icon_names::CROSS_LARGE,
                    set_tooltip: &lang::lookup("custom-metadata-remove"),
                    add_css_class: "flat",

                    connect_clicked[sender] => move |_| {
                        sender.input(CustomMetadataEditorFactoryInput::DeleteSelf);
                    }
                },
            },
        }
    }

    fn init_model(init: Self::Init, index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let Self::Init { root, key, field } = init;
        Self {
            root,
            latest_new_custom_metadata_dlg: None,
            index: index.clone(),
            key,
            name: field.name().clone(),
            description: field.description().clone(),
            primary: *field.primary(),
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();
        widgets
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            CustomMetadataEditorFactoryInput::UpdatePrimary(new_primary) => {
                self.primary = new_primary;
            }
            CustomMetadataEditorFactoryInput::EditSelf => {
                let new_custom_metadata_dlg = CustomMetadataDialogModel::builder()
                    .launch(())
                    .forward(sender.input_sender(), move |msg| match msg {
                        CustomMetadataDialogOutput::SaveField {
                            name, description, ..
                        } => CustomMetadataEditorFactoryInput::UpdateCustomField {
                            name,
                            description,
                        },
                    });
                new_custom_metadata_dlg.emit(CustomMetadataDialogInput::Editing {
                    name: self.name.clone(),
                    description: self.description.clone(),
                });
                new_custom_metadata_dlg.emit(CustomMetadataDialogInput::Present(
                    self.root.clone(),
                    Some(self.key.clone()),
                ));
                self.latest_new_custom_metadata_dlg = Some(new_custom_metadata_dlg);
            }
            CustomMetadataEditorFactoryInput::MakeSelfPrimary => {
                let _ = sender.output(CustomMetadataEditorFactoryOutput::MakeFieldPrimary {
                    index: if self.primary {
                        None
                    } else {
                        Some(self.index.clone())
                    },
                    key: if self.primary {
                        None
                    } else {
                        Some(self.key.clone())
                    },
                });
            }
            CustomMetadataEditorFactoryInput::DeleteSelf => {
                let _ = sender.output(CustomMetadataEditorFactoryOutput::DeleteCustomField {
                    index: self.index.clone(),
                    key: self.key.clone(),
                });
            }
            CustomMetadataEditorFactoryInput::UpdateCustomField { name, description } => {
                self.name.clone_from(&name);
                self.description.clone_from(&description);
                let _ = sender.output(CustomMetadataEditorFactoryOutput::UpdateCustomField {
                    key: self.key.clone(),
                    name,
                    description,
                });
            }
        }
    }
}
