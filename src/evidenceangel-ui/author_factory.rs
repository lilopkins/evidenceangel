use adw::prelude::*;
use evidenceangel::Author;
use relm4::{
    adw,
    factory::FactoryView,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    FactorySender, RelmWidgetExt,
};

use crate::lang;

pub struct AuthorFactoryModel {
    author: Author,
}

#[derive(Debug)]
pub enum AuthorFactoryInput {
    DeleteSelf,
}

#[derive(Debug)]
pub enum AuthorFactoryOutput {
    DeleteAuthor(Author),
}

#[relm4::factory(pub)]
impl FactoryComponent for AuthorFactoryModel {
    type ParentWidget = adw::PreferencesGroup;
    type Input = AuthorFactoryInput;
    type Output = AuthorFactoryOutput;
    type Init = Author;
    type CommandOutput = ();

    view! {
        #[root]
        adw::ActionRow {
            set_title: &self.author.name(),
            set_subtitle: &self.author.email().clone().unwrap_or_default(),
            set_use_markup: false,

            add_suffix = &gtk::Button {
                set_icon_name: relm4_icons::icon_names::CROSS_LARGE,
                set_tooltip: &lang::lookup("author-remove"),
                add_css_class: "flat",

                connect_clicked[sender] => move |_| {
                    sender.input(AuthorFactoryInput::DeleteSelf);
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            author: init.clone(),
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
            AuthorFactoryInput::DeleteSelf => {
                let _ = sender.output(AuthorFactoryOutput::DeleteAuthor(self.author.clone()));
            }
        }
    }
}
