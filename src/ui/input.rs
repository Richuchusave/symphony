use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    PlayPause,
    Stop,
    Next,
    Previous,
    VolumeUp,
    VolumeDown,
    SeekForward(f64),
    SeekBackward(f64),
    Search,
    CommandPalette,
    Help,
    Quit,
    Enter,
    Back,
    Delete,
    ToggleSidebar,
    FocusSearch,
    ShowQueue,
    SavePlaylist,
    ToggleShuffle,
    ToggleRepeat,
    SelectNext,
    SelectPrevious,
    ScrollUp(u16, u16),
    ScrollDown(u16, u16),
    MouseClick(u16, u16),
    MouseDoubleClick(u16, u16),
    MouseDrag(u16, u16),
    RightClick(u16, u16),
    Resize(u16, u16),
    Tick,
    None,
}

pub fn handle_key(event: KeyEvent) -> Action {
    match (event.modifiers, event.code) {
        (KeyModifiers::NONE, KeyCode::Char(' ')) => Action::PlayPause,
        (KeyModifiers::NONE, KeyCode::Char('q')) => Action::Quit,
        (KeyModifiers::NONE, KeyCode::Char('s')) => Action::Stop,
        (KeyModifiers::NONE, KeyCode::Char('h')) => Action::Previous,
        (KeyModifiers::NONE, KeyCode::Char('l')) => Action::Next,
        (KeyModifiers::NONE, KeyCode::Char('j')) => Action::SelectNext,
        (KeyModifiers::NONE, KeyCode::Char('k')) => Action::SelectPrevious,
        (KeyModifiers::NONE, KeyCode::Char('z')) => Action::ToggleShuffle,
        (KeyModifiers::NONE, KeyCode::Char('x')) => Action::ToggleRepeat,
        (KeyModifiers::NONE, KeyCode::Char('/')) => Action::FocusSearch,
        (KeyModifiers::NONE, KeyCode::Char('?')) => Action::Help,
        (KeyModifiers::NONE, KeyCode::Up) => Action::VolumeUp,
        (KeyModifiers::NONE, KeyCode::Down) => Action::VolumeDown,
        (KeyModifiers::NONE, KeyCode::Left) => Action::SeekBackward(5.0),
        (KeyModifiers::NONE, KeyCode::Right) => Action::SeekForward(5.0),
        (KeyModifiers::NONE, KeyCode::Enter) => Action::Enter,
        (KeyModifiers::NONE, KeyCode::Esc) => Action::Back,
        (KeyModifiers::NONE, KeyCode::Backspace) => Action::Delete,
        (KeyModifiers::NONE, KeyCode::Tab) => Action::SelectNext,
        (KeyModifiers::SHIFT, KeyCode::BackTab) => Action::SelectPrevious,
        (KeyModifiers::SHIFT, KeyCode::Tab) => Action::SelectPrevious,
        (KeyModifiers::CONTROL, KeyCode::Char('b')) => Action::ToggleSidebar,
        (KeyModifiers::CONTROL, KeyCode::Char('q')) => Action::ShowQueue,
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => Action::SavePlaylist,
        (KeyModifiers::CONTROL, KeyCode::Char('k')) => Action::FocusSearch,
        (KeyModifiers::CONTROL, KeyCode::Char('p')) => Action::CommandPalette,
        _ => Action::None,
    }
}

pub fn handle_mouse(event: MouseEvent) -> Action {
    match event.kind {
        MouseEventKind::Down(button) => match button {
            crossterm::event::MouseButton::Left => Action::MouseClick(event.column, event.row),
            crossterm::event::MouseButton::Right => Action::RightClick(event.column, event.row),
            _ => Action::None,
        },
        MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
            Action::MouseDrag(event.column, event.row)
        }
        MouseEventKind::ScrollUp => Action::ScrollUp(event.column, event.row),
        MouseEventKind::ScrollDown => Action::ScrollDown(event.column, event.row),
        _ => Action::None,
    }
}

pub fn format_action(action: &Action) -> &str {
    match action {
        Action::PlayPause => "Play/Pause",
        Action::Stop => "Stop",
        Action::Next => "Next Track",
        Action::Previous => "Previous Track",
        Action::VolumeUp => "Volume Up",
        Action::VolumeDown => "Volume Down",
        Action::SeekForward(_) => "Seek Forward",
        Action::SeekBackward(_) => "Seek Backward",
        Action::Search => "Search",
        Action::CommandPalette => "Command Palette",
        Action::Help => "Help",
        Action::Quit => "Quit",
        Action::Enter => "Confirm",
        Action::Back => "Go Back",
        Action::Delete => "Delete",
        Action::ToggleSidebar => "Toggle Sidebar",
        Action::FocusSearch => "Focus Search",
        Action::ShowQueue => "Show Queue",
        Action::SavePlaylist => "Save Playlist",
        Action::ToggleShuffle => "Toggle Shuffle",
        Action::ToggleRepeat => "Toggle Repeat",
        Action::SelectNext => "Next Item",
        Action::SelectPrevious => "Previous Item",
        Action::ScrollUp(_, _) => "Scroll Up",
        Action::ScrollDown(_, _) => "Scroll Down",
        Action::MouseClick(_, _) => "Click",
        Action::MouseDoubleClick(_, _) => "Double Click",
        Action::MouseDrag(_, _) => "Drag",
        Action::RightClick(_, _) => "Right Click",
        Action::Resize(_, _) => "Resize",
        Action::Tick => "Tick",
        Action::None => "",
    }
}
