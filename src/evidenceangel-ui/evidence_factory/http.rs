use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

use crate::lang;

pub struct ComponentModel {}

#[derive(Debug)]
pub enum ComponentInput {
    /// An internal message was triggered
    #[allow(
        private_interfaces,
        reason = "These messages should only be produced by this component."
    )]
    Internal(ComponentInputInternal),
}

#[derive(Debug)]
enum ComponentInputInternal {
    RequestChanged,
    ResponseChanged,
}

#[derive(Debug)]
pub enum ComponentOutput {
    /// The text in this text evidence item has been changed.
    RequestChanged {
        new_text: String,
    },
    ResponseChanged {
        new_text: String,
    },
}

pub struct ComponentInit {
    pub request: String,
    pub response: String,
}

#[relm4::component(pub)]
impl Component for ComponentModel {
    type CommandOutput = ();
    type Input = ComponentInput;
    type Output = ComponentOutput;
    type Init = ComponentInit;

    view! {
        #[root]
        gtk::Box {
            set_spacing: 8,

            gtk::Frame {
                set_height_request: super::EVIDENCE_HEIGHT_REQUEST,
                set_label: Some(&lang::lookup("evidence-http-request")),

                gtk::ScrolledWindow {
                    set_hexpand: true,

                    gtk::TextView {
                        add_css_class: "monospace",
                        set_left_margin: 8,
                        set_right_margin: 8,
                        set_top_margin: 8,
                        set_bottom_margin: 8,
                        set_halign: gtk::Align::Fill,
                        set_valign: gtk::Align::Fill,

                        #[name = "request_text_buffer"]
                        #[wrap(Some)]
                        set_buffer = &gtk::TextBuffer {
                            set_text: &init.request,
                            connect_changed => ComponentInput::Internal(ComponentInputInternal::RequestChanged),
                        }
                    }
                }
            },
            gtk::Frame {
                set_height_request: super::EVIDENCE_HEIGHT_REQUEST,
                set_label: Some(&lang::lookup("evidence-http-response")),

                gtk::ScrolledWindow {
                    set_hexpand: true,

                    gtk::TextView {
                        add_css_class: "monospace",
                        set_left_margin: 8,
                        set_right_margin: 8,
                        set_top_margin: 8,
                        set_bottom_margin: 8,
                        set_halign: gtk::Align::Fill,
                        set_valign: gtk::Align::Fill,

                        #[name = "response_text_buffer"]
                        #[wrap(Some)]
                        set_buffer = &gtk::TextBuffer {
                            set_text: &init.response,
                            connect_changed => ComponentInput::Internal(ComponentInputInternal::ResponseChanged),
                        }
                    }
                }
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ComponentModel {};
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ComponentInput::Internal(ComponentInputInternal::RequestChanged) => {
                let buf = &widgets.request_text_buffer;
                let new_text = buf
                    .text(&buf.start_iter(), &buf.end_iter(), false)
                    .to_string();
                let _ = sender.output(ComponentOutput::RequestChanged { new_text });
            }
            ComponentInput::Internal(ComponentInputInternal::ResponseChanged) => {
                let buf = &widgets.response_text_buffer;
                let new_text = buf
                    .text(&buf.start_iter(), &buf.end_iter(), false)
                    .to_string();
                let _ = sender.output(ComponentOutput::ResponseChanged { new_text });
            }
        }
    }
}
