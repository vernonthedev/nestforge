use anyhow::{bail, Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Terminal,
};
use std::io::{self, Stdout};

use crate::cli::{AppTransport, GeneratorKindArg, GeneratorLayout};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TuiGenerateResult {
    pub kind: GeneratorKindArg,
    pub name: String,
    pub module: Option<String>,
    pub layout: GeneratorLayout,
    pub no_prompt: bool,
}

pub fn run_new_wizard() -> Result<(String, AppTransport)> {
    let mut terminal = TerminalSession::start()?;
    let mut state = NewWizardState::default();

    loop {
        terminal.draw(|frame| state.render(frame))?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if state.handle_key(key.code)? {
            break;
        }
    }

    Ok((state.app_name.trim().to_string(), state.transport))
}

pub fn run_generate_wizard() -> Result<TuiGenerateResult> {
    let mut terminal = TerminalSession::start()?;
    let mut state = GenerateWizardState::default();

    loop {
        terminal.draw(|frame| state.render(frame))?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if state.handle_key(key.code)? {
            break;
        }
    }

    Ok(TuiGenerateResult {
        kind: state.kind,
        name: state.name.trim().to_string(),
        module: state.module_name(),
        layout: if state.flat {
            GeneratorLayout::Flat
        } else {
            GeneratorLayout::Nested
        },
        no_prompt: state.no_prompt,
    })
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    fn start() -> Result<Self> {
        enable_raw_mode().context("Failed to initialize TUI raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)
            .context("Failed to switch terminal to alternate screen")?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).context("Failed to initialize TUI backend")?;
        Ok(Self { terminal })
    }

    fn draw<F>(&mut self, render: F) -> Result<()>
    where
        F: FnOnce(&mut ratatui::Frame<'_>),
    {
        self.terminal.draw(render)?;
        Ok(())
    }
}

pub fn should_fallback_to_prompt(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        let message = cause.to_string().to_ascii_lowercase();
        message.contains("failed to initialize tui")
            || message.contains("alternate screen")
            || message.contains("raw mode")
            || message.contains("the system cannot find the file specified")
            || message.contains("os error 2")
    })
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

#[derive(Clone, Copy)]
enum NewField {
    Name,
    Transport,
    Submit,
}

struct NewWizardState {
    app_name: String,
    transport: AppTransport,
    focus: NewField,
    error: Option<String>,
}

impl Default for NewWizardState {
    fn default() -> Self {
        Self {
            app_name: String::new(),
            transport: AppTransport::Http,
            focus: NewField::Name,
            error: None,
        }
    }
}

impl NewWizardState {
    fn render(&self, frame: &mut ratatui::Frame<'_>) {
        let area = centered_rect(frame.area(), 78, 48);
        frame.render_widget(Clear, area);
        frame.render_widget(
            Block::default()
                .title(" NestForge New App ")
                .borders(Borders::ALL)
                .border_style(Style::default().cyan()),
            area,
        );

        let inner = area.inner(Margin {
            horizontal: 2,
            vertical: 1,
        });
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(inner);

        frame.render_widget(
            Paragraph::new(
                "Create a new NestForge app. Tab moves focus, Enter advances, Esc cancels.",
            )
            .style(Style::default().gray()),
            chunks[0],
        );
        frame.render_widget(
            value_block(
                "Application Name",
                &self.app_name,
                "Type the project folder name",
                matches!(self.focus, NewField::Name),
            ),
            chunks[1],
        );
        frame.render_widget(
            value_block(
                "Transport",
                self.transport.label(),
                "Use Left/Right to change",
                matches!(self.focus, NewField::Transport),
            ),
            chunks[2],
        );
        frame.render_widget(
            submit_block("Create App", matches!(self.focus, NewField::Submit)),
            chunks[3],
        );
        frame.render_widget(
            status_line(
                self.error.as_deref(),
                "Type into the name field. Left/Right changes transport.",
            ),
            chunks[4],
        );
    }

