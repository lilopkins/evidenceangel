use adw::prelude::*;
use evidenceangel::CustomMetadataField;
use relm4::{
    FactorySender, adw,
    factory::FactoryView,
    prelude::{DynamicIndex, FactoryComponent},
};

pub struct CustomMetadataFactoryInit {
    pub key: String,
    pub field: CustomMetadataField,
    pub value: String,
}

pub struct CustomMetadataFactoryModel {
    key: String,
    field: CustomMetadataField,
    init_value: String,
}

#[derive(Debug)]
pub enum CustomMetadataFactoryInput {
    ValueChanged(String),
}

#[derive(Debug)]
pub enum CustomMetadataFactoryOutput {
    ValueChanged { key: String, new_value: String },
}

#[relm4::factory(pub)]
impl FactoryComponent for CustomMetadataFactoryModel {
    type ParentWidget = adw::PreferencesGroup;
    type Input = CustomMetadataFactoryInput;
    type Output = CustomMetadataFactoryOutput;
    type Init = CustomMetadataFactoryInit;
    type CommandOutput = ();

    view! {
        #[root]
        adw::EntryRow {
            set_title: self.field.name(),
            // set_subtitle: self.field.description(),
            set_use_markup: false,
            set_text: &self.init_value,

            connect_changed[sender] => move |entry| {
                sender.input(CustomMetadataFactoryInput::ValueChanged(entry.text().to_string()));
            },
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let Self::Init { key, field, value } = init;
        Self {
            key,
            field,
            init_value: value,
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
            CustomMetadataFactoryInput::ValueChanged(new_value) => {
                sender
                    .output(CustomMetadataFactoryOutput::ValueChanged {
                        key: self.key.clone(),
                        new_value,
                    })
                    .unwrap();
            }
        }
    }
}
