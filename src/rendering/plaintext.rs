use pulldown_cmark::{Event, Parser, Tag, TagEnd};

enum ListState {
    Unordered,
    Ordered(u64),
}

struct PlaintextRenderer {
    output: String,
    list_stack: Vec<ListState>,
    list_item_depth: usize,
    current_item_marker_range: Option<(usize, usize)>,
    blockquote_depth: usize,
    link_stack: Vec<(String, usize)>,
}

impl PlaintextRenderer {
    fn new() -> Self {
        Self {
            output: String::new(),
            list_stack: Vec::new(),
            list_item_depth: 0,
            current_item_marker_range: None,
            blockquote_depth: 0,
            link_stack: Vec::new(),
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
        self.push_str(&marker);
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
            self.output.replace_range(start..end, marker);
            self.current_item_marker_range = Some((start, start + marker.len()));
        }
    }

    fn soft_break(&mut self) {
        if self.blockquote_depth > 0 {
            self.push_str("\n> ");
        } else {
            self.push_str("\n");
        }
    }

    fn begin_link(&mut self, destination: &str) {
        self.link_stack
            .push((destination.to_string(), self.output.len()));
    }

    fn end_link(&mut self) {
        if let Some((destination, start)) = self.link_stack.pop() {
            if self.output.len() > start {
                self.push_str(": ");
            }
            self.push_str(&destination);
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
    let mut renderer = PlaintextRenderer::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {}
                Tag::Heading { .. } => renderer.ensure_blank_line(),
                Tag::BlockQuote(_) => {
                    renderer.ensure_blank_line();
                    renderer.blockquote_depth += 1;
                    renderer.push_str("> ");
                }
                Tag::CodeBlock(_) => renderer.ensure_blank_line(),
                Tag::List(start) => match start {
                    Some(start) => renderer.list_stack.push(ListState::Ordered(start)),
                    None => renderer.list_stack.push(ListState::Unordered),
                },
                Tag::Item => renderer.start_list_item(),
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
                TagEnd::Heading(_)
                | TagEnd::BlockQuote(_)
                | TagEnd::CodeBlock
                | TagEnd::List(_) => {
                    if matches!(tag, TagEnd::BlockQuote(_)) {
                        renderer.blockquote_depth = renderer.blockquote_depth.saturating_sub(1);
                    }
                    if matches!(tag, TagEnd::List(_)) {
                        renderer.list_stack.pop();
                    }
                    renderer.ensure_blank_line();
                }
                TagEnd::Item => renderer.end_list_item(),
                TagEnd::Link | TagEnd::Image => renderer.end_link(),
                _ => {}
            },
            Event::Text(text) | Event::Code(text) => renderer.push_str(text.as_ref()),
            Event::Html(html) | Event::InlineHtml(html) => renderer.push_str(html.as_ref()),
            Event::SoftBreak | Event::HardBreak => renderer.soft_break(),
            Event::Rule => {
                renderer.ensure_blank_line();
                renderer.push_str("------------------------");
                renderer.ensure_blank_line();
            }
            Event::TaskListMarker(checked) => {
                renderer.replace_current_item_marker(if checked { "[x] " } else { "[ ] " });
            }
            _ => {}
        }
    }

    renderer.finish()
}