    fn handle_key(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Esc => bail!("TUI cancelled by user."),
            KeyCode::Up => {
                self.focus = match self.focus {
                    NewField::Name => NewField::Submit,
                    NewField::Transport => NewField::Name,
                    NewField::Submit => NewField::Transport,
                }
            }
            KeyCode::Down | KeyCode::Tab => {
                self.focus = match self.focus {
                    NewField::Name => NewField::Transport,
                    NewField::Transport => NewField::Submit,
                    NewField::Submit => NewField::Name,
                }
            }
            KeyCode::Left if matches!(self.focus, NewField::Transport) => {
                self.transport = previous_transport(self.transport)
            }
            KeyCode::Right if matches!(self.focus, NewField::Transport) => {
                self.transport = next_transport(self.transport)
            }
            KeyCode::Backspace if matches!(self.focus, NewField::Name) => {
                self.app_name.pop();
            }
            KeyCode::Char(ch) if matches!(self.focus, NewField::Name) => self.app_name.push(ch),
            KeyCode::Enter if matches!(self.focus, NewField::Submit) => {
                if self.app_name.trim().is_empty() {
                    self.error = Some("Application name cannot be empty.".to_string());
                } else {
                    return Ok(true);
                }
            }
            KeyCode::Enter => {
                self.focus = match self.focus {
                    NewField::Name => NewField::Transport,
                    NewField::Transport => NewField::Submit,
                    NewField::Submit => NewField::Submit,
                };
            }
            _ => {}
        }

        if !matches!(code, KeyCode::Enter) {
            self.error = None;
        }

        Ok(false)
    }
}

#[derive(Clone, Copy)]
enum GenerateField {
    Kind,
    Name,
    InModule,
    ModuleName,
    Layout,
    Prompt,
    Submit,
}

struct GenerateWizardState {
    kind: GeneratorKindArg,
    name: String,
    in_module: bool,
    module: String,
    flat: bool,
    no_prompt: bool,
    focus: GenerateField,
    error: Option<String>,
}

impl Default for GenerateWizardState {
    fn default() -> Self {
        Self {
            kind: GeneratorKindArg::Resource,
            name: String::new(),
            in_module: false,
            module: String::new(),
            flat: false,
            no_prompt: false,
            focus: GenerateField::Name,
            error: None,
        }
    }
}

