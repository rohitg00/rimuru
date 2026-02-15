use super::handler::Action;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub action: Action,
}

impl Command {
    pub fn new(name: &str, description: &str, action: Action) -> Self {
        Self {
            name: name.to_string(),
            aliases: Vec::new(),
            description: description.to_string(),
            action,
        }
    }

    pub fn with_aliases(mut self, aliases: Vec<&str>) -> Self {
        self.aliases = aliases.into_iter().map(String::from).collect();
        self
    }

    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name.to_lowercase().starts_with(&query_lower)
            || self
                .aliases
                .iter()
                .any(|a| a.to_lowercase().starts_with(&query_lower))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandPaletteState {
    Hidden,
    Open,
    Executing,
}

pub struct CommandPalette {
    state: CommandPaletteState,
    input: String,
    cursor_position: usize,
    commands: Vec<Command>,
    filtered_commands: Vec<usize>,
    selected_index: usize,
    history: Vec<String>,
    history_index: Option<usize>,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        let commands = Self::default_commands();
        let filtered_commands = (0..commands.len()).collect();

        Self {
            state: CommandPaletteState::Hidden,
            input: String::new(),
            cursor_position: 0,
            commands,
            filtered_commands,
            selected_index: 0,
            history: Vec::new(),
            history_index: None,
        }
    }

    fn default_commands() -> Vec<Command> {
        vec![
            Command::new("quit", "Exit the application", Action::Quit)
                .with_aliases(vec!["q", "exit"]),
            Command::new("dashboard", "Go to dashboard view", Action::GoToView(0))
                .with_aliases(vec!["home", "d"]),
            Command::new("agents", "Go to agents view", Action::GoToView(1))
                .with_aliases(vec!["a"]),
            Command::new("sessions", "Go to sessions view", Action::GoToView(2))
                .with_aliases(vec!["s"]),
            Command::new("costs", "Go to costs view", Action::GoToView(3))
                .with_aliases(vec!["c", "cost"]),
            Command::new("metrics", "Go to metrics view", Action::GoToView(4))
                .with_aliases(vec!["m", "stats"]),
            Command::new("help", "Show help", Action::Help).with_aliases(vec!["h", "?"]),
            Command::new("refresh", "Refresh data", Action::Refresh)
                .with_aliases(vec!["r", "reload"]),
            Command::new("theme", "Toggle theme", Action::ToggleTheme)
                .with_aliases(vec!["t", "dark", "light"]),
            Command::new("search", "Start search", Action::Search).with_aliases(vec!["find", "/"]),
            Command::new("sort", "Sort current view", Action::SortColumn)
                .with_aliases(vec!["order"]),
            Command::new("filter", "Filter current view", Action::FilterToggle)
                .with_aliases(vec!["f"]),
        ]
    }

    pub fn state(&self) -> CommandPaletteState {
        self.state
    }

    pub fn is_open(&self) -> bool {
        self.state == CommandPaletteState::Open
    }

    pub fn open(&mut self) {
        self.state = CommandPaletteState::Open;
        self.input.clear();
        self.cursor_position = 0;
        self.selected_index = 0;
        self.history_index = None;
        self.update_filtered_commands();
    }

    pub fn close(&mut self) {
        self.state = CommandPaletteState::Hidden;
        self.input.clear();
        self.cursor_position = 0;
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn filtered_commands(&self) -> Vec<&Command> {
        self.filtered_commands
            .iter()
            .map(|&i| &self.commands[i])
            .collect()
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
        self.update_filtered_commands();
        self.selected_index = 0;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input.remove(self.cursor_position);
            self.update_filtered_commands();
            self.selected_index = 0;
        }
    }

    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.input.len() {
            self.input.remove(self.cursor_position);
            self.update_filtered_commands();
            self.selected_index = 0;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    pub fn move_cursor_start(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.input.len();
    }

    pub fn select_next(&mut self) {
        if !self.filtered_commands.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_commands.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.filtered_commands.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.filtered_commands.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }

        let new_index = match self.history_index {
            None => self.history.len() - 1,
            Some(i) if i > 0 => i - 1,
            Some(_) => return,
        };

        self.history_index = Some(new_index);
        self.input = self.history[new_index].clone();
        self.cursor_position = self.input.len();
        self.update_filtered_commands();
    }

    pub fn history_next(&mut self) {
        let new_index = match self.history_index {
            None => return,
            Some(i) if i < self.history.len() - 1 => i + 1,
            Some(_) => {
                self.history_index = None;
                self.input.clear();
                self.cursor_position = 0;
                self.update_filtered_commands();
                return;
            }
        };

        self.history_index = Some(new_index);
        self.input = self.history[new_index].clone();
        self.cursor_position = self.input.len();
        self.update_filtered_commands();
    }

    pub fn execute(&mut self) -> Option<Action> {
        if self.filtered_commands.is_empty() {
            return None;
        }

        let command_index = self.filtered_commands[self.selected_index];
        let action = self.commands[command_index].action.clone();

        if !self.input.is_empty() {
            self.history.push(self.input.clone());
            if self.history.len() > 100 {
                self.history.remove(0);
            }
        }

        self.close();
        Some(action)
    }

