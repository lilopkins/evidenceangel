use std::{
    cmp::Ordering,
    sync::{Arc, Mutex, RwLock},
};

use adw::prelude::*;
use angelmark::{
    parse_angelmark, AngelmarkLine, AngelmarkTable, AngelmarkTableRow, AngelmarkText, OwnedSpan,
};
use evidenceangel::{Evidence, EvidenceData, EvidenceKind, EvidencePackage};
#[allow(unused_imports)]
use gtk::prelude::*;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    adw,
    factory::FactoryView,
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    FactorySender, RelmWidgetExt,
};

use crate::util::BoxedEvidenceJson;
use crate::{lang, lang_args};

const EVIDENCE_HEIGHT_REQUEST: i32 = 300;
const HTTP_SEPARATOR: char = '\x1e';

relm4::new_action_group!(RichTextEditorActionGroup, "rich-text-editor");
relm4::new_stateless_action!(BoldAction, RichTextEditorActionGroup, "bold");
relm4::new_stateless_action!(ItalicAction, RichTextEditorActionGroup, "italic");

pub struct EvidenceFactoryModel {
    index: DynamicIndex,
    evidence: Arc<RwLock<Evidence>>,
    package: Arc<RwLock<EvidencePackage>>,
}

impl EvidenceFactoryModel {
    fn get_data(&self) -> Vec<u8> {
        tracing::debug!("Got some {:?} data", self.evidence.read().unwrap().kind());
        match self.evidence.read().unwrap().value() {
            EvidenceData::Text { content } => content.as_bytes().to_vec(),
            EvidenceData::Base64 { data } => data.clone(),
            EvidenceData::Media { hash } => {
                tracing::debug!("Fetching media with hash {hash}");
                let mut pkg = self.package.write().unwrap();
                tracing::debug!("Got package instance!");
                let media = pkg.get_media(hash).ok().flatten();
                tracing::debug!("Got media {media:?}");
                if let Some(media) = media {
                    media.data().clone()
                } else {
                    lang::lookup("invalid-data").as_bytes().to_vec()
                }
            }
        }
    }

    fn get_data_as_string(&self) -> String {
        tracing::debug!("Converting media to string...");
        String::from_utf8(self.get_data()).unwrap_or(lang::lookup("invalid-data"))
    }

    fn get_data_as_texture(&self) -> Option<gtk::gdk::Texture> {
        tracing::debug!("Converting media to texture...");
        let glib_bytes = gtk::glib::Bytes::from_owned(self.get_data().clone());
        let r = gtk::gdk::Texture::from_bytes(&glib_bytes).ok();
        tracing::debug!("Resultant texture: {r:?}");
        r
    }
}

#[derive(Debug)]
pub enum EvidenceFactoryInput {
    /// Set the text for a text evidence object. If not text evidence, ignore.
    TextSetText(String),
    /// Set the text for a rich text evidence object. If not rich text evidence, ignore.
    RichTextSetText(String),
    /// Set the HTTP request text. If not HTTP evidence, ignore.
    HttpSetRequest(String),
    /// Set the HTTP response text. If not HTTP evidence, ignore.
    HttpSetResponse(String),
    /// Set the caption for this evidence.
    SetCaption(String),
    MoveUp,
    MoveDown,
    Delete,
}

#[derive(Debug)]
pub enum EvidenceFactoryOutput {
    /// Replace the evidence at the given position. This MUST NOT trigger an update to the interface.
    UpdateEvidence(DynamicIndex, Evidence),
    /// Delete evidence at the given position. This MUST trigger an update to the interface.
    /// Second parameter defines if it is user inititiated
    DeleteEvidence(DynamicIndex, bool),
    /// `InsertEvidenceAt` MUST be followed by a `DeleteEvidence` call as it is only triggered by a data move.
    /// As such, it MUST NOT trigger an update to the interface.
    InsertEvidenceAt(DynamicIndex, isize, Evidence),
}

pub struct EvidenceFactoryInit {
    pub evidence: Evidence,
    pub package: Arc<RwLock<EvidencePackage>>,
}

