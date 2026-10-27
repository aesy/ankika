use html2text::Colour;
use html2text::render::{RichAnnotation, RichDecorator, TextDecorator};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::tui::theme::Theme;

/// Stands in for the card's own CSS, which is ignored. Colors match Anki's reviewer.
/// html2text lowercases selectors, so the classes are lowercase here and in the HTML.
const CARD_CSS: &str = "
    .cloze { color: #82aaff; }
    .typegood { color: #4ade80; }
    .typebad { color: #f87171; }
    .typemissed { color: #9ca3af; }
";

/// Anki's type-in comparison classes, which html2text would otherwise never match.
const MIXED_CASE_CLASSES: [(&str, &str); 3] = [
    ("class=typeGood", "class=typegood"),
    ("class=typeBad", "class=typebad"),
    ("class=typeMissed", "class=typemissed"),
];

/// A piece of a rendered card, top to bottom.
#[derive(Debug, PartialEq, Eq)]
pub enum CardBlock {
    Line(Line<'static>),
    /// An image's `src`, drawn below the text it appeared in.
    Image(String),
}

/// Renders card HTML as styled lines wrapped to `width`, with images split out.
pub fn render(html: &str, width: u16, theme: Theme) -> Vec<CardBlock> {
    let config = html2text::config::with_decorator(CardDecorator(RichDecorator::new()))
        .link_footnotes(false)
        // Inline `style` attributes; the card's own stylesheet is never passed in
        .use_doc_css()
        .add_css(CARD_CSS);
    let html = MIXED_CASE_CLASSES.iter().fold(
        with_rule_markers(&with_image_alt(html)),
        |html, (from, to)| html.replace(from, to),
    );
    let lines = config
        .and_then(|config| config.lines_from_read(html.as_bytes(), usize::from(width.max(1))));

    match lines {
        Ok(lines) => {
            let mut blocks = Vec::new();

            for line in &lines {
                let mut images = Vec::new();
                let spans: Line<'static> = line
                    .tagged_strings()
                    .filter_map(|string| {
                        let image = string.tag.iter().find_map(|annotation| match annotation {
                            RichAnnotation::Image(src) => Some(src.clone()),
                            _ => None,
                        });

                        if let Some(src) = image {
                            images.push(src);
                            return None;
                        }

                        Some(Span::styled(string.s.clone(), style(&string.tag, theme)))
                    })
                    .collect();
                let text = spans.to_string();

                if text.trim() == RULE_MARKER {
                    blocks.push(CardBlock::Line(Line::styled(
                        "─".repeat(usize::from(width)),
                        theme.border,
                    )));
                } else if images.is_empty() || !text.trim().is_empty() {
                    blocks.push(CardBlock::Line(spans));
                }

                blocks.extend(images.into_iter().map(CardBlock::Image));
            }

            blocks
        },
        Err(error) => {
            tracing::warn!(?error, "Failed to render card HTML");
            html.lines()
                .map(|line| CardBlock::Line(Line::raw(line.to_owned())))
                .collect()
        },
    }
}

/// Private-use character standing in for `<hr>`, which html2text ignores but Anki uses to
/// separate the question from the answer.
const RULE_MARKER: &str = "\u{e000}";

fn with_rule_markers(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(start) = find_ignore_case(rest, "<hr") {
        let Some(length) = rest[start..].find('>') else {
            break;
        };

        result.push_str(&rest[..start]);
        result.push_str("<div>");
        result.push_str(RULE_MARKER);
        result.push_str("</div>");
        rest = &rest[start + length + 1..];
    }

    result.push_str(rest);
    result
}

/// html2text drops images without alt text, which Anki's media images usually lack. The
/// decorator shows the file name instead, so the value never appears.
fn with_image_alt(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(index) = find_ignore_case(rest, "<img") {
        let end = index + "<img".len();

        result.push_str(&rest[..end]);
        result.push_str(r#" alt="image""#);
        rest = &rest[end..];
    }

    result.push_str(rest);
    result
}

fn find_ignore_case(haystack: &str, needle: &str) -> Option<usize> {
    haystack
        .as_bytes()
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

fn style(annotations: &[RichAnnotation], theme: Theme) -> Style {
    annotations
        .iter()
        .fold(Style::new(), |style, annotation| match annotation {
            RichAnnotation::Strong => style.add_modifier(Modifier::BOLD),
            RichAnnotation::Emphasis => style.add_modifier(Modifier::ITALIC),
            RichAnnotation::Strikeout => style.add_modifier(Modifier::CROSSED_OUT),
            RichAnnotation::Link(_) => style.fg(theme.accent).add_modifier(Modifier::UNDERLINED),
            RichAnnotation::Image(_) | RichAnnotation::Code => style.fg(theme.muted),
            RichAnnotation::Colour(colour) => style.fg(rgb(*colour)),
            RichAnnotation::BgColour(colour) => style.bg(rgb(*colour)),
            // Default, Preformat and any annotation a newer html2text adds
            _ => style,
        })
}

const fn rgb(colour: Colour) -> Color {
    Color::Rgb(colour.r, colour.g, colour.b)
}

/// The rich decorator, except that images show as a placeholder with their file name.
#[derive(Clone)]
struct CardDecorator(RichDecorator);

impl TextDecorator for CardDecorator {
    type Annotation = RichAnnotation;

    fn decorate_link_start(&mut self, url: &str) -> (String, Self::Annotation) {
        self.0.decorate_link_start(url)
    }

    fn decorate_link_end(&mut self) -> String {
        self.0.decorate_link_end()
    }

    fn decorate_em_start(&self) -> (String, Self::Annotation) {
        self.0.decorate_em_start()
    }

    fn decorate_em_end(&self) -> String {
        self.0.decorate_em_end()
    }

    fn decorate_strong_start(&self) -> (String, Self::Annotation) {
        self.0.decorate_strong_start()
    }

    fn decorate_strong_end(&self) -> String {
        self.0.decorate_strong_end()
    }

    fn decorate_strikeout_start(&self) -> (String, Self::Annotation) {
        self.0.decorate_strikeout_start()
    }

    fn decorate_strikeout_end(&self) -> String {
        self.0.decorate_strikeout_end()
    }

    fn decorate_code_start(&self) -> (String, Self::Annotation) {
        self.0.decorate_code_start()
    }

    fn decorate_code_end(&self) -> String {
        self.0.decorate_code_end()
    }

    fn decorate_preformat_first(&self) -> Self::Annotation {
        self.0.decorate_preformat_first()
    }

    fn decorate_preformat_cont(&self) -> Self::Annotation {
        self.0.decorate_preformat_cont()
    }

    fn decorate_image(&mut self, src: &str, _title: &str) -> (String, Self::Annotation) {
        (
            format!("[image: {src}]"),
            RichAnnotation::Image(src.to_owned()),
        )
    }

    fn header_prefix(&self, level: usize) -> String {
        self.0.header_prefix(level)
    }

    fn quote_prefix(&self) -> String {
        self.0.quote_prefix()
    }

    fn unordered_item_prefix(&self) -> String {
        "• ".to_owned()
    }

    fn ordered_item_prefix(&self, i: i64) -> String {
        self.0.ordered_item_prefix(i)
    }

    fn make_subblock_decorator(&self) -> Self {
        Self(self.0.make_subblock_decorator())
    }

    fn push_colour(&mut self, colour: Colour) -> Option<Self::Annotation> {
        self.0.push_colour(colour)
    }

    fn pop_colour(&mut self) -> bool {
        self.0.pop_colour()
    }

    fn push_bgcolour(&mut self, colour: Colour) -> Option<Self::Annotation> {
        self.0.push_bgcolour(colour)
    }

    fn pop_bgcolour(&mut self) -> bool {
        self.0.pop_bgcolour()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(blocks: Vec<CardBlock>) -> Vec<Line<'static>> {
        blocks
            .into_iter()
            .filter_map(|block| match block {
                CardBlock::Line(line) => Some(line),
                CardBlock::Image(_) => None,
            })
            .collect()
    }

    fn text(lines: &[Line<'_>]) -> Vec<String> {
        lines.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn renders_line_breaks_and_bold() {
        let lines = lines(render("<b>front</b><br>back", 40, Theme::detect()));

        assert_eq!(text(&lines), vec!["front", "back"]);
        assert!(
            lines[0].spans[0]
                .style
                .add_modifier
                .contains(Modifier::BOLD)
        );
    }

    #[test]
    fn draws_rules() {
        let lines = lines(render("front<hr id=answer>back", 10, Theme::detect()));

        assert_eq!(text(&lines), vec!["front", &"─".repeat(10), "back"]);
    }

    #[test]
    fn applies_inline_colors() {
        let lines = lines(render(
            r#"<span style="color: #ff8800">a</span> <font color="red">b</font>"#,
            40,
            Theme::detect(),
        ));
        let colors: Vec<_> = lines[0]
            .spans
            .iter()
            .filter_map(|span| span.style.fg)
            .collect();

        assert_eq!(colors, vec![Color::Rgb(255, 136, 0), Color::Rgb(255, 0, 0)]);
    }

    #[test]
    fn colors_typed_answer_comparison() {
        let lines = lines(render(
            "<code id=typeans><span class=typeGood>a</span><span class=typeBad>b</span></code>",
            40,
            Theme::detect(),
        ));
        let colors: Vec<_> = lines[0]
            .spans
            .iter()
            .filter_map(|span| span.style.fg)
            .collect();

        assert_eq!(
            colors,
            vec![Color::Rgb(0x4a, 0xde, 0x80), Color::Rgb(0xf8, 0x71, 0x71)]
        );
    }

    #[test]
    fn splits_out_images() {
        let blocks = render(
            r#"Cat: <img src="cat.jpg"><br><img src="dog.png">"#,
            40,
            Theme::detect(),
        );

        assert_eq!(
            blocks,
            vec![
                CardBlock::Line(Line::from("Cat: ")),
                CardBlock::Image("cat.jpg".to_owned()),
                CardBlock::Image("dog.png".to_owned()),
            ]
        );
    }

    #[test]
    fn colors_clozes() {
        let lines = lines(render(
            r#"a <span class="cloze">[...]</span>"#,
            40,
            Theme::detect(),
        ));
        let cloze = lines[0].spans.iter().find(|span| span.content == "[...]");

        assert_eq!(
            cloze.and_then(|span| span.style.fg),
            Some(Color::Rgb(0x82, 0xaa, 0xff))
        );
    }
}
