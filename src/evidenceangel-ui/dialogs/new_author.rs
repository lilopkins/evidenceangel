use adw::prelude::*;
use evidenceangel::Author;
use relm4::{
    Component, ComponentParts, ComponentSender, RelmWidgetExt,
    adw::{self, ApplicationWindow},
    gtk,
};

use crate::lang;

pub struct NewAuthorDialogModel {}

#[derive(Debug)]
pub enum NewAuthorInput {
    Present(ApplicationWindow),
    _Create,
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
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: &lang::lookup("author-create-title"),
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,
                    set_margin_all: 16,

                    adw::PreferencesGroup {
                        //set_title: &lang::lookup("author-create-title"),

                        #[name = "name"]
                        adw::EntryRow {
                            set_title: &lang::lookup("author-create-name"),
                            connect_entry_activated => NewAuthorInput::_Create,
                        },
                        #[name = "email"]
                        adw::EntryRow {
                            set_title: &lang::lookup("author-create-email"),
                            connect_entry_activated => NewAuthorInput::_Create,
                        }
                    },

                    gtk::Button {
                        set_label: &lang::lookup("author-create-submit"),
                        add_css_class: "pill",
                        add_css_class: "suggested-action",
                        set_halign: gtk::Align::Center,

                        connect_clicked => NewAuthorInput::_Create,
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
            NewAuthorInput::_Create => {
                let name = widgets.name.text().to_string();
                let email = widgets.email.text().to_string();
                if !(name.trim().is_empty() && email.trim().is_empty()) {
                    let author = if email.trim().is_empty() {
                        Author::new(name)
                    } else {
                        Author::new_with_email(name, email.trim().to_string())
                    };
                    sender
                        .output(NewAuthorOutput::CreateAuthor(author))
                        .unwrap();
                }
                root.close();
            }
        }
        self.update_view(widgets, sender);
    }
}
