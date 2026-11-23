/// How many suggestions show under the input.
const MAX_COMPLETIONS: usize = 5;

/// Space-separated tags, completed from the collection's tags.
pub struct TagInput {
    known: Vec<String>,
    text: String,
    completions: Vec<String>,
    completion: usize,
}

impl TagInput {
    pub const fn new(known: Vec<String>) -> Self {
        Self {
            known,
            text: String::new(),
            completions: Vec::new(),
            completion: 0,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn tags(&self) -> Vec<String> {
        self.text.split_whitespace().map(str::to_owned).collect()
    }

    pub fn completions(&self) -> &[String] {
        &self.completions
    }

    pub const fn completion(&self) -> usize {
        self.completion
    }

    pub fn push(&mut self, character: char) {
        self.text.push(character);
        self.update_completions();
    }

    pub fn pop(&mut self) {
        self.text.pop();
        self.update_completions();
    }

    pub const fn select_completion(&mut self, index: usize) {
        if index < self.completions.len() {
            self.completion = index;
        }
    }

    pub const fn previous_completion(&mut self) {
        self.completion = self.completion.saturating_sub(1);
    }

    pub const fn next_completion(&mut self) {
        self.select_completion(self.completion.saturating_add(1));
    }

    /// Replaces the word being typed with the selected suggestion. Returns whether there
    /// was one.
    pub fn complete(&mut self) -> bool {
        let Some(tag) = self.completions.get(self.completion).cloned() else {
            return false;
        };
        let start = self
            .text
            .rfind(char::is_whitespace)
            .map_or(0, |index| index + 1);

        self.text.truncate(start);
        self.text.push_str(&tag);
        self.text.push(' ');
        self.update_completions();

        true
    }

    /// Tags starting with the word being typed, ignoring case and the tags already there.
    fn update_completions(&mut self) {
        let typing = !self.text.ends_with(char::is_whitespace);
        let mut words: Vec<&str> = self.text.split_whitespace().collect();
        let word = if typing { words.pop() } else { None };

        self.completions = word.map_or_else(Vec::new, |word| {
            let word = word.to_lowercase();

            self.known
                .iter()
                .filter(|tag| tag.to_lowercase().starts_with(&word))
                .filter(|tag| !words.iter().any(|done| done.eq_ignore_ascii_case(tag)))
                .take(MAX_COMPLETIONS)
                .cloned()
                .collect()
        });
        self.completion = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(text: &str) -> TagInput {
        let mut input = TagInput::new(vec![
            "verbs".to_owned(),
            "Vocab".to_owned(),
            "nouns".to_owned(),
        ]);
        text.chars().for_each(|character| input.push(character));
        input
    }

    #[test]
    fn completes_the_word_being_typed() {
        let mut input = input("nouns v");

        assert_eq!(input.completions(), ["verbs", "Vocab"]);

        input.next_completion();
        assert!(input.complete());
        assert_eq!(input.text(), "nouns Vocab ");
        assert_eq!(input.completions(), &[] as &[String]);
    }

    #[test]
    fn skips_tags_already_typed() {
        assert_eq!(input("verbs v").completions(), ["Vocab"]);
    }
}
