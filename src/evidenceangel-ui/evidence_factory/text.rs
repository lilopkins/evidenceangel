use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

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
    TextChanged,
}

#[derive(Debug)]
pub enum ComponentOutput {
    /// The text in this text evidence item has been changed.
    TextChanged { new_text: String },
}

pub struct ComponentInit {
    pub text: String,
}

#[relm4::component(pub)]
impl Component for ComponentModel {
    type CommandOutput = ();
    type Input = ComponentInput;
    type Output = ComponentOutput;
    type Init = ComponentInit;

    view! {
        #[root]
        gtk::Frame {
            gtk::ScrolledWindow {
                set_height_request: 100,
                set_hexpand: true,

                gtk::TextView {
                    set_left_margin: 8,
                    set_right_margin: 8,
                    set_top_margin: 8,
                    set_bottom_margin: 8,

                    #[name = "text_buffer"]
                    #[wrap(Some)]
                    set_buffer = &gtk::TextBuffer {
                        set_text: &init.text,
                        connect_changed => ComponentInput::Internal(ComponentInputInternal::TextChanged),
                    }
                }
            }
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
            ComponentInput::Internal(ComponentInputInternal::TextChanged) => {
                let buf = &widgets.text_buffer;
                let new_text = buf
                    .text(&buf.start_iter(), &buf.end_iter(), false)
                    .to_string();
                let _ = sender.output(ComponentOutput::TextChanged { new_text });
            }
        }
    }
}
