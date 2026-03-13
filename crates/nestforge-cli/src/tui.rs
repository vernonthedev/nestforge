use anyhow::{bail, Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Terminal,
};
use std::io::{self, Stdout};

use crate::cli::{AppTransport, GeneratorKindArg, GeneratorLayout};
use crate::ui::BRAND_BANNER;

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
        let area = centered_rect(frame.area(), 94, 72);
        frame.render_widget(Clear, area);
        frame.render_widget(
            Block::default()
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
                Constraint::Length(1),
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(inner);

        frame.render_widget(brand_banner_block(), chunks[1]);
        frame.render_widget(
            Paragraph::new(
                "Create a new NestForge app. Tab moves focus, Enter advances, Esc cancels.",
            )
            .style(Style::default().gray()),
            chunks[3],
        );
        frame.render_widget(
            field_row(
                "Application Name",
                &self.app_name,
                "Type the project folder name",
                matches!(self.focus, NewField::Name),
            ),
            chunks[4],
        );
        frame.render_widget(
            field_row(
                "Transport",
                self.transport.label(),
                "Use Left/Right to change",
                matches!(self.focus, NewField::Transport),
            ),
            chunks[5],
        );
        frame.render_widget(
            submit_block("Create App", matches!(self.focus, NewField::Submit)),
            chunks[6],
        );
        frame.render_widget(
            status_line(
                self.error.as_deref(),
                "Only the name field accepts typed text. Transport is a selector.",
            ),
            chunks[7],
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
            KeyCode::Enter if matches!(self.focus, NewField::Transport) => {
                self.focus = NewField::Submit;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GenerateStep {
    Kind,
    Name,
    InModule,
    ModuleName,
    Layout,
    Prompt,
    Review,
}

struct GenerateWizardState {
    kind: GeneratorKindArg,
    name: String,
    in_module: bool,
    module: String,
    flat: bool,
    no_prompt: bool,
    step: GenerateStep,
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
            step: GenerateStep::Kind,
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
        let area = centered_rect(frame.area(), 94, 82);
        frame.render_widget(Clear, area);
        frame.render_widget(
            Block::default()
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
                Constraint::Length(1),
                Constraint::Length(6),
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(7),
                Constraint::Min(1),
            ])
            .split(inner);

        frame.render_widget(brand_banner_block(), chunks[1]);
        frame.render_widget(
            Paragraph::new(
                "Step-by-step wizard. Enter continues, Backspace edits text, Esc cancels.",
            )
            .style(Style::default().gray()),
            chunks[3],
        );
        frame.render_widget(
            Paragraph::new(format!(
                "Step {} of {}: {}",
                self.step_number(),
                self.total_steps(),
                self.step_title()
            ))
            .style(Style::default().cyan().add_modifier(Modifier::BOLD)),
            chunks[4],
        );
        frame.render_widget(
            prompt_card(self.step_title(), self.step_value(), self.step_hint()),
            chunks[5],
        );
        frame.render_widget(
            status_line(self.error.as_deref(), self.step_controls()),
            chunks[6],
        );
        frame.render_widget(
            summary_card(&[
                ("Generator", self.kind.label().to_string()),
                ("Name", value_or_dash(&self.name)),
                (
                    "Inside module",
                    if self.in_module {
                        "Yes".to_string()
                    } else {
                        "No".to_string()
                    },
                ),
                (
                    "Module name",
                    self.module_name().unwrap_or_else(|| "-".to_string()),
                ),
                (
                    "Layout",
                    if self.flat {
                        "Flat".to_string()
                    } else {
                        "Nested".to_string()
                    },
                ),
                (
                    "DTO prompts",
                    if self.no_prompt {
                        "Disabled".to_string()
                    } else {
                        "Enabled".to_string()
                    },
                ),
            ]),
            chunks[7],
        );
        frame.render_widget(
            Paragraph::new(
                "Enter applies the current step. Up/Left goes back. Down/Right goes forward.",
            )
            .style(Style::default().dark_gray()),
            chunks[8],
        );
    }

    fn handle_key(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Esc => bail!("TUI cancelled by user."),
            KeyCode::Up | KeyCode::Left => match self.step {
                GenerateStep::Kind => self.kind = previous_kind(self.kind),
                GenerateStep::InModule => self.in_module = false,
                GenerateStep::Layout => self.flat = false,
                GenerateStep::Prompt => self.no_prompt = false,
                _ => self.step = self.previous_step(),
            },
            KeyCode::Down | KeyCode::Right => match self.step {
                GenerateStep::Kind => self.kind = next_kind(self.kind),
                GenerateStep::InModule => self.in_module = true,
                GenerateStep::Layout => self.flat = true,
                GenerateStep::Prompt => self.no_prompt = true,
                _ => self.step = self.next_step(),
            },
            KeyCode::Backspace if matches!(self.step, GenerateStep::Name) => {
                self.name.pop();
            }
            KeyCode::Backspace if matches!(self.step, GenerateStep::ModuleName) => {
                self.module.pop();
            }
            KeyCode::Char(ch) if matches!(self.step, GenerateStep::Name) => self.name.push(ch),
            KeyCode::Char(ch) if matches!(self.step, GenerateStep::ModuleName) => {
                self.module.push(ch)
            }
            KeyCode::Char(' ') if matches!(self.step, GenerateStep::InModule) => {
                self.in_module = !self.in_module;
            }
            KeyCode::Char(' ') if matches!(self.step, GenerateStep::Layout) => {
                self.flat = !self.flat;
            }
            KeyCode::Char(' ') if matches!(self.step, GenerateStep::Prompt) => {
                self.no_prompt = !self.no_prompt;
            }
            KeyCode::Char('1') if matches!(self.step, GenerateStep::InModule) => {
                self.in_module = false;
            }
            KeyCode::Char('2') if matches!(self.step, GenerateStep::InModule) => {
                self.in_module = true;
            }
            KeyCode::Char('y') | KeyCode::Char('Y')
                if matches!(self.step, GenerateStep::InModule) =>
            {
                self.in_module = true;
            }
            KeyCode::Char('n') | KeyCode::Char('N')
                if matches!(self.step, GenerateStep::InModule) =>
            {
                self.in_module = false;
            }
            KeyCode::Char('f') | KeyCode::Char('F')
                if matches!(self.step, GenerateStep::Layout) =>
            {
                self.flat = true;
            }
            KeyCode::Char('n') | KeyCode::Char('N')
                if matches!(self.step, GenerateStep::Layout) =>
            {
                self.flat = false;
            }
            KeyCode::Char('1') if matches!(self.step, GenerateStep::Layout) => {
                self.flat = false;
            }
            KeyCode::Char('2') if matches!(self.step, GenerateStep::Layout) => {
                self.flat = true;
            }
            KeyCode::Char('e') | KeyCode::Char('E')
                if matches!(self.step, GenerateStep::Prompt) =>
            {
                self.no_prompt = false;
            }
            KeyCode::Char('d') | KeyCode::Char('D')
                if matches!(self.step, GenerateStep::Prompt) =>
            {
                self.no_prompt = true;
            }
            KeyCode::Char('1') if matches!(self.step, GenerateStep::Prompt) => {
                self.no_prompt = false;
            }
            KeyCode::Char('2') if matches!(self.step, GenerateStep::Prompt) => {
                self.no_prompt = true;
            }
            KeyCode::Enter if matches!(self.step, GenerateStep::Kind) => {
                self.step = self.next_step();
            }
            KeyCode::Enter if matches!(self.step, GenerateStep::Name) => {
                if self.name.trim().is_empty() {
                    self.error = Some("Generator name cannot be empty.".to_string());
                } else {
                    self.step = self.next_step();
                }
            }
            KeyCode::Enter if matches!(self.step, GenerateStep::InModule) => {
                self.step = self.next_step();
            }
            KeyCode::Enter if matches!(self.step, GenerateStep::ModuleName) => {
                if self.module.trim().is_empty() {
                    self.error = Some(
                        "Module name cannot be empty when module mode is enabled.".to_string(),
                    );
                } else {
                    self.step = self.next_step();
                }
            }
            KeyCode::Enter if matches!(self.step, GenerateStep::Layout) => {
                self.step = self.next_step();
            }
            KeyCode::Enter if matches!(self.step, GenerateStep::Prompt) => {
                self.step = self.next_step();
            }
            KeyCode::Enter if matches!(self.step, GenerateStep::Review) => {
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
            _ => {}
        }

        if !matches!(code, KeyCode::Enter) {
            self.error = None;
        }

        Ok(false)
    }

    fn next_step(&self) -> GenerateStep {
        match self.step {
            GenerateStep::Kind => GenerateStep::Name,
            GenerateStep::Name => GenerateStep::InModule,
            GenerateStep::InModule if self.in_module => GenerateStep::ModuleName,
            GenerateStep::InModule => GenerateStep::Layout,
            GenerateStep::ModuleName => GenerateStep::Layout,
            GenerateStep::Layout => GenerateStep::Prompt,
            GenerateStep::Prompt => GenerateStep::Review,
            GenerateStep::Review => GenerateStep::Review,
        }
    }

    fn previous_step(&self) -> GenerateStep {
        match self.step {
            GenerateStep::Kind => GenerateStep::Kind,
            GenerateStep::Name => GenerateStep::Kind,
            GenerateStep::InModule => GenerateStep::Name,
            GenerateStep::ModuleName => GenerateStep::InModule,
            GenerateStep::Layout if self.in_module => GenerateStep::ModuleName,
            GenerateStep::Layout => GenerateStep::InModule,
            GenerateStep::Prompt => GenerateStep::Layout,
            GenerateStep::Review => GenerateStep::Prompt,
        }
    }

    fn total_steps(&self) -> usize {
        if self.in_module {
            7
        } else {
            6
        }
    }

    fn step_number(&self) -> usize {
        match self.step {
            GenerateStep::Kind => 1,
            GenerateStep::Name => 2,
            GenerateStep::InModule => 3,
            GenerateStep::ModuleName => 4,
            GenerateStep::Layout if self.in_module => 5,
            GenerateStep::Layout => 4,
            GenerateStep::Prompt if self.in_module => 6,
            GenerateStep::Prompt => 5,
            GenerateStep::Review if self.in_module => 7,
            GenerateStep::Review => 6,
        }
    }

    fn step_title(&self) -> &'static str {
        match self.step {
            GenerateStep::Kind => "Choose generator",
            GenerateStep::Name => "Enter resource or module name",
            GenerateStep::InModule => "Generate inside a module?",
            GenerateStep::ModuleName => "Enter target module name",
            GenerateStep::Layout => "Choose layout",
            GenerateStep::Prompt => "DTO prompts",
            GenerateStep::Review => "Review and generate",
        }
    }

    fn step_hint(&self) -> &'static str {
        match self.step {
            GenerateStep::Kind => {
                "Use Left/Right to cycle. Enter keeps the current generator and continues."
            }
            GenerateStep::Name => "Type a name like users, health, auth, or billing.",
            GenerateStep::InModule => {
                "Type 1 for No or 2 for Yes. Left/Right and Y/N also work. Enter continues."
            }
            GenerateStep::ModuleName => "Type the existing feature module name.",
            GenerateStep::Layout => {
                "Type 1 for Nested or 2 for Flat. Left/Right and N/F also work. Enter continues."
            }
            GenerateStep::Prompt => {
                "Type 1 for enabled or 2 for disabled DTO prompts. Left/Right and E/D also work. Enter continues."
            }
            GenerateStep::Review => {
                "Press Enter to generate now, or Left/Up to revise earlier answers."
            }
        }
    }

    fn step_controls(&self) -> &'static str {
        match self.step {
            GenerateStep::Kind => "Left/Right changes the generator. Enter continues.",
            GenerateStep::Name => "Type the name. Backspace edits. Enter continues.",
            GenerateStep::InModule => "1=No, 2=Yes, Left=No, Right=Yes, Y=yes, N=no. Enter continues.",
            GenerateStep::ModuleName => "Type the module name. Backspace edits. Enter continues.",
            GenerateStep::Layout => {
                "1=Nested, 2=Flat, Left=Nested, Right=Flat, N=nested, F=flat. Enter continues."
            }
            GenerateStep::Prompt => {
                "1=Enabled, 2=Disabled, Left=enabled, Right=disabled, E=enabled, D=disabled. Enter continues."
            }
            GenerateStep::Review => "Enter generates. Left or Up goes back to the previous step.",
        }
    }

    fn step_value(&self) -> String {
        match self.step {
            GenerateStep::Kind => format!("Current generator: {}", self.kind.label()),
            GenerateStep::Name => value_or_hint(&self.name, "Type the generated name here"),
            GenerateStep::InModule => {
                if self.in_module {
                    "Current choice: Yes (2)".to_string()
                } else {
                    "Current choice: No (1)".to_string()
                }
            }
            GenerateStep::ModuleName => value_or_hint(&self.module, "Type the target module name"),
            GenerateStep::Layout => {
                if self.flat {
                    "Current layout: Flat (2)".to_string()
                } else {
                    "Current layout: Nested (1)".to_string()
                }
            }
            GenerateStep::Prompt => {
                if self.no_prompt {
                    "Current choice: Disabled (2)".to_string()
                } else {
                    "Current choice: Enabled (1)".to_string()
                }
            }
            GenerateStep::Review => "Ready to generate with the current selections.".to_string(),
        }
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

fn field_row<'a>(title: &'a str, value: &'a str, hint: &'a str, active: bool) -> Paragraph<'a> {
    let (display, style) = if value.trim().is_empty() {
        (
            format!("{title}: {hint}"),
            Style::default().gray().add_modifier(Modifier::ITALIC),
        )
    } else {
        (format!("{title}: {value}"), Style::default())
    };

    Paragraph::new(display).style(style).block(
        Block::default()
            .title(if active { " > " } else { " " })
            .borders(Borders::ALL)
            .border_style(active_border(active)),
    )
}