    pub fn execute_input(&mut self) -> Option<Action> {
        let trimmed = self.input.trim();
        if trimmed.is_empty() {
            self.close();
            return None;
        }

        for cmd in self.commands.iter() {
            if cmd.name.to_lowercase() == trimmed.to_lowercase()
                || cmd
                    .aliases
                    .iter()
                    .any(|a| a.to_lowercase() == trimmed.to_lowercase())
            {
                let action = cmd.action.clone();
                self.history.push(self.input.clone());
                self.close();
                return Some(action);
            }
        }

        if !self.filtered_commands.is_empty() {
            return self.execute();
        }

        self.close();
        None
    }

    fn update_filtered_commands(&mut self) {
        if self.input.is_empty() {
            self.filtered_commands = (0..self.commands.len()).collect();
        } else {
            self.filtered_commands = self
                .commands
                .iter()
                .enumerate()
                .filter(|(_, cmd)| cmd.matches(&self.input))
                .map(|(i, _)| i)
                .collect();
        }
    }

    pub fn add_command(&mut self, command: Command) {
        self.commands.push(command);
        self.update_filtered_commands();
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
        self.update_filtered_commands();
        self.selected_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_matches() {
        let cmd = Command::new("quit", "Exit", Action::Quit).with_aliases(vec!["q", "exit"]);

        assert!(cmd.matches("q"));
        assert!(cmd.matches("qu"));
        assert!(cmd.matches("quit"));
        assert!(cmd.matches("exit"));
        assert!(cmd.matches("ex"));
        assert!(!cmd.matches("xyz"));
    }

    #[test]
    fn test_palette_open_close() {
        let mut palette = CommandPalette::new();

        assert_eq!(palette.state(), CommandPaletteState::Hidden);

        palette.open();
        assert_eq!(palette.state(), CommandPaletteState::Open);
        assert!(palette.is_open());

        palette.close();
        assert_eq!(palette.state(), CommandPaletteState::Hidden);
        assert!(!palette.is_open());
    }

    #[test]
    fn test_input_handling() {
        let mut palette = CommandPalette::new();
        palette.open();

        palette.insert_char('h');
        palette.insert_char('e');
        palette.insert_char('l');
        palette.insert_char('p');

        assert_eq!(palette.input(), "help");
        assert_eq!(palette.cursor_position(), 4);

        palette.delete_char();
        assert_eq!(palette.input(), "hel");

        palette.move_cursor_left();
        palette.move_cursor_left();
        assert_eq!(palette.cursor_position(), 1);

        palette.insert_char('x');
        assert_eq!(palette.input(), "hxel");
    }

    #[test]
    fn test_filtering() {
        let mut palette = CommandPalette::new();
        palette.open();

        let initial_count = palette.filtered_commands().len();
        assert!(initial_count > 0);

        palette.insert_char('q');
        let filtered = palette.filtered_commands();
        assert!(filtered.iter().any(|c| c.name == "quit"));
    }

    #[test]
    fn test_selection() {
        let mut palette = CommandPalette::new();
        palette.open();

        assert_eq!(palette.selected_index(), 0);

        palette.select_next();
        assert_eq!(palette.selected_index(), 1);

        palette.select_prev();
        assert_eq!(palette.selected_index(), 0);
    }

    #[test]
    fn test_execute() {
        let mut palette = CommandPalette::new();
        palette.open();

        palette.insert_char('h');
        palette.insert_char('e');
        palette.insert_char('l');
        palette.insert_char('p');

        let action = palette.execute_input();
        assert_eq!(action, Some(Action::Help));
        assert!(!palette.is_open());
    }

    #[test]
    fn test_history() {
        let mut palette = CommandPalette::new();

        palette.open();
        palette.insert_char('h');
        palette.insert_char('e');
        palette.insert_char('l');
        palette.insert_char('p');
        palette.execute_input();

        palette.open();
        palette.insert_char('q');
        palette.insert_char('u');
        palette.insert_char('i');
        palette.insert_char('t');
        palette.execute_input();

        palette.open();
        palette.history_prev();
        assert_eq!(palette.input(), "quit");

        palette.history_prev();
        assert_eq!(palette.input(), "help");

        palette.history_next();
        assert_eq!(palette.input(), "quit");
    }

    #[test]
    fn test_cursor_movement() {
        let mut palette = CommandPalette::new();
        palette.open();

        palette.insert_char('t');
        palette.insert_char('e');
        palette.insert_char('s');
        palette.insert_char('t');

        palette.move_cursor_start();
        assert_eq!(palette.cursor_position(), 0);

        palette.move_cursor_end();
        assert_eq!(palette.cursor_position(), 4);
    }

    #[test]
    fn test_add_command() {
        let mut palette = CommandPalette::new();
        let initial_count = palette.commands.len();

        palette.add_command(Command::new("custom", "Custom command", Action::Refresh));

        assert_eq!(palette.commands.len(), initial_count + 1);
    }
}
