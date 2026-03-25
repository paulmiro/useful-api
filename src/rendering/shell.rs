use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

const ANSI_RESET: &str = "\x1b[0m";
const OSC: &str = "\x1b]";
const ST: &str = "\x1b\\";

#[derive(Clone, Copy, PartialEq, Eq)]
enum TextStyle {
    Emphasis,
    Strong,
    Strikethrough,
    Code,
    CodeInfo,
    Heading,
    Link,
    Quote,
    ListMarker,
}

impl TextStyle {
    fn ansi(self) -> &'static str {
        match self {
            TextStyle::Emphasis => "\x1b[3;38;5;213m",
            TextStyle::Strong => "\x1b[1;38;5;231m",
            TextStyle::Strikethrough => "\x1b[9;38;5;203m",
            TextStyle::Code => "\x1b[38;5;220m",
            TextStyle::CodeInfo => "\x1b[38;5;245m",
            TextStyle::Heading => "\x1b[1;38;5;51m",
            TextStyle::Link => "\x1b[1;4;38;5;117m",
            TextStyle::Quote => "\x1b[38;5;150m",
            TextStyle::ListMarker => "\x1b[1;38;5;81m",
        }
    }
}

enum ListState {
    Unordered,
    Ordered(u64),
}

struct TerminalRenderer {
    output: String,
    styles: Vec<TextStyle>,
    list_stack: Vec<ListState>,
    blockquote_depth: usize,
    list_item_depth: usize,
    current_item_marker_range: Option<(usize, usize)>,
}

impl TerminalRenderer {
    fn new() -> Self {
        Self {
            output: String::new(),
            styles: Vec::new(),
            list_stack: Vec::new(),
            blockquote_depth: 0,
            list_item_depth: 0,
            current_item_marker_range: None,
        }
    }

    fn push_str(&mut self, text: &str) {
        self.output.push_str(text);
    }

    fn starts_line(&self) -> bool {
        self.output.is_empty() || self.output.ends_with('\n')
    }

    fn ensure_newline(&mut self) {
        if !self.starts_line() {
            self.output.push('\n');
        }
    }

    fn ensure_blank_line(&mut self) {
        if self.output.is_empty() {
            return;
        }

        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        if !self.output.ends_with("\n\n") {
            self.output.push('\n');
        }
    }

    fn push_style(&mut self, style: TextStyle) {
        self.styles.push(style);
        self.output.push_str(style.ansi());
    }

    fn pop_style(&mut self, style: TextStyle) {
        if let Some(index) = self.styles.iter().rposition(|active| *active == style) {
            self.styles.remove(index);
        }
        self.output.push_str(ANSI_RESET);
        for active in &self.styles {
            self.output.push_str(active.ansi());
        }
    }

    fn begin_link(&mut self, destination: &str) {
        self.output.push_str(&format!("{OSC}8;;{destination}{ST}"));
        self.push_style(TextStyle::Link);
    }

    fn end_link(&mut self) {
        self.pop_style(TextStyle::Link);
        self.output.push_str(&format!("{OSC}8;;{ST}"));
    }

    fn start_list_item(&mut self) {
        self.ensure_newline();

        let indent = "  ".repeat(self.list_stack.len().saturating_sub(1));
        self.push_str(&indent);

        let marker = match self.list_stack.last_mut() {
            Some(ListState::Unordered) | None => "- ".to_string(),
            Some(ListState::Ordered(next)) => {
                let marker = format!("{next}. ");
                *next += 1;
                marker
            }
        };

        let marker_start = self.output.len();
        self.push_style(TextStyle::ListMarker);
        self.push_str(&marker);
        self.pop_style(TextStyle::ListMarker);
        self.current_item_marker_range = Some((marker_start, self.output.len()));
        self.list_item_depth += 1;
    }

    fn end_list_item(&mut self) {
        self.list_item_depth = self.list_item_depth.saturating_sub(1);
        self.current_item_marker_range = None;
        self.ensure_newline();
    }

    fn replace_current_item_marker(&mut self, marker: &str) {
        if let Some((start, end)) = self.current_item_marker_range.take() {
            let replacement = format!("{}{}{}", TextStyle::ListMarker.ansi(), marker, ANSI_RESET);
            self.output.replace_range(start..end, &replacement);
            self.current_item_marker_range = Some((start, start + replacement.len()));
        }
    }

