use adw::prelude::*;
use relm4::{
    Component, ComponentParts, ComponentSender,
    adw::{self, ApplicationWindow},
};

use crate::lang;

pub struct ErrorDialogModel {
    title: String,
    body: String,
}

pub struct ErrorDialogInit {
    pub title: Box<dyn ToString>,
    pub body: Box<dyn ToString>,
}

#[derive(Debug)]
pub enum ErrorDialogInput {
    Present(ApplicationWindow),
}

#[derive(Debug)]
pub enum ErrorDialogOutput {}

#[relm4::component(pub)]
impl Component for ErrorDialogModel {
    type Input = ErrorDialogInput;
    type Output = ErrorDialogOutput;
    type CommandOutput = ();
    type Init = ErrorDialogInit;

    view! {
        #[root]
        adw::AlertDialog {
            set_heading: Some(&model.title),
            set_body: &model.body,

            add_response: ("ok", &lang::lookup("ok")),
            set_default_response: Some("ok"),
            set_close_response: "ok",
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let ErrorDialogInit { title, body } = init;
        let model = ErrorDialogModel {
            title: title.to_string(),
            body: body.to_string(),
        };
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
            ErrorDialogInput::Present(window) => {
                root.present(Some(&window));
            }
        }
        self.update_view(widgets, sender);
    }
}