impl GenerateWizardState {
    fn module_name(&self) -> Option<String> {
        self.in_module
            .then(|| self.module.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn render(&self, frame: &mut ratatui::Frame<'_>) {
        let area = centered_rect(frame.area(), 84, 68);
        frame.render_widget(Clear, area);
        frame.render_widget(
            Block::default()
                .title(" NestForge Generate ")
                .borders(Borders::ALL)
                .border_style(Style::default().cyan()),
            area,
        );

        let inner = area.inner(Margin {
            horizontal: 2,
            vertical: 1,
        });
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(inner);

        frame.render_widget(
            Paragraph::new(
                "Tab or arrows move focus. Type into text fields. Enter advances or submits.",
            )
            .style(Style::default().gray()),
            chunks[0],
        );
        frame.render_widget(
            value_block(
                "Generator",
                self.kind.label(),
                "Use Left/Right to change",
                matches!(self.focus, GenerateField::Kind),
            ),
            chunks[1],
        );
        frame.render_widget(
            value_block(
                "Name",
                &self.name,
                "Type the generated resource or module name",
                matches!(self.focus, GenerateField::Name),
            ),
            chunks[2],
        );
        frame.render_widget(
            toggle_block(
                "Generate inside module",
                self.in_module,
                matches!(self.focus, GenerateField::InModule),
            ),
            chunks[3],
        );
        frame.render_widget(
            value_block(
                "Module Name",
                &self.module_name_value(),
                if self.in_module {
                    "Type an existing feature module name"
                } else {
                    "Turn on module mode to edit this field"
                },
                matches!(self.focus, GenerateField::ModuleName),
            ),
            chunks[4],
        );
        frame.render_widget(
            toggle_block(
                "Flat layout",
                self.flat,
                matches!(self.focus, GenerateField::Layout),
            ),
            chunks[5],
        );
        frame.render_widget(
            toggle_block(
                "Disable DTO prompts",
                self.no_prompt,
                matches!(self.focus, GenerateField::Prompt),
            ),
            chunks[6],
        );
        frame.render_widget(
            submit_block("Generate", matches!(self.focus, GenerateField::Submit)),
            chunks[7],
        );
        frame.render_widget(
            status_line(
                self.error.as_deref(),
                "Left/Right changes the generator or toggles the active switch. Space also toggles switches.",
            ),
            chunks[8],
        );
    }

    fn module_name_value(&self) -> String {
        if self.in_module {
            self.module.clone()
        } else {
            String::new()
        }
    }

    fn handle_key(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Esc => bail!("TUI cancelled by user."),
            KeyCode::Up => self.focus = previous_generate_field(self.focus, self.in_module),
            KeyCode::Down | KeyCode::Tab => {
                self.focus = next_generate_field(self.focus, self.in_module)
            }
            KeyCode::Left if matches!(self.focus, GenerateField::Kind) => {
                self.kind = previous_kind(self.kind)
            }
            KeyCode::Right if matches!(self.focus, GenerateField::Kind) => {
                self.kind = next_kind(self.kind)
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Char(' ') => match self.focus {
                GenerateField::InModule => self.in_module = !self.in_module,
                GenerateField::Layout => self.flat = !self.flat,
                GenerateField::Prompt => self.no_prompt = !self.no_prompt,
                _ => {}
            },
            KeyCode::Backspace if matches!(self.focus, GenerateField::Name) => {
                self.name.pop();
            }
            KeyCode::Backspace if matches!(self.focus, GenerateField::ModuleName) => {
                self.module.pop();
            }
            KeyCode::Char(ch) if matches!(self.focus, GenerateField::Name) => self.name.push(ch),
            KeyCode::Char(ch) if matches!(self.focus, GenerateField::ModuleName) => {
                self.module.push(ch)
            }
            KeyCode::Enter if matches!(self.focus, GenerateField::Submit) => {
                if self.name.trim().is_empty() {
                    self.error = Some("Generator name cannot be empty.".to_string());
                } else if self.in_module && self.module.trim().is_empty() {
                    self.error = Some(
                        "Module name cannot be empty when module mode is enabled.".to_string(),
                    );
                } else {
                    return Ok(true);
                }
            }
            KeyCode::Enter => {
                self.focus = next_generate_field(self.focus, self.in_module);
            }
            _ => {}
        }

        if !matches!(code, KeyCode::Enter) {
            self.error = None;
        }

        Ok(false)
    }
}

fn centered_rect(
    area: ratatui::layout::Rect,
    width_percent: u16,
    height_percent: u16,
) -> ratatui::layout::Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
}

fn value_block<'a>(title: &'a str, value: &'a str, hint: &'a str, active: bool) -> Paragraph<'a> {
    let (display, style) = if value.trim().is_empty() {
        (
            hint.to_string(),
            Style::default().dark_gray().add_modifier(Modifier::ITALIC),
        )
    } else {
        (
            format!("{value}{}", if active { " _" } else { "" }),
            Style::default(),
        )
    };

    Paragraph::new(display).style(style).block(
        Block::default()
            .title(format!(" {}{} ", if active { "> " } else { "" }, title))
            .borders(Borders::ALL)
            .border_style(active_border(active)),
    )
}

fn toggle_block<'a>(title: &'a str, enabled: bool, active: bool) -> Paragraph<'a> {
    let state = if enabled { "On" } else { "Off" };
    Paragraph::new(Line::from(vec![Span::styled(
        state,
        if enabled {
            Style::default().green().add_modifier(Modifier::BOLD)
        } else {
            Style::default().dark_gray()
        },
    )]))
    .block(
        Block::default()
            .title(format!(" {}{} ", if active { "> " } else { "" }, title))
            .borders(Borders::ALL)
            .border_style(active_border(active)),
    )
}

