use std::cmp::Ordering;

use angelmark::{
    AngelmarkLine, AngelmarkTable, AngelmarkTableRow, AngelmarkText, OwnedSpan, parse_angelmark,
};
use gtk::prelude::*;
use relm4::{
    Component, ComponentParts, ComponentSender, RelmWidgetExt,
    actions::{RelmAction, RelmActionGroup},
    gtk,
};

use crate::lang;

relm4::new_action_group!(RichTextEditorActionGroup, "rich-text-editor");
relm4::new_stateless_action!(BoldAction, RichTextEditorActionGroup, "bold");
relm4::new_stateless_action!(ItalicAction, RichTextEditorActionGroup, "italic");

pub struct ComponentModel {
    parse_warning: bool,
}

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
    AddTokens(&'static str),
    SetHeadingLevel(usize),
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
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 4,

            // Toolbar
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 4,

                gtk::Button {
                    set_icon_name: relm4_icons::icon_names::TEXT_BOLD,
                    set_tooltip: &lang::lookup("rich-text-bold"),
                    connect_clicked => ComponentInput::Internal(ComponentInputInternal::AddTokens("**")),
                },
                gtk::Button {
                    set_icon_name: relm4_icons::icon_names::TEXT_ITALIC,
                    set_tooltip: &lang::lookup("rich-text-italic"),
                    connect_clicked => ComponentInput::Internal(ComponentInputInternal::AddTokens("_")),
                },
                gtk::Button {
                    set_icon_name: relm4_icons::icon_names::CODE,
                    set_tooltip: &lang::lookup("rich-text-monospace"),
                    connect_clicked => ComponentInput::Internal(ComponentInputInternal::AddTokens("`")),
                },
                gtk::Separator {
                    set_orientation: gtk::Orientation::Vertical,
                },
                gtk::Button {
                    set_icon_name: relm4_icons::icon_names::TEXT_HEADER_1_LINES_CARET_REGULAR,
                    set_tooltip: &lang::lookup("rich-text-heading-1"),
                    connect_clicked => ComponentInput::Internal(ComponentInputInternal::SetHeadingLevel(1)),
                },
                gtk::Button {
                    set_icon_name: relm4_icons::icon_names::TEXT_HEADER_2_LINES_CARET_REGULAR,
                    set_tooltip: &lang::lookup("rich-text-heading-2"),
                    connect_clicked => ComponentInput::Internal(ComponentInputInternal::SetHeadingLevel(2)),
                },
                gtk::Button {
                    set_icon_name: relm4_icons::icon_names::TEXT_HEADER_3_LINES_CARET_REGULAR,
                    set_tooltip: &lang::lookup("rich-text-heading-3"),
                    connect_clicked => ComponentInput::Internal(ComponentInputInternal::SetHeadingLevel(3)),
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 4,
                    set_margin_start: 8,
                    #[watch]
                    set_visible: model.parse_warning,

                    gtk::Image {
                        set_icon_name: Some(relm4_icons::icon_names::WARNING),
                    },
                    gtk::Label {
                        set_text: &lang::lookup("rich-text-parsing-failure"),
                    },
                },
            },
            // Text
            #[name = "frame"]
            gtk::Frame {
                gtk::ScrolledWindow {
                    set_height_request: 200,
                    set_hexpand: true,

                    #[name = "text_view"]
                    gtk::TextView {
                        set_left_margin: 8,
                        set_right_margin: 8,
                        set_top_margin: 8,
                        set_bottom_margin: 8,
                        set_wrap_mode: gtk::WrapMode::Word,

                        #[name = "text_buffer"]
                        #[wrap(Some)]
                        set_buffer = &gtk::TextBuffer {
                            set_text: &init.text,
                            connect_changed => ComponentInput::Internal(ComponentInputInternal::TextChanged) @signal_text_changed,
                        },

                        add_controller = gtk::ShortcutController {
                            add_shortcut = gtk::Shortcut {
                                #[wrap(Some)]
                                set_trigger = gtk::ShortcutTrigger::parse_string("<primary>B").unwrap(),
                                #[wrap(Some)]
                                set_action = gtk::ShortcutAction::parse_string("action(rich-text-editor.bold)").unwrap(),
                            },
                            add_shortcut = gtk::Shortcut {
                                #[wrap(Some)]
                                set_trigger = gtk::ShortcutTrigger::parse_string("<primary>I").unwrap(),
                                #[wrap(Some)]
                                set_action = gtk::ShortcutAction::parse_string("action(rich-text-editor.italic)").unwrap(),
                            },
                        },
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
        let mut model = ComponentModel {
            parse_warning: false,
        };
        let widgets = view_output!();

        // Initial previewing
        model.update_preview(
            &widgets
                .text_buffer
                .text(
                    &widgets.text_buffer.start_iter(),
                    &widgets.text_buffer.end_iter(),
                    false,
                )
                .to_string(),
            &widgets.text_buffer,
            &widgets.signal_text_changed,
            &widgets.frame,
        );

        // Register accelerators
        let mut group = RelmActionGroup::<RichTextEditorActionGroup>::new();
        let action_bold: RelmAction<BoldAction> = {
            let tv = widgets.text_view.clone();
            let buf = widgets.text_buffer.clone();
            RelmAction::new_stateless(move |_| {
                add_angelmark_tokens(&tv, &buf, "**");
            })
        };
        group.add_action(action_bold);
        let action_italic: RelmAction<ItalicAction> = {
            let tv = widgets.text_view.clone();
            let buf = widgets.text_buffer.clone();
            RelmAction::new_stateless(move |_| {
                add_angelmark_tokens(&tv, &buf, "_");
            })
        };
        group.add_action(action_italic);
        group.register_for_widget(&widgets.text_view);

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
                self.update_preview(
                    &new_text,
                    &widgets.text_buffer,
                    &widgets.signal_text_changed,
                    &widgets.frame,
                );
                let _ = sender.output(ComponentOutput::TextChanged { new_text });
            }
            ComponentInput::Internal(ComponentInputInternal::AddTokens(token)) => {
                add_angelmark_tokens(&widgets.text_view, &widgets.text_buffer, token);
            }
            ComponentInput::Internal(ComponentInputInternal::SetHeadingLevel(level)) => {
                set_angelmark_heading_level(&widgets.text_view, &widgets.text_buffer, level);
            }
        }
    }
}

impl ComponentModel {
    fn update_preview(
        &mut self,
        text: &String,
        buffer: &gtk::TextBuffer,
        signal: &gtk::glib::signal::SignalHandlerId,
        frame: &gtk::Frame,
    ) {
        buffer.block_signal(signal);
        if let Ok(lines) = parse_angelmark(text) {
            let cursor_pos = buffer.cursor_position();

            for line in lines {
                let (start, end) = *line.span().span();
                // Remove plain text
                #[allow(clippy::cast_possible_wrap)]
                buffer.delete(
                    &mut buffer.iter_at_offset(start as i32),
                    &mut buffer.iter_at_offset(end as i32),
                );

                // Determine formatting
                let markup = angelmark_to_pango(&line);
                tracing::trace!("line: {line:?}");
                tracing::trace!("pango: {markup:?}");

                // Reinsert marked up text
                #[allow(clippy::cast_possible_wrap)]
                buffer.insert_markup(&mut buffer.iter_at_offset(start as i32), &markup);
            }

            buffer.place_cursor(&buffer.iter_at_offset(cursor_pos));
            frame.remove_css_class("warning");
            self.parse_warning = true;
        } else {
            frame.add_css_class("warning");
            self.parse_warning = false;
        }

        buffer.unblock_signal(signal);
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