#[relm4::factory(pub)]
impl FactoryComponent for EvidenceFactoryModel {
    type ParentWidget = gtk::Box;
    type Input = EvidenceFactoryInput;
    type Output = EvidenceFactoryOutput;
    type Init = EvidenceFactoryInit;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            add_controller = gtk::DragSource {
                set_actions: gtk::gdk::DragAction::MOVE,

                connect_prepare => move |_slf, _x, _y| {
                    let dnd_data = BoxedEvidenceJson::new((*ev.read().unwrap()).clone());
                    tracing::debug!("Drag data started: {dnd_data:?}");
                    Some(gtk::gdk::ContentProvider::for_value(&dnd_data.to_value()))
                },

                connect_drag_end[sender, index] => move |_slf, _drag, delete_data| {
                    if delete_data {
                        tracing::debug!("Deleting drag start item");
                        sender.output(EvidenceFactoryOutput::DeleteEvidence(index.clone(), false)).unwrap();
                    }
                }
            },
            add_controller = gtk::DropTarget {
                set_actions: gtk::gdk::DragAction::MOVE,
                set_types: &[BoxedEvidenceJson::static_type()],

                connect_drop[sender, index] => move |_slf, val, _x, _y| {
                    tracing::debug!("Dropped type: {:?}", val.type_());
                    if let Ok(data) = val.get::<BoxedEvidenceJson>() {
                        let ev = data.inner();
                        tracing::debug!("Dropped data: {ev:?}");
                        sender.output(EvidenceFactoryOutput::InsertEvidenceAt(index.clone(), 0, ev)).unwrap();
                        return true;
                    }
                    false
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 8,
                set_hexpand: true,

                gtk::Separator {},

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,

                    gtk::Image {
                        set_icon_name: Some(relm4_icons::icon_names::CHEVRON_UP_DOWN_REGULAR),
                    },
                    gtk::Entry {
                        set_placeholder_text: Some(&lang::lookup("test-evidence-caption")),
                        set_hexpand: true,
                        set_text: &self.evidence.read().unwrap().caption().as_ref().unwrap_or(&String::new()),

                        connect_changed[sender] => move |entry| {
                            sender.input(EvidenceFactoryInput::SetCaption(entry.text().to_string()));
                        }
                    },
                    gtk::MenuButton {
                        set_tooltip: &lang::lookup("evidence-menu"),

                        #[wrap(Some)]
                        set_popover = &gtk::Popover {
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 4,

                                gtk::Button {
                                    set_label: &lang::lookup("evidence-move-up"),
                                    add_css_class: "flat",

                                    connect_clicked => EvidenceFactoryInput::MoveUp,
                                },
                                gtk::Button {
                                    set_label: &lang::lookup("evidence-move-down"),
                                    add_css_class: "flat",

                                    connect_clicked => EvidenceFactoryInput::MoveDown,
                                },
                                gtk::Button {
                                    set_label: &lang::lookup("evidence-delete"),
                                    add_css_class: "flat",
                                    add_css_class: "destructive-action",

                                    connect_clicked => EvidenceFactoryInput::Delete,
                                },
                            }
                        }
                    },
                },
                #[name = "evidence_child"]
                gtk::Box {},
            },
        }
    }

    fn init_model(init: Self::Init, index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let EvidenceFactoryInit { evidence, package } = init;
        Self {
            index: index.clone(),
            evidence: Arc::new(RwLock::new(evidence)),
            package,
        }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let ev = self.evidence.clone();
        let widgets = view_output!();
        self.index = index.clone();

        match self.evidence.read().unwrap().kind() {
            EvidenceKind::Text => {
                let scroll_window = gtk::ScrolledWindow::default();
                scroll_window.set_height_request(100);
                scroll_window.set_hexpand(true);

                let frame = gtk::Frame::new(None);

                let text_view = gtk::TextView::new();
                text_view.set_top_margin(8);
                text_view.set_bottom_margin(8);
                text_view.set_left_margin(8);
                text_view.set_right_margin(8);

                text_view.buffer().set_text(&self.get_data_as_string());
                let sender_c = sender.clone();
                text_view.buffer().connect_changed(move |buf| {
                    sender_c.input(EvidenceFactoryInput::TextSetText(
                        buf.text(&buf.start_iter(), &buf.end_iter(), false)
                            .to_string(),
                    ));
                });

                scroll_window.set_child(Some(&text_view));
                frame.set_child(Some(&scroll_window));
                widgets.evidence_child.append(&frame);
            }
            EvidenceKind::RichText => {
                let bx = gtk::Box::new(gtk::Orientation::Vertical, 4);

                let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 4);

                let btn_bold = gtk::Button::new();
                btn_bold.set_icon_name(relm4_icons::icon_names::TEXT_BOLD);
                btn_bold.set_tooltip(&lang::lookup("rich-text-bold"));
                toolbar.append(&btn_bold);
                let btn_italic = gtk::Button::new();
                btn_italic.set_icon_name(relm4_icons::icon_names::TEXT_ITALIC);
                btn_italic.set_tooltip(&lang::lookup("rich-text-italic"));
                toolbar.append(&btn_italic);
                let btn_monospace = gtk::Button::new();
                btn_monospace.set_icon_name(relm4_icons::icon_names::CODE);
                btn_monospace.set_tooltip(&lang::lookup("rich-text-monospace"));
                toolbar.append(&btn_monospace);
                toolbar.append(&gtk::Separator::new(gtk::Orientation::Vertical));
                let btn_h1 = gtk::Button::new();
                btn_h1.set_icon_name(relm4_icons::icon_names::TEXT_HEADER_1_LINES_CARET_REGULAR);
                btn_h1.set_tooltip(&lang::lookup("rich-text-heading-1"));
                toolbar.append(&btn_h1);
                let btn_h2 = gtk::Button::new();
                btn_h2.set_icon_name(relm4_icons::icon_names::TEXT_HEADER_2_LINES_CARET_REGULAR);
                btn_h2.set_tooltip(&lang::lookup("rich-text-heading-2"));
                toolbar.append(&btn_h2);
                let btn_h3 = gtk::Button::new();
                btn_h3.set_icon_name(relm4_icons::icon_names::TEXT_HEADER_3_LINES_CARET_REGULAR);
                btn_h3.set_tooltip(&lang::lookup("rich-text-heading-3"));
                toolbar.append(&btn_h3);

                let scroll_window = gtk::ScrolledWindow::default();
                scroll_window.set_height_request(200);
                scroll_window.set_hexpand(true);

                let frame = gtk::Frame::new(None);

                let text_view = gtk::TextView::new();
                text_view.set_top_margin(8);
                text_view.set_bottom_margin(8);
                text_view.set_left_margin(8);
                text_view.set_right_margin(8);
                text_view.set_wrap_mode(gtk::WrapMode::Word);

                let error_message = gtk::Box::new(gtk::Orientation::Horizontal, 4);
                error_message.set_margin_start(8);
                error_message.append(&{
                    let i = gtk::Image::new();
                    i.set_icon_name(Some(relm4_icons::icon_names::WARNING));
                    i
                });
                error_message.append(&gtk::Label::new(Some(&lang::lookup(
                    "rich-text-parsing-failure",
                ))));
                toolbar.append(&error_message);

                let processing = Arc::new(Mutex::new(false));
                {
                    let processing = processing.clone();
                    let error_popover = error_message.clone();
                    let frame = frame.clone();
                    text_view.buffer().connect_changed(move |buf| {
                        if *processing.lock().unwrap() {
                            return;
                        }

                        let text = buf
                            .text(&buf.start_iter(), &buf.end_iter(), false)
                            .to_string();

                        // Update preview
                        *processing.lock().unwrap() = true;
                        if let Ok(lines) = parse_angelmark(&text) {
                            let cursor_pos = buf.cursor_position();

                            for line in lines {
                                let (start, end) = *line.span().span();
                                // Remove plain text
                                #[allow(clippy::cast_possible_wrap)]
                                buf.delete(
                                    &mut buf.iter_at_offset(start as i32),
                                    &mut buf.iter_at_offset(end as i32),
                                );

                                // Determine formatting
                                let markup = angelmark_to_pango(&line);
                                tracing::trace!("line: {line:?}");
                                tracing::trace!("pango: {markup:?}");

                                // Reinsert marked up text
                                #[allow(clippy::cast_possible_wrap)]
                                buf.insert_markup(&mut buf.iter_at_offset(start as i32), &markup);
                            }

                            buf.place_cursor(&buf.iter_at_offset(cursor_pos));
                            frame.remove_css_class("warning");
                            error_popover.set_visible(false);
                        } else {
                            frame.add_css_class("warning");
                            error_popover.set_visible(true);
                        }
                        *processing.lock().unwrap() = false;
                    });
                }
                text_view.buffer().set_text(&self.get_data_as_string());

                {
                    let sender = sender.clone();
                    let processing = processing.clone();
                    text_view.buffer().connect_changed(move |buf| {
                        if *processing.lock().unwrap() {
                            return;
                        }

                        let text = buf
                            .text(&buf.start_iter(), &buf.end_iter(), false)
                            .to_string();
                        sender.input(EvidenceFactoryInput::RichTextSetText(text.clone()));
                    });
                }

                // Register accelerators
                let mut group = RelmActionGroup::<RichTextEditorActionGroup>::new();
                let action_bold: RelmAction<BoldAction> = {
                    let tv = text_view.clone();
                    let buf = text_view.buffer();
                    RelmAction::new_stateless(move |_| {
                        add_angelmark_tokens(&tv, &buf, "**");
                    })
                };
                group.add_action(action_bold);
                let action_italic: RelmAction<ItalicAction> = {
                    let tv = text_view.clone();
                    let buf = text_view.buffer();
                    RelmAction::new_stateless(move |_| {
                        add_angelmark_tokens(&tv, &buf, "_");
                    })
                };
                group.add_action(action_italic);
                group.register_for_widget(&text_view);

                let sc = gtk::ShortcutController::new();
                sc.add_shortcut(gtk::Shortcut::new(
                    Some(gtk::ShortcutTrigger::parse_string("<primary>B").unwrap()),
                    Some(
                        gtk::ShortcutAction::parse_string("action(rich-text-editor.bold)").unwrap(),
                    ),
                ));
                sc.add_shortcut(gtk::Shortcut::new(
                    Some(gtk::ShortcutTrigger::parse_string("<primary>I").unwrap()),
                    Some(
                        gtk::ShortcutAction::parse_string("action(rich-text-editor.italic)")
                            .unwrap(),
                    ),
                ));
                text_view.add_controller(sc);

                // Set up buttons
                {
                    let tv = text_view.clone();
                    let buf = text_view.buffer();
                    btn_bold.connect_clicked(move |_| {
                        add_angelmark_tokens(&tv, &buf, "**");
                    });
                }
                {
                    let tv = text_view.clone();
                    let buf = text_view.buffer();
                    btn_italic.connect_clicked(move |_| {
                        add_angelmark_tokens(&tv, &buf, "_");
                    });
                }
                {
                    let tv = text_view.clone();
                    let buf = text_view.buffer();
                    btn_monospace.connect_clicked(move |_| {
                        add_angelmark_tokens(&tv, &buf, "`");
                    });
                }
                {
                    let tv = text_view.clone();
                    let buf = text_view.buffer();
                    btn_h1.connect_clicked(move |_| {
                        set_angelmark_heading_level(&tv, &buf, 1);
                    });
                }
                {
                    let tv = text_view.clone();
                    let buf = text_view.buffer();
                    btn_h2.connect_clicked(move |_| {
                        set_angelmark_heading_level(&tv, &buf, 2);
                    });
                }
                {
                    let tv = text_view.clone();
                    let buf = text_view.buffer();
                    btn_h3.connect_clicked(move |_| {
                        set_angelmark_heading_level(&tv, &buf, 3);
                    });
                }

                scroll_window.set_child(Some(&text_view));
                frame.set_child(Some(&scroll_window));
                bx.append(&toolbar);
                bx.append(&frame);
                widgets.evidence_child.append(&bx);
            }
            EvidenceKind::Image => {
                let img = gtk::Picture::new();
                img.set_paintable(self.get_data_as_texture().as_ref());
                img.set_hexpand(true);
                img.set_height_request(EVIDENCE_HEIGHT_REQUEST);
                widgets.evidence_child.append(&img);
            }
            EvidenceKind::Http => {
                let data = self.get_data_as_string();
                let data_parts = data
                    .split(HTTP_SEPARATOR)
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                let request = data_parts.first().cloned().unwrap_or_default();
                let response = data_parts.get(1).cloned().unwrap_or_default();

                let frame = gtk::Frame::default();
                frame.set_height_request(EVIDENCE_HEIGHT_REQUEST);
                frame.set_label(Some(&lang::lookup("evidence-http-request")));
                let scrolled = gtk::ScrolledWindow::new();
                scrolled.set_hexpand(true);
                let txt_request = gtk::TextView::default();
                txt_request.add_css_class("monospace");
                txt_request.buffer().set_text(&request);
                txt_request.set_top_margin(8);
                txt_request.set_bottom_margin(8);
                txt_request.set_left_margin(8);
                txt_request.set_right_margin(8);
                txt_request.set_halign(gtk::Align::Fill);
                txt_request.set_valign(gtk::Align::Fill);
                let sender_c = sender.clone();
                txt_request.buffer().connect_changed(move |buf| {
                    sender_c.input(EvidenceFactoryInput::HttpSetRequest(
                        buf.text(&buf.start_iter(), &buf.end_iter(), false)
                            .to_string(),
                    ));
                });
                scrolled.set_child(Some(&txt_request));
                frame.set_child(Some(&scrolled));

                widgets.evidence_child.set_spacing(8);
                widgets.evidence_child.append(&frame);

                let frame = gtk::Frame::default();
                frame.set_height_request(EVIDENCE_HEIGHT_REQUEST);
                frame.set_label(Some(&lang::lookup("evidence-http-response")));
                let scrolled = gtk::ScrolledWindow::new();
                scrolled.set_hexpand(true);
                let txt_response = gtk::TextView::default();
                txt_response.add_css_class("monospace");
                txt_response.buffer().set_text(&response);
                txt_response.set_top_margin(8);
                txt_response.set_bottom_margin(8);
                txt_response.set_left_margin(8);
                txt_response.set_right_margin(8);
                txt_response.set_halign(gtk::Align::Fill);
                txt_response.set_valign(gtk::Align::Fill);
                let sender_c = sender.clone();
                txt_response.buffer().connect_changed(move |buf| {
                    sender_c.input(EvidenceFactoryInput::HttpSetResponse(
                        buf.text(&buf.start_iter(), &buf.end_iter(), false)
                            .to_string(),
                    ));
                });
                scrolled.set_child(Some(&txt_response));
                frame.set_child(Some(&scrolled));
                widgets.evidence_child.append(&frame);
            }
            EvidenceKind::File => {
                let label = gtk::Label::default();
                label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
                if let Some(filename) = self.evidence.read().unwrap().original_filename() {
                    label.set_markup(&lang::lookup_with_args(
                        "test-evidence-file-named",
                        &lang_args!("filename", filename),
                    ));
                } else {
                    label.set_markup(&lang::lookup("test-evidence-file-unnamed"));
                }
                widgets.evidence_child.set_halign(gtk::Align::Center);
                widgets.evidence_child.append(&label);
            }
        };

        widgets
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            EvidenceFactoryInput::SetCaption(new_caption) => {
                self.evidence
                    .write()
                    .unwrap()
                    .caption_mut()
                    .replace(new_caption);
                sender
                    .output(EvidenceFactoryOutput::UpdateEvidence(
                        self.index.clone(),
                        self.evidence.read().unwrap().clone(),
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::TextSetText(new_text) => {
                if *self.evidence.read().unwrap().kind() != EvidenceKind::Text {
                    return;
                }
                match self.evidence.write().unwrap().value_mut() {
                    EvidenceData::Text { content } => {
                        *content = new_text;
                    }
                    EvidenceData::Base64 { data } => {
                        *data = new_text.into_bytes();
                    }
                    EvidenceData::Media { .. } => panic!("cannot handle text of media type"),
                }
                sender
                    .output(EvidenceFactoryOutput::UpdateEvidence(
                        self.index.clone(),
                        self.evidence.read().unwrap().clone(),
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::RichTextSetText(new_text) => {
                if *self.evidence.read().unwrap().kind() != EvidenceKind::RichText {
                    return;
                }
                match self.evidence.write().unwrap().value_mut() {
                    EvidenceData::Text { content } => {
                        *content = new_text;
                    }
                    EvidenceData::Base64 { data } => {
                        *data = new_text.into_bytes();
                    }
                    EvidenceData::Media { .. } => panic!("cannot handle text of media type"),
                }
                sender
                    .output(EvidenceFactoryOutput::UpdateEvidence(
                        self.index.clone(),
                        self.evidence.read().unwrap().clone(),
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::HttpSetRequest(mut new_req) => {
                if *self.evidence.read().unwrap().kind() != EvidenceKind::Http {
                    return;
                }
                match self.evidence.write().unwrap().value_mut() {
                    EvidenceData::Text { content } => {
                        let data_parts = content
                            .split(HTTP_SEPARATOR)
                            .map(ToString::to_string)
                            .collect::<Vec<_>>();
                        let response = data_parts.get(1).cloned().unwrap_or_default();

                        new_req.push(HTTP_SEPARATOR);
                        new_req.push_str(&response);
                        *content = new_req;
                    }
                    EvidenceData::Base64 { data } => {
                        let data_parts = String::from_utf8_lossy(data)
                            .split(HTTP_SEPARATOR)
                            .map(ToString::to_string)
                            .collect::<Vec<_>>();
                        let response = data_parts.get(1).cloned().unwrap_or_default();

                        new_req.push(HTTP_SEPARATOR);
                        new_req.push_str(&response);
                        *data = new_req.into_bytes();
                    }
                    EvidenceData::Media { .. } => panic!("cannot handle text of media type"),
                }
                sender
                    .output(EvidenceFactoryOutput::UpdateEvidence(
                        self.index.clone(),
                        self.evidence.read().unwrap().clone(),
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::HttpSetResponse(new_res) => {
                if *self.evidence.read().unwrap().kind() != EvidenceKind::Http {
                    return;
                }
                match self.evidence.write().unwrap().value_mut() {
                    EvidenceData::Text { content } => {
                        let data_parts = content
                            .split(HTTP_SEPARATOR)
                            .map(ToString::to_string)
                            .collect::<Vec<_>>();
                        let mut request = data_parts.first().cloned().unwrap_or_default();

                        request.push(HTTP_SEPARATOR);
                        request.push_str(&new_res);
                        *content = request;
                    }
                    EvidenceData::Base64 { data } => {
                        let data_parts = String::from_utf8_lossy(data)
                            .split(HTTP_SEPARATOR)
                            .map(ToString::to_string)
                            .collect::<Vec<_>>();
                        let mut request = data_parts.first().cloned().unwrap_or_default();

                        request.push(HTTP_SEPARATOR);
                        request.push_str(&new_res);
                        *data = request.into_bytes();
                    }
                    EvidenceData::Media { .. } => panic!("cannot handle text of media type"),
                }
                sender
                    .output(EvidenceFactoryOutput::UpdateEvidence(
                        self.index.clone(),
                        self.evidence.read().unwrap().clone(),
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::Delete => {
                sender
                    .output(EvidenceFactoryOutput::DeleteEvidence(
                        self.index.clone(),
                        true,
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::MoveUp => {
                sender
                    .output(EvidenceFactoryOutput::InsertEvidenceAt(
                        self.index.clone(),
                        -1,
                        self.evidence.read().unwrap().clone(),
                    ))
                    .unwrap();
                sender
                    .output(EvidenceFactoryOutput::DeleteEvidence(
                        self.index.clone(),
                        false,
                    ))
                    .unwrap();
            }
            EvidenceFactoryInput::MoveDown => {
                sender
                    .output(EvidenceFactoryOutput::InsertEvidenceAt(
                        self.index.clone(),
                        2, // insert after self, which hasn't yet been deleted
                        self.evidence.read().unwrap().clone(),
                    ))
                    .unwrap();
                sender
                    .output(EvidenceFactoryOutput::DeleteEvidence(
                        self.index.clone(),
                        false,
                    ))
                    .unwrap();
            }
        }
    }
}

#[allow(clippy::cast_possible_wrap)]
fn add_angelmark_tokens(text_view: &gtk::TextView, buf: &gtk::TextBuffer, token: &str) {
    let token_len = token.len() as i32;
    if let Some((start, end)) = buf.selection_bounds() {
        // Check if already has token
        let start_pos = start.offset();
        let end_pos = end.offset();
        let mut add_token = true;

        if start_pos >= token_len
            && end_pos <= (buf.end_iter().offset() - buf.start_iter().offset() - token_len)
            && buf.text(
                &buf.iter_at_offset(start_pos - token_len),
                &buf.iter_at_offset(start_pos),
                false,
            ) == token
            && buf.text(
                &buf.iter_at_offset(start_pos - token_len),
                &buf.iter_at_offset(start_pos),
                false,
            ) == token
        {
            add_token = false;
        }

        if add_token {
            buf.insert(&mut buf.iter_at_offset(end_pos), token);
            buf.insert(&mut buf.iter_at_offset(start_pos), token);
            buf.select_range(
                &buf.iter_at_offset(start_pos + token_len),
                &buf.iter_at_offset(end_pos + token_len),
            );
        } else {
            buf.delete(
                &mut buf.iter_at_offset(end_pos),
                &mut buf.iter_at_offset(end_pos + token_len),
            );
            buf.delete(
                &mut buf.iter_at_offset(start_pos - token_len),
                &mut buf.iter_at_offset(start_pos),
            );
        }
    } else {
        let mut iter = buf.iter_at_offset(buf.cursor_position());
        buf.insert(&mut iter, &format!("{token}{token}"));
        iter.backward_chars(token_len);
        buf.place_cursor(&iter);
    }
    text_view.grab_focus();
}

#[allow(clippy::cast_possible_wrap)]
fn set_angelmark_heading_level(text_view: &gtk::TextView, buf: &gtk::TextBuffer, level: usize) {
    // Check number of "#" at beginning of line
    let mut current_heading_level = 0;
    let mut start_of_line = buf.iter_at_offset(buf.cursor_position());
    start_of_line.set_line_offset(0);
    let mut six_in = start_of_line;
    for _ in 0..6 {
        six_in.forward_char();
        if six_in.ends_line() {
            break;
        }
    }
    let first_6 = buf.text(&start_of_line, &six_in, false);
    for c in first_6.chars() {
        if c == '#' {
            current_heading_level += 1;
        } else {
            break;
        }
    }

    tracing::trace!("heading level: target = {level}, curr = {current_heading_level}");

    // Add or remove "#" as needed to meet `level`
    match level.cmp(&current_heading_level) {
        Ordering::Greater => {
            // Add
            let mut s = String::new();
            for _ in current_heading_level..level {
                s.push('#');
            }
            if current_heading_level == 0 {
                s.push(' ');
            }

            tracing::trace!("Adding: {s:?}");
            buf.insert(&mut start_of_line, &s);
        }
        Ordering::Less => {
            // Remove
            let num_to_delete = current_heading_level - level;
            let mut delete_end = start_of_line;
            delete_end.forward_chars(num_to_delete as i32);
            tracing::trace!("Deleting {num_to_delete} chars");
            buf.delete(&mut start_of_line, &mut delete_end);
        }
        Ordering::Equal => (),
    }

    text_view.grab_focus();
}

fn angelmark_to_pango(angelmark: &AngelmarkLine) -> String {
    match angelmark {
        AngelmarkLine::Newline(span) => span.original().clone(),
        AngelmarkLine::TextLine(txt, _span) => angelmark_text_to_pango(txt),
        AngelmarkLine::Heading1(txt, span) => {
            let (prefix, suffix) =
                get_prefix_and_suffix_around_children(span, txt.iter().map(AngelmarkText::span));
            format!(
                r#"<span size="xx-large">{prefix}{}</span>{suffix}"#,
                txt.iter().map(angelmark_text_to_pango).collect::<String>()
            )
        }
        AngelmarkLine::Heading2(txt, span) => {
            let (prefix, suffix) =
                get_prefix_and_suffix_around_children(span, txt.iter().map(AngelmarkText::span));
            format!(
                r#"<span size="x-large">{prefix}{}</span>{suffix}"#,
                txt.iter().map(angelmark_text_to_pango).collect::<String>()
            )
        }
        AngelmarkLine::Heading3(txt, span) => {
            let (prefix, suffix) =
                get_prefix_and_suffix_around_children(span, txt.iter().map(AngelmarkText::span));
            format!(
                r#"<span size="large">{prefix}{}</span>{suffix}"#,
                txt.iter().map(angelmark_text_to_pango).collect::<String>()
            )
        }
        AngelmarkLine::Heading4(txt, span)
        | AngelmarkLine::Heading5(txt, span)
        | AngelmarkLine::Heading6(txt, span) => {
            let (prefix, suffix) =
                get_prefix_and_suffix_around_children(span, txt.iter().map(AngelmarkText::span));
            let sub = txt.iter().map(angelmark_text_to_pango).collect::<String>();
            format!("{prefix}{sub}{suffix}")
        }
        AngelmarkLine::Table(table, span) => {
            let (prefix, suffix) = get_prefix_and_suffix_around_child(span, table.span());
            let sub = angelmark_table_to_pango(table);
            format!("{prefix}{sub}{suffix}")
        }
    }
}

fn angelmark_text_to_pango(angelmark: &AngelmarkText) -> String {
    match angelmark {
        AngelmarkText::Raw(_, span) => span.original().clone(),
        AngelmarkText::Bold(content, span) => {
            let (prefix, suffix) = get_prefix_and_suffix_around_child(span, content.span());
            format!(
                "<b>{prefix}{}{suffix}</b>",
                angelmark_text_to_pango(content)
            )
        }
        AngelmarkText::Italic(content, span) => {
            let (prefix, suffix) = get_prefix_and_suffix_around_child(span, content.span());
            format!(
                "<i>{prefix}{}{suffix}</i>",
                angelmark_text_to_pango(content)
            )
        }
        AngelmarkText::Monospace(content, span) => {
            let (prefix, suffix) = get_prefix_and_suffix_around_child(span, content.span());
            format!(
                "<tt>{prefix}{}{suffix}</tt>",
                angelmark_text_to_pango(content)
            )
        }
    }
}

fn angelmark_table_to_pango(table: &AngelmarkTable) -> String {
    let (prefix, suffix) = get_prefix_and_suffix_around_children(
        table.span(),
        table.rows().iter().map(AngelmarkTableRow::span),
    );
    let mut output = String::new();

    let mut row_iter = table.rows().iter();

    // Row 0
    output.push_str(&angelmark_table_row_to_pango(row_iter.next().unwrap()));

    // Alignment row (no styling)
    output.push_str(table.alignment().span().original());

    // Rows 1+
    for row in row_iter {
        output.push_str(&angelmark_table_row_to_pango(row));
    }

    format!("{prefix}{output}{suffix}")
}

#[allow(clippy::cast_possible_wrap)]
fn angelmark_table_row_to_pango(row: &AngelmarkTableRow) -> String {
    let mut output = row.span().original().clone();
    let mut offset = row.span().span().0 as i32;

    for cell in row.cells() {
        let original = cell.span().original();
        let formatted = cell
            .content()
            .iter()
            .map(angelmark_text_to_pango)
            .collect::<String>();

        let (start, end) = cell.span().span();
        let start = *start as i32 - offset;
        let end = *end as i32 - offset;
        assert!(start >= 0);
        assert!(end >= 0);
        let start = start as usize;
        let end = end as usize;

        // Replace output with formatted text
        output.replace_range(start..end, &formatted);

        // Update offset
        offset -= formatted.len() as i32 - original.len() as i32;
    }

    output
}

fn get_prefix_and_suffix_around_children<'a, I>(
    my_span: &'a OwnedSpan,
    children_span: I,
) -> (&'a str, &'a str)
where
    I: Iterator<Item = &'a OwnedSpan> + Clone,
{
    let (s, e) = *my_span.span();
    let (s2, e2) = (
        children_span.clone().map(|t| t.span().0).min().unwrap(),
        children_span.map(|t| t.span().1).max().unwrap(),
    );
    let prefix_len = s2 - s;
    let suffix_len = e - e2;
    let prefix = &my_span.original()[0..prefix_len];
    let suffix = &my_span.original()[(e2 - s)..(e2 - s + suffix_len)];
    (prefix, suffix)
}

fn get_prefix_and_suffix_around_child<'a>(
    my_span: &'a OwnedSpan,
    child_span: &'a OwnedSpan,
) -> (&'a str, &'a str) {
    get_prefix_and_suffix_around_children(my_span, [child_span].iter().copied())
}
