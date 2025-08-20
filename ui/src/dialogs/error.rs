use std::{fs, path::PathBuf};

use adw::prelude::*;
use relm4::{
    Component, ComponentParts, ComponentSender,
    adw::{self, ApplicationWindow},
    gtk::gio::Cancellable,
};

use crate::lang;

pub struct ErrorDialogModel {
    title: String,
    body: String,
    lock_file: Option<PathBuf>,
}

pub struct ErrorDialogInit {
    pub title: Box<dyn ToString>,
    pub body: Box<dyn ToString>,
}

#[derive(Debug)]
pub enum ErrorDialogInput {
    OfferLockRelease { lock_file: PathBuf },
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
            lock_file: None,
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
                let lock_file = self.lock_file.clone();
                root.clone()
                    .choose(&window, None::<&Cancellable>, move |response| {
                        if response == "unlock" {
                            if let Some(lock_file) = lock_file {
                                if let Err(e) = fs::remove_file(lock_file) {
                                    tracing::warn!("Failed to remove lock file: {e}");
                                }
                            }
                        }
                    });
            }
            ErrorDialogInput::OfferLockRelease { lock_file } => {
                root.add_response("unlock", &lang::lookup("delete-lock"));
                self.lock_file = Some(lock_file);
            }
        }
        self.update_view(widgets, sender);
    }
}