    fn soft_break(&mut self) {
        if self.blockquote_depth > 0 {
            self.push_str("\n> ");
        } else {
            self.push_str("\n");
        }
    }

    fn finish(mut self) -> String {
        while self.output.ends_with("\n\n") {
            self.output.pop();
        }
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.output
    }
}

pub fn render(message: &str) -> String {
    let parser = Parser::new_ext(message, super::markdown_options());
    let mut renderer = TerminalRenderer::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {}
                Tag::Heading { .. } => {
                    renderer.ensure_blank_line();
                    renderer.push_style(TextStyle::Heading);
                }
                Tag::BlockQuote(_) => {
                    renderer.ensure_blank_line();
                    renderer.blockquote_depth += 1;
                    renderer.push_style(TextStyle::Quote);
                    renderer.push_str("> ");
                }
                Tag::CodeBlock(kind) => {
                    renderer.ensure_blank_line();
                    renderer.push_style(TextStyle::Code);
                    if let CodeBlockKind::Fenced(language) = kind {
                        if !language.is_empty() {
                            renderer.push_style(TextStyle::CodeInfo);
                            renderer.push_str(&format!("[{language}]"));
                            renderer.pop_style(TextStyle::CodeInfo);
                            renderer.push_str("\n");
                        }
                    }
                }
                Tag::List(start) => match start {
                    Some(start) => renderer.list_stack.push(ListState::Ordered(start)),
                    None => renderer.list_stack.push(ListState::Unordered),
                },
                Tag::Item => renderer.start_list_item(),
                Tag::Emphasis => renderer.push_style(TextStyle::Emphasis),
                Tag::Strong => renderer.push_style(TextStyle::Strong),
                Tag::Strikethrough => renderer.push_style(TextStyle::Strikethrough),
                Tag::Link { dest_url, .. } => renderer.begin_link(dest_url.as_ref()),
                Tag::Image { dest_url, .. } => renderer.begin_link(dest_url.as_ref()),
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => {
                    if renderer.list_item_depth == 0 && renderer.blockquote_depth == 0 {
                        renderer.ensure_blank_line();
                    }
                }
                TagEnd::Heading(_) => {
                    renderer.pop_style(TextStyle::Heading);
                    renderer.ensure_blank_line();
                }
                TagEnd::BlockQuote(_) => {
                    renderer.blockquote_depth = renderer.blockquote_depth.saturating_sub(1);
                    renderer.pop_style(TextStyle::Quote);
                    renderer.ensure_blank_line();
                }
                TagEnd::CodeBlock => {
                    renderer.pop_style(TextStyle::Code);
                    renderer.ensure_blank_line();
                }
                TagEnd::List(_) => {
                    renderer.list_stack.pop();
                    renderer.ensure_blank_line();
                }
                TagEnd::Item => renderer.end_list_item(),
                TagEnd::Emphasis => renderer.pop_style(TextStyle::Emphasis),
                TagEnd::Strong => renderer.pop_style(TextStyle::Strong),
                TagEnd::Strikethrough => renderer.pop_style(TextStyle::Strikethrough),
                TagEnd::Link | TagEnd::Image => renderer.end_link(),
                _ => {}
            },
            Event::Text(text) => renderer.push_str(text.as_ref()),
            Event::Code(code) => {
                renderer.push_style(TextStyle::Code);
                renderer.push_str(code.as_ref());
                renderer.pop_style(TextStyle::Code);
            }
            Event::Html(html) | Event::InlineHtml(html) => renderer.push_str(html.as_ref()),
            Event::SoftBreak | Event::HardBreak => renderer.soft_break(),
            Event::Rule => {
                renderer.ensure_blank_line();
                renderer.push_str("\x1b[38;5;240m────────────────────────\x1b[0m");
                renderer.ensure_blank_line();
            }
            Event::TaskListMarker(checked) => {
                renderer.replace_current_item_marker(if checked { "󰱒 " } else { "󰄱 " });
            }
            _ => {}
        }
    }

    renderer.finish()
}