fn submit_block<'a>(label: &'a str, active: bool) -> Paragraph<'a> {
    Paragraph::new(Line::from(vec![Span::styled(
        format!("[ Enter ] {label}"),
        if active {
            Style::default().yellow().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        },
    )]))
    .block(
        Block::default()
            .title(format!(" {}{} ", if active { "> " } else { "" }, "Submit"))
            .borders(Borders::ALL)
            .border_style(active_border(active)),
    )
}

fn status_line<'a>(error: Option<&'a str>, hint: &'a str) -> Paragraph<'a> {
    match error {
        Some(message) => Paragraph::new(message).style(Style::default().red()),
        None => Paragraph::new(hint).style(Style::default().dark_gray()),
    }
}

fn active_border(active: bool) -> Style {
    if active {
        Style::default().yellow()
    } else {
        Style::default().gray()
    }
}

fn next_transport(current: AppTransport) -> AppTransport {
    match current {
        AppTransport::Http => AppTransport::Graphql,
        AppTransport::Graphql => AppTransport::Grpc,
        AppTransport::Grpc => AppTransport::Microservices,
        AppTransport::Microservices => AppTransport::Websockets,
        AppTransport::Websockets => AppTransport::Http,
    }
}

fn previous_transport(current: AppTransport) -> AppTransport {
    match current {
        AppTransport::Http => AppTransport::Websockets,
        AppTransport::Graphql => AppTransport::Http,
        AppTransport::Grpc => AppTransport::Graphql,
        AppTransport::Microservices => AppTransport::Grpc,
        AppTransport::Websockets => AppTransport::Microservices,
    }
}

fn next_kind(current: GeneratorKindArg) -> GeneratorKindArg {
    use GeneratorKindArg as K;
    match current {
        K::Resource => K::Controller,
        K::Controller => K::Service,
        K::Service => K::Module,
        K::Module => K::Guard,
        K::Guard => K::Decorator,
        K::Decorator => K::Filter,
        K::Filter => K::Middleware,
        K::Middleware => K::Interceptor,
        K::Interceptor => K::Serializer,
        K::Serializer => K::Graphql,
        K::Graphql => K::Grpc,
        K::Grpc => K::Gateway,
        K::Gateway => K::Microservice,
        K::Microservice => K::Resource,
    }
}

fn previous_kind(current: GeneratorKindArg) -> GeneratorKindArg {
    use GeneratorKindArg as K;
    match current {
        K::Resource => K::Microservice,
        K::Controller => K::Resource,
        K::Service => K::Controller,
        K::Module => K::Service,
        K::Guard => K::Module,
        K::Decorator => K::Guard,
        K::Filter => K::Decorator,
        K::Middleware => K::Filter,
        K::Interceptor => K::Middleware,
        K::Serializer => K::Interceptor,
        K::Graphql => K::Serializer,
        K::Grpc => K::Graphql,
        K::Gateway => K::Grpc,
        K::Microservice => K::Gateway,
    }
}

fn next_generate_field(current: GenerateField, in_module: bool) -> GenerateField {
    match current {
        GenerateField::Kind => GenerateField::Name,
        GenerateField::Name => GenerateField::InModule,
        GenerateField::InModule if in_module => GenerateField::ModuleName,
        GenerateField::InModule => GenerateField::Layout,
        GenerateField::ModuleName => GenerateField::Layout,
        GenerateField::Layout => GenerateField::Prompt,
        GenerateField::Prompt => GenerateField::Submit,
        GenerateField::Submit => GenerateField::Kind,
    }
}

fn previous_generate_field(current: GenerateField, in_module: bool) -> GenerateField {
    match current {
        GenerateField::Kind => GenerateField::Submit,
        GenerateField::Name => GenerateField::Kind,
        GenerateField::InModule => GenerateField::Name,
        GenerateField::ModuleName => GenerateField::InModule,
        GenerateField::Layout if in_module => GenerateField::ModuleName,
        GenerateField::Layout => GenerateField::InModule,
        GenerateField::Prompt => GenerateField::Layout,
        GenerateField::Submit => GenerateField::Prompt,
    }
}
