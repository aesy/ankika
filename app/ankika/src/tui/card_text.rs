use html2text::Colour;
use html2text::render::{RichAnnotation, RichDecorator, TextDecorator};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use ankika_core::review::{Side, Sound, SoundRef, Sounds};

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
    Line(CardLine),
    /// An image's `src`, drawn below the text it appeared in.
    Image(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct CardLine {
    pub line: Line<'static>,
    pub sounds: Vec<SoundSpan>,
}

/// Where a sound's placeholder sits in its line, for clicking it.
#[derive(Debug, PartialEq, Eq)]
pub struct SoundSpan {
    pub sound: SoundRef,
    /// Columns from the start of the line.
    pub start: u16,
    pub width: u16,
}

/// Link target for sound placeholders; never shown, since links have no footnotes.
const SOUND_LINK: &str = "ankika-sound:";

/// Renders card HTML as styled lines wrapped to `width`, with images split out and sound
/// references shown as placeholders, the one in `playing` highlighted.
pub fn render(
    html: &str,
    width: u16,
    theme: Theme,
    sounds: &Sounds,
    playing: Option<SoundRef>,
) -> Vec<CardBlock> {
    let config = html2text::config::with_decorator(CardDecorator(RichDecorator::new()))
        .link_footnotes(false)
        // Inline `style` attributes; the card's own stylesheet is never passed in
        .use_doc_css()
        .add_css(CARD_CSS);
    let html = MIXED_CASE_CLASSES.iter().fold(
        with_rule_markers(&with_image_alt(&with_sound_links(html, sounds))),
        |html, (from, to)| html.replace(from, to),
    );
    let lines = config
        .and_then(|config| config.lines_from_read(html.as_bytes(), usize::from(width.max(1))));

    match lines {
        Ok(lines) => {
            let mut blocks = Vec::new();

            for line in &lines {
                let mut images = Vec::new();
                let mut sound_spans = Vec::new();
                let mut column: u16 = 0;
                let mut spans = Vec::new();

                for string in line.tagged_strings() {
                    let mut image = None;
                    let mut sound = None;

                    for annotation in &string.tag {
                        match annotation {
                            RichAnnotation::Image(src) => image = Some(src.clone()),
                            RichAnnotation::Link(url) => sound = parse_sound_link(url),
                            _ => {},
                        }
                    }

                    if let Some(src) = image {
                        images.push(src);
                        continue;
                    }

                    let span_width =
                        u16::try_from(Span::raw(string.s.as_str()).width()).unwrap_or(u16::MAX);
                    let style = sound.map_or_else(
                        || style(&string.tag, theme),
                        |sound| {
                            sound_spans.push(SoundSpan {
                                sound,
                                start: column,
                                width: span_width,
                            });

                            sound_style(Some(sound) == playing, theme)
                        },
                    );

                    spans.push(Span::styled(string.s.clone(), style));
                    column = column.saturating_add(span_width);
                }

                let line = Line::from(spans);
                let text = line.to_string();

                if text.trim() == RULE_MARKER {
                    blocks.push(CardBlock::Line(CardLine {
                        line: Line::styled("─".repeat(usize::from(width)), theme.border),
                        sounds: Vec::new(),
                    }));
                } else if images.is_empty() || !text.trim().is_empty() {
                    blocks.push(CardBlock::Line(CardLine {
                        line,
                        sounds: sound_spans,
                    }));
                }

                blocks.extend(images.into_iter().map(CardBlock::Image));
            }

            blocks
        },
        Err(error) => {
            tracing::warn!(?error, "Failed to render card HTML");
            html.lines()
                .map(|line| {
                    CardBlock::Line(CardLine {
                        line: Line::raw(line.to_owned()),
                        sounds: Vec::new(),
                    })
                })
                .collect()
        },
    }
}

/// Turns Anki's sound references into placeholders; files become links so they can be
/// highlighted and clicked.
fn with_sound_links(html: &str, sounds: &Sounds) -> String {
    let mut html = html.to_owned();

    for side in [Side::Question, Side::Answer] {
        for (index, sound) in sounds.side(side).iter().enumerate() {
            let sound_ref = SoundRef { side, index };
            let placeholder = match sound {
                Sound::File(file) => format!(
                    r#"<a href="{SOUND_LINK}{}">[sound: {}]</a>"#,
                    link_target(sound_ref),
                    escape(file)
                ),
                Sound::Tts(text) => format!("[tts: {}]", escape(text)),
            };

            html = html.replace(&sound_ref.tag(), &placeholder);
        }
    }

    html
}

fn link_target(sound: SoundRef) -> String {
    let side = match sound.side {
        Side::Question => 'q',
        Side::Answer => 'a',
    };

    format!("{side}{}", sound.index)
}

fn parse_sound_link(url: &str) -> Option<SoundRef> {
    let target = url.strip_prefix(SOUND_LINK)?;
    let side = match target.chars().next()? {
        'q' => Side::Question,
        'a' => Side::Answer,
        _ => return None,
    };
    let index = target.get(1..)?.parse().ok()?;

    Some(SoundRef { side, index })
}

const fn sound_style(playing: bool, theme: Theme) -> Style {
    if playing {
        Style::new().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(theme.muted)
    }
}

pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
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
    use crate::config::ThemeName;

    fn no_sounds() -> Sounds {
        Sounds {
            question: Vec::new(),
            answer: Vec::new(),
            autoplay: true,
            replay_question: true,
        }
    }

    fn render(html: &str, width: u16, theme: Theme) -> Vec<CardBlock> {
        super::render(html, width, theme, &no_sounds(), None)
    }

    fn lines(blocks: Vec<CardBlock>) -> Vec<Line<'static>> {
        blocks
            .into_iter()
            .filter_map(|block| match block {
                CardBlock::Line(line) => Some(line.line),
                CardBlock::Image(_) => None,
            })
            .collect()
    }

    fn text(lines: &[Line<'_>]) -> Vec<String> {
        lines.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn renders_line_breaks_and_bold() {
        let lines = lines(render(
            "<b>front</b><br>back",
            40,
            Theme::new(ThemeName::Dark),
        ));

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
        let lines = lines(render(
            "front<hr id=answer>back",
            10,
            Theme::new(ThemeName::Dark),
        ));

        assert_eq!(text(&lines), vec!["front", &"─".repeat(10), "back"]);
    }

    #[test]
    fn applies_inline_colors() {
        let lines = lines(render(
            r#"<span style="color: #ff8800">a</span> <font color="red">b</font>"#,
            40,
            Theme::new(ThemeName::Dark),
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
            Theme::new(ThemeName::Dark),
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
    fn marks_sound_placeholders() {
        let sounds = Sounds {
            question: vec![Sound::File("a.mp3".to_owned())],
            ..no_sounds()
        };
        let playing = SoundRef {
            side: Side::Question,
            index: 0,
        };
        let blocks = super::render(
            "Hi [anki:play:q:0]",
            40,
            Theme::new(ThemeName::Dark),
            &sounds,
            Some(playing),
        );
        let CardBlock::Line(line) = &blocks[0] else {
            panic!("expected a line");
        };

        assert_eq!(line.line.to_string(), "Hi [sound: a.mp3]");
        assert_eq!(
            line.sounds,
            vec![SoundSpan {
                sound: playing,
                start: 3,
                width: 14,
            }]
        );
    }

    #[test]
    fn splits_out_images() {
        let blocks = render(
            r#"Cat: <img src="cat.jpg"><br><img src="dog.png">"#,
            40,
            Theme::new(ThemeName::Dark),
        );

        assert_eq!(
            blocks,
            vec![
                CardBlock::Line(CardLine {
                    line: Line::from("Cat: "),
                    sounds: Vec::new(),
                }),
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
            Theme::new(ThemeName::Dark),
        ));
        let cloze = lines[0].spans.iter().find(|span| span.content == "[...]");

        assert_eq!(
            cloze.and_then(|span| span.style.fg),
            Some(Color::Rgb(0x82, 0xaa, 0xff))
        );
    }
}