fn brand_banner_block<'a>() -> Paragraph<'a> {
    let lines = BRAND_BANNER
        .iter()
        .map(|line| {
            Line::from(vec![Span::styled(
                *line,
                Style::default().cyan().add_modifier(Modifier::BOLD),
            )])
        })
        .collect::<Vec<_>>();

    Paragraph::new(lines).alignment(Alignment::Center)
}

fn prompt_card<'a>(title: &'a str, value: String, hint: &'a str) -> Paragraph<'a> {
    Paragraph::new(vec![
        Line::from(vec![Span::styled(
            title,
            Style::default().yellow().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(value),
        Line::from(""),
        Line::from(vec![Span::styled(
            hint,
            Style::default().gray().add_modifier(Modifier::ITALIC),
        )]),
    ])
    .block(Block::default().borders(Borders::ALL))
}

fn summary_card<'a>(items: &'a [(&'a str, String)]) -> Paragraph<'a> {
    let mut lines = Vec::with_capacity(items.len());
    for (label, value) in items {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{label}: "),
                Style::default().cyan().add_modifier(Modifier::BOLD),
            ),
            Span::raw(value.clone()),
        ]));
    }

    Paragraph::new(lines).block(Block::default().title(" Summary ").borders(Borders::ALL))
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
            .title(if active { " > " } else { " " })
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

fn value_or_hint(value: &str, hint: &str) -> String {
    if value.trim().is_empty() {
        hint.to_string()
    } else {
        value.to_string()
    }
}

fn value_or_dash(value: &str) -> String {
    if value.trim().is_empty() {
        "-".to_string()
    } else {
        value.to_string()
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

#[derive(Clone, Copy)]
struct DocPage {
    slug: &'static str,
    title: &'static str,
    body: &'static str,
}

const DOC_PAGES: &[DocPage] = &[
    DocPage {
        slug: "overview",
        title: "Overview",
        body: r#"NestForge CLI keeps the full scaffold-and-generate workflow inside the terminal.

The intended loop is:
1. Create a new app with `nestforge new`.
2. Generate modules and resources with `nestforge g ...`.
3. Run `cargo check` or `cargo run`.
4. Export OpenAPI docs or run database commands when needed.

This browser is meant to replace context switching to a web page when you just need the next command, generator shape, or a reminder of the normal workflow."#,
    },
    DocPage {
        slug: "getting-started",
        title: "Getting Started",
        body: r#"Quick start:

  nestforge new my-app --transport http --no-tui
  cd my-app
  cargo run

Recommended first pass for a feature:

  nestforge g module users --flat
  nestforge g resource users --module users --flat --no-prompt
  cargo check

If you want guided prompts, omit `--no-tui` and the CLI will use the interactive flow when the terminal supports it."#,
    },
    DocPage {
        slug: "new",
        title: "Create Apps",
        body: r#"Use `nestforge new <app-name>` to scaffold a fresh application.

Common examples:
  nestforge new billing-api --transport http
  nestforge new graph-api --transport graphql --openapi
  nestforge new chat-app --transport websockets
  nestforge new events-app --transport microservices
  nestforge new edge-grpc --transport grpc

Important flags:
  --transport <http|graphql|grpc|microservices|websockets>
  --openapi
  --no-tui

`--no-tui` disables the fullscreen creation wizard and falls back to direct flags or prompt mode."#,
    },
    DocPage {
        slug: "generate",
        title: "Generate Workflow",
        body: r#"Use `nestforge generate` or the short alias `nestforge g`.

The most common sequence is:
  nestforge g module users
  nestforge g resource users --module users

That creates a feature module and then adds DTOs, controller, and service for the resource.

You can also generate individual pieces:
  nestforge g controller auth --module users
  nestforge g service auth --module users
  nestforge g guard auth
  nestforge g interceptor logging

The CLI is designed so resource generation is usually the fastest way to create a full feature slice."#,
    },
    DocPage {
        slug: "generators",
        title: "Generator Reference",
        body: r#"Available generators:

  resource      DTOs + controller + service
  module        Feature module scaffold
  controller    Controller only
  service       Service only
  guard         Guard scaffold
  decorator     Request decorator scaffold
  filter        Exception filter scaffold
  middleware    Middleware scaffold
  interceptor   Interceptor scaffold
  serializer    Response serializer scaffold
  graphql       GraphQL resolver scaffold
  grpc          gRPC service scaffold
  gateway       WebSocket gateway scaffold
  microservice  Microservice pattern scaffold

Examples:
  nestforge g serializer user
  nestforge g graphql users
  nestforge g gateway events
  nestforge g microservice billing"#,
    },
    DocPage {
        slug: "layout",
        title: "Layout and Module Options",
        body: r#"Generator flags that shape where files go:

  --module <name>
    Generate inside an existing feature module.

  --flat
    Generate files directly in the module root instead of nested folders.

  --no-prompt
    Skip DTO field customization prompts.

Examples:
  nestforge g module billing --flat
  nestforge g resource invoices --module billing --flat --no-prompt

Use nested layout when you want larger features grouped by folders.
Use flat layout when you want a compact, NestJS-like feature folder."#,
    },
    DocPage {
        slug: "openapi",
        title: "OpenAPI Export",
        body: r#"CLI docs and API docs are separate concerns.

Use `nestforge docs` for CLI help inside the terminal.
Use `nestforge export-docs` when you want generated OpenAPI files.

Examples:
  nestforge export-docs --format json
  nestforge export-docs --format yaml --output docs/openapi.yaml
  nestforge export-docs --format yaml --title "Users API" --version "1.0.0"

The `export-docs` command inspects the app module and writes OpenAPI output for supported routes."#,
    },
    DocPage {
        slug: "db",
        title: "Database Workflow",
        body: r#"Database commands:

  nestforge db init
  nestforge db generate create_users
  nestforge db migrate
  nestforge db status

Typical flow:
1. Initialize migration support with `db init`.
2. Create a named migration with `db generate`.
3. Edit the SQL file.
4. Apply migrations with `db migrate`.
5. Check current state with `db status`."#,
    },
    DocPage {
        slug: "tips",
        title: "Tips and Conventions",
        body: r#"Navigation in this browser:
  j / Down     next topic
  k / Up       previous topic
  PageDown     scroll content down
  PageUp       scroll content up
  /            search topics
  q            quit

General CLI tips:
  Use `nestforge --help` for the top-level command map.
  Use `nestforge <command> --help` for flag details.
  Prefer `resource` generation when starting a CRUD-style feature.
  Prefer `module` first when you want to grow a feature incrementally.
  Keep `export-docs` for OpenAPI output and `docs` for CLI workflow help."#,
    },
];

pub fn render_docs_plaintext(topic: Option<&str>) -> String {
    let pages = filtered_doc_pages(topic);
    let mut out = String::from("NestForge CLI Docs\n");

    if pages.len() == 1 {
        let page = pages[0];
        out.push('\n');
        out.push_str(page.title);
        out.push('\n');
        out.push('\n');
        out.push_str(page.body);
        out.push('\n');
        return out;
    }

    for page in pages {
        out.push('\n');
        out.push_str(page.title);
        out.push('\n');
        out.push('\n');
        out.push_str(page.body);
        out.push('\n');
    }

    out
}

pub fn run_docs_browser(topic: Option<&str>) -> Result<()> {
    let mut terminal = TerminalSession::start()?;
    let mut state = DocsBrowserState::new(topic);

    loop {
        terminal.draw(|frame| state.render(frame))?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if state.handle_key(key.code, key.modifiers) {
            break;
        }
    }

    Ok(())
}

struct DocsBrowserState {
    selected: usize,
    scroll: u16,
    query: String,
    search_mode: bool,
    status: Option<String>,
}

impl DocsBrowserState {
    fn new(topic: Option<&str>) -> Self {
        let mut state = Self {
            selected: 0,
            scroll: 0,
            query: String::new(),
            search_mode: false,
            status: None,
        };

        if let Some(topic) = topic {
            let normalized = topic.trim().to_ascii_lowercase();
            if let Some(index) = DOC_PAGES.iter().position(|page| {
                page.slug.contains(&normalized)
                    || page.title.to_ascii_lowercase().contains(&normalized)
                    || page.body.to_ascii_lowercase().contains(&normalized)
            }) {
                state.selected = index;
                state.status = Some(format!("Opened topic `{}`.", DOC_PAGES[index].title));
            } else if !normalized.is_empty() {
                state.status = Some(format!(
                    "Topic `{}` was not found. Showing all docs instead.",
                    topic
                ));
            }
        }

        state
    }

    fn render(&self, frame: &mut ratatui::Frame<'_>) {
        let area = frame.area();
        frame.render_widget(Clear, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(2),
            ])
            .split(area);

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(28), Constraint::Min(40)])
            .split(layout[1]);

        let filtered = self.filtered_indices();
        let current = self.current_page();

        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![Span::styled(
                    " NestForge Docs ",
                    Style::default().cyan().add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![Span::styled(
                    if self.search_mode {
                        format!(
                            "Search: {}",
                            if self.query.is_empty() {
                                "_".to_string()
                            } else {
                                self.query.clone()
                            }
                        )
                    } else {
                        format!(
                            "Topic: {}  |  {} results  |  Press / to search",
                            current.title,
                            filtered.len()
                        )
                    },
                    Style::default().gray(),
                )]),
            ])
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().cyan())),
            layout[0],
        );

        let nav_lines = if filtered.is_empty() {
            vec![Line::from(vec![Span::styled(
                "No topics match the current search.",
                Style::default().red(),
            )])]
        } else {
            filtered
                .iter()
                .enumerate()
                .map(|(visible_index, page_index)| {
                    let page = DOC_PAGES[*page_index];
                    let active = visible_index == self.filtered_selected_position();
                    let prefix = if active { "› " } else { "  " };
                    let style = if active {
                        Style::default().yellow().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    Line::from(vec![Span::styled(
                        format!("{prefix}{}", page.title),
                        style,
                    )])
                })
                .collect::<Vec<_>>()
        };

        frame.render_widget(
            Paragraph::new(nav_lines)
                .block(Block::default().title(" Topics ").borders(Borders::ALL)),
            body[0],
        );

        frame.render_widget(
            Paragraph::new(current.body)
                .block(
                    Block::default()
                        .title(format!(" {} ", current.title))
                        .borders(Borders::ALL),
                )
                .wrap(Wrap { trim: false })
                .scroll((self.scroll, 0)),
            body[1],
        );

        frame.render_widget(
            Paragraph::new(match &self.status {
                Some(status) => format!(
                    "{status}  |  j/k topics  PageUp/PageDown scroll  / search  q quit"
                ),
                None => "j/k topics  PageUp/PageDown scroll  / search  q quit".to_string(),
            })
            .style(Style::default().dark_gray()),
            layout[2],
        );
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        if self.search_mode {
            match code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.search_mode = false;
                    self.status = Some(if self.query.trim().is_empty() {
                        "Search cleared.".to_string()
                    } else {
                        format!("Filtered docs by `{}`.", self.query)
                    });
                    self.ensure_selection_visible();
                }
                KeyCode::Backspace => {
                    self.query.pop();
                    self.ensure_selection_visible();
                }
                KeyCode::Char(ch) if !modifiers.contains(KeyModifiers::CONTROL) => {
                    self.query.push(ch);
                    self.ensure_selection_visible();
                }
                _ => {}
            }
            return false;
        }

        match code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('/') => {
                self.search_mode = true;
                self.status = Some("Type to filter docs. Enter applies the search.".to_string());
            }
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('G') => {
                self.select_last();
            }
            KeyCode::Char('g') | KeyCode::Home => {
                self.select_first();
            }
            KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll = self.scroll.saturating_add(8);
            }
            KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll = self.scroll.saturating_sub(8);
            }
            KeyCode::PageDown => {
                self.scroll = self.scroll.saturating_add(12);
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(12);
            }
            _ => {}
        }

        false
    }

    fn filtered_indices(&self) -> Vec<usize> {
        let query = self.query.trim().to_ascii_lowercase();
        if query.is_empty() {
            return (0..DOC_PAGES.len()).collect();
        }

        DOC_PAGES
            .iter()
            .enumerate()
            .filter(|(_, page)| {
                page.slug.contains(&query)
                    || page.title.to_ascii_lowercase().contains(&query)
                    || page.body.to_ascii_lowercase().contains(&query)
            })
            .map(|(index, _)| index)
            .collect()
    }

    fn filtered_selected_position(&self) -> usize {
        self.filtered_indices()
            .iter()
            .position(|index| *index == self.selected)
            .unwrap_or(0)
    }

    fn current_page(&self) -> &'static DocPage {
        let filtered = self.filtered_indices();
        let index = filtered.first().copied().unwrap_or(0);
        let selected = if filtered.contains(&self.selected) {
            self.selected
        } else {
            index
        };
        &DOC_PAGES[selected]
    }

    fn ensure_selection_visible(&mut self) {
        let filtered = self.filtered_indices();
        if filtered.is_empty() {
            self.selected = 0;
            self.scroll = 0;
            return;
        }

        if !filtered.contains(&self.selected) {
            self.selected = filtered[0];
            self.scroll = 0;
        }
    }

    fn move_selection(&mut self, delta: isize) {
        let filtered = self.filtered_indices();
        if filtered.is_empty() {
            return;
        }

        let current = filtered
            .iter()
            .position(|index| *index == self.selected)
            .unwrap_or(0);
        let next = if delta.is_negative() {
            current.saturating_sub(delta.unsigned_abs())
        } else {
            (current + delta as usize).min(filtered.len().saturating_sub(1))
        };
        self.selected = filtered[next];
        self.scroll = 0;
        self.status = None;
    }

    fn select_first(&mut self) {
        if let Some(first) = self.filtered_indices().first().copied() {
            self.selected = first;
            self.scroll = 0;
        }
    }

    fn select_last(&mut self) {
        if let Some(last) = self.filtered_indices().last().copied() {
            self.selected = last;
            self.scroll = 0;
        }
    }
}

fn filtered_doc_pages(topic: Option<&str>) -> Vec<&'static DocPage> {
    match topic.and_then(|value| {
        let normalized = value.trim().to_ascii_lowercase();
        DOC_PAGES.iter().find(|page| {
            page.slug.contains(&normalized) || page.title.to_ascii_lowercase().contains(&normalized)
        })
    }) {
        Some(page) => vec![page],
        None => DOC_PAGES.iter().collect(),
    }
}
