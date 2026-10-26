use anki::decks::DeckId;
use ankika_core::decks::DeckNode;

/// A visible deck with its nesting depth, in display order.
pub struct Row<'a> {
    pub deck: &'a DeckNode,
    pub depth: usize,
}

/// The deck tree as shown on the home screen, with the selected deck.
#[derive(Default)]
pub struct DeckList {
    decks: Vec<DeckNode>,
    selected: Option<DeckId>,
}

impl DeckList {
    /// Keeps the selection when that deck is still visible.
    pub fn set_decks(&mut self, decks: Vec<DeckNode>) {
        self.decks = decks;

        if !self.is_visible(self.selected) {
            self.selected = self.rows().first().map(|row| row.deck.id);
        }
    }

    pub fn rows(&self) -> Vec<Row<'_>> {
        let mut rows = Vec::new();
        push_rows(&self.decks, 0, &mut rows);
        rows
    }

    pub const fn selected(&self) -> Option<DeckId> {
        self.selected
    }

    /// Cards due today over all decks.
    pub fn due_count(&self) -> u32 {
        self.decks.iter().map(DeckNode::due_count).sum()
    }

    pub fn select(&mut self, id: DeckId) {
        if self.is_visible(Some(id)) {
            self.selected = Some(id);
        }
    }

    pub fn select_next(&mut self) {
        self.move_selection(|index, count| (index + 1).min(count - 1));
    }

    pub fn select_previous(&mut self) {
        self.move_selection(|index, _| index.saturating_sub(1));
    }

    pub fn select_first(&mut self) {
        self.move_selection(|_, _| 0);
    }

    pub fn select_last(&mut self) {
        self.move_selection(|_, count| count - 1);
    }

    /// Expands the selected deck, or moves into it when already expanded.
    /// Returns the deck whose collapsed state changed.
    pub fn expand(&mut self) -> Option<(DeckId, bool)> {
        let deck = self.selected_deck()?;

        if deck.children.is_empty() {
            return None;
        }

        if deck.collapsed {
            return self.set_collapsed(deck.id, false);
        }

        self.selected = deck.children.first().map(|child| child.id);
        None
    }

    /// Collapses the selected deck, or moves to its parent when there is nothing to collapse.
    pub fn collapse(&mut self) -> Option<(DeckId, bool)> {
        let deck = self.selected_deck()?;

        if !deck.children.is_empty() && !deck.collapsed {
            return self.set_collapsed(deck.id, true);
        }

        if let Some(parent) = find_parent(&self.decks, deck.id) {
            self.selected = Some(parent.id);
        }

        None
    }

    /// Selects the deck when collapsing it hides the selection.
    pub fn toggle(&mut self, id: DeckId) -> Option<(DeckId, bool)> {
        let collapsed = find(&self.decks, id)?.collapsed;
        let change = self.set_collapsed(id, !collapsed);

        if !self.is_visible(self.selected) {
            self.selected = Some(id);
        }

        change
    }

    fn set_collapsed(&mut self, id: DeckId, collapsed: bool) -> Option<(DeckId, bool)> {
        find_mut(&mut self.decks, id)?.collapsed = collapsed;

        Some((id, collapsed))
    }

    fn selected_deck(&self) -> Option<&DeckNode> {
        find(&self.decks, self.selected?)
    }

    fn is_visible(&self, id: Option<DeckId>) -> bool {
        id.is_some_and(|id| self.rows().iter().any(|row| row.deck.id == id))
    }

    fn move_selection(&mut self, target: impl FnOnce(usize, usize) -> usize) {
        let rows = self.rows();

        if rows.is_empty() {
            return;
        }

        let index = rows
            .iter()
            .position(|row| Some(row.deck.id) == self.selected)
            .unwrap_or(0);

        self.selected = Some(rows[target(index, rows.len())].deck.id);
    }
}

fn push_rows<'a>(decks: &'a [DeckNode], depth: usize, rows: &mut Vec<Row<'a>>) {
    for deck in decks {
        rows.push(Row { deck, depth });

        if !deck.collapsed {
            push_rows(&deck.children, depth + 1, rows);
        }
    }
}

fn find(decks: &[DeckNode], id: DeckId) -> Option<&DeckNode> {
    decks.iter().find_map(|deck| {
        if deck.id == id {
            Some(deck)
        } else {
            find(&deck.children, id)
        }
    })
}

fn find_mut(decks: &mut [DeckNode], id: DeckId) -> Option<&mut DeckNode> {
    decks.iter_mut().find_map(|deck| {
        if deck.id == id {
            Some(deck)
        } else {
            find_mut(&mut deck.children, id)
        }
    })
}

fn find_parent(decks: &[DeckNode], id: DeckId) -> Option<&DeckNode> {
    decks.iter().find_map(|deck| {
        if deck.children.iter().any(|child| child.id == id) {
            Some(deck)
        } else {
            find_parent(&deck.children, id)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deck(id: i64, children: Vec<DeckNode>) -> DeckNode {
        DeckNode {
            id: DeckId(id),
            name: id.to_string(),
            collapsed: false,
            filtered: false,
            new_count: 1,
            learn_count: 0,
            review_count: 2,
            children,
        }
    }

    fn list() -> DeckList {
        let mut list = DeckList::default();
        list.set_decks(vec![
            deck(1, vec![deck(2, vec![]), deck(3, vec![])]),
            deck(4, vec![]),
        ]);
        list
    }

    fn visible(list: &DeckList) -> Vec<i64> {
        list.rows().iter().map(|row| row.deck.id.0).collect()
    }

    #[test]
    fn selects_first_deck() {
        assert_eq!(list().selected(), Some(DeckId(1)));
    }

    #[test]
    fn collapse_hides_children_and_moves_to_parent() {
        let mut list = list();
        list.select(DeckId(3));

        assert_eq!(list.collapse(), None);
        assert_eq!(list.selected(), Some(DeckId(1)));
        assert_eq!(list.collapse(), Some((DeckId(1), true)));
        assert_eq!(visible(&list), vec![1, 4]);
    }

    #[test]
    fn expand_then_enters_children() {
        let mut list = list();
        list.toggle(DeckId(1));

        assert_eq!(list.expand(), Some((DeckId(1), false)));
        assert_eq!(list.expand(), None);
        assert_eq!(list.selected(), Some(DeckId(2)));
    }

    #[test]
    fn toggling_hides_selection_selects_deck() {
        let mut list = list();
        list.select(DeckId(2));
        list.toggle(DeckId(1));

        assert_eq!(list.selected(), Some(DeckId(1)));
    }

    #[test]
    fn selection_stops_at_ends() {
        let mut list = list();
        list.select_previous();
        assert_eq!(list.selected(), Some(DeckId(1)));

        list.select_last();
        list.select_next();
        assert_eq!(list.selected(), Some(DeckId(4)));
    }

    #[test]
    fn due_count_sums_top_level_decks() {
        assert_eq!(list().due_count(), 6);
    }
}
