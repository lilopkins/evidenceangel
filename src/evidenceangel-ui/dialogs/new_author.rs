use adw::prelude::*;
use evidenceangel::Author;
use relm4::{
    adw::{self, ApplicationWindow},
    gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt,
};

use crate::lang;

pub struct NewAuthorDialogModel {}

#[derive(Debug)]
pub enum NewAuthorInput {
    Present(ApplicationWindow),
    Create,
}

#[derive(Debug)]
pub enum NewAuthorOutput {
    CreateAuthor(Author),
}

#[relm4::component(pub)]
impl Component for NewAuthorDialogModel {
    type Input = NewAuthorInput;
    type Output = NewAuthorOutput;
    type CommandOutput = ();
    type Init = ();

    view! {
        #[root]
        adw::Dialog {
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::PreferencesGroup {
                    set_title: &lang::lookup("author-create-title"),
                    set_margin_all: 16,

                    #[name = "name"]
                    adw::EntryRow {
                        set_title: &lang::lookup("author-create-name"),
                    },
                    #[name = "email"]
                    adw::EntryRow {
                        set_title: &lang::lookup("author-create-email"),
                    }
                },
                gtk::Separator {
                    set_orientation: gtk::Orientation::Horizontal,
                },
                gtk::Button {
                    set_label: &lang::lookup("author-create-submit"),
                    add_css_class: "flat",

                    connect_clicked[sender] => move |_| {
                        sender.input(NewAuthorInput::Create);
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
        let model = NewAuthorDialogModel {};
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
            NewAuthorInput::Present(window) => {
                root.present(Some(&window));
            }
            NewAuthorInput::Create => {
                let name = widgets.name.text().to_string();
                let email = widgets.email.text().to_string();
                let author = if email.trim().is_empty() {
                    Author::new(name)
                } else {
                    Author::new_with_email(name, email.trim().to_string())
                };
                sender
                    .output(NewAuthorOutput::CreateAuthor(author))
                    .unwrap();
                root.close();
            }
        }
        self.update_view(widgets, sender)
    }
}
