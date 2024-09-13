use gtk::prelude::*;
use relm4::{
    factory::FactoryView,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    FactorySender,
};
use uuid::Uuid;

use crate::lang;

pub struct NavFactoryModel {
    selected: bool,
    name: String,
    id: Uuid,
}

#[derive(Clone, Debug)]
pub enum NavFactoryInput {
    ShowAsSelected(bool),
    UpdateTitle(String),
}

#[derive(Debug)]
pub enum NavFactoryOutput {
    NavigateTo(usize, Uuid),
    DeleteCase(usize, Uuid),
}

pub struct NavFactoryInit {
    pub id: Uuid,
    pub name: String,
}

#[relm4::factory(pub)]
impl FactoryComponent for NavFactoryModel {
    type ParentWidget = gtk::Box;
    type Input = NavFactoryInput;
    type Output = NavFactoryOutput;
    type Init = NavFactoryInit;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            gtk::Button {
                #[watch]
                set_label: &self.name,
                add_css_class: "flat",
                set_hexpand: true,
                #[watch]
                set_has_frame: self.selected,

                connect_clicked[sender, index, id] => move |_| {
                    let _ = sender.output(NavFactoryOutput::NavigateTo(index.current_index(), id));
                },
            },

            gtk::Button {
                set_icon_name: relm4_icons::icon_names::CROSS_LARGE,
                set_tooltip_text: Some(&lang::lookup("nav-delete-case")),
                add_css_class: "flat",

                connect_clicked[sender, index, id] => move |_| {
                    let _ = sender.output(NavFactoryOutput::DeleteCase(index.current_index(), id));
                },
            },
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            selected: false,
            name: init.name,
            id: init.id,
        }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let id = self.id;
        let widgets = view_output!();
        widgets
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            NavFactoryInput::ShowAsSelected(sel) => {
                self.selected = sel;
            }
            NavFactoryInput::UpdateTitle(new_title) => {
                self.name = new_title;
            }
        }
    }
}
