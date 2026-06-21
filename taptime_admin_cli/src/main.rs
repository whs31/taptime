use std::{error::Error, io, time::Duration};

use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use chrono::{TimeZone, Utc};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand_core::OsRng;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
};
use taptime_schema::{
    User, Uuid as ProtoUuid,
    services::{
        AdminLoginRequest, AdminUserDetail, BanKind, BanRecord, CreateBanRequest,
        DeleteUserRequest, ListBansRequest, ListUsersRequest, RevokeBanRequest,
        admin_service_client::AdminServiceClient,
    },
};
use tonic::{Request, metadata::MetadataValue, transport::Channel};
use uuid::Uuid;

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Parser)]
#[command(name = "taptime_admin_cli")]
#[command(version, about = "TapTime administration TUI")]
struct Args {
    #[arg(long, env = "ADMIN_API_URL", default_value = "http://127.0.0.1:50051")]
    admin_api_url: String,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    HashPassword {
        #[arg(long)]
        password: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Tab {
    Users,
    Bans,
}

#[derive(Clone, Debug)]
enum Mode {
    Normal,
    Search { input: String },
    BanIp { input: String },
    Confirm { action: Action, input: String },
}

#[derive(Clone, Debug)]
enum Action {
    BanUser(ProtoUuid),
    BanIp(String),
    RevokeBan(BanKind, ProtoUuid),
    DeleteData(ProtoUuid),
    DeleteAccount(ProtoUuid),
}

impl Action {
    fn expected(&self) -> &'static str {
        match self {
            Self::BanUser(_) => "BAN USER",
            Self::BanIp(_) => "BAN IP",
            Self::RevokeBan(_, _) => "REVOKE BAN",
            Self::DeleteData(_) => "DELETE DATA",
            Self::DeleteAccount(_) => "DELETE ACCOUNT",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::BanUser(_) => "Ban selected user",
            Self::BanIp(_) => "Ban IP/CIDR",
            Self::RevokeBan(_, _) => "Revoke selected ban",
            Self::DeleteData(_) => "Delete selected user's time data",
            Self::DeleteAccount(_) => "Delete selected user account",
        }
    }
}

struct App {
    client: AdminClient,
    tab: Tab,
    mode: Mode,
    users: Vec<taptime_schema::services::AdminUserListItem>,
    bans: Vec<BanRecord>,
    detail: Option<AdminUserDetail>,
    query: String,
    selected_user: usize,
    selected_ban: usize,
    status: String,
    should_quit: bool,
}

impl App {
    async fn new(client: AdminClient) -> AppResult<Self> {
        let mut app = Self {
            client,
            tab: Tab::Users,
            mode: Mode::Normal,
            users: Vec::new(),
            bans: Vec::new(),
            detail: None,
            query: String::new(),
            selected_user: 0,
            selected_ban: 0,
            status: String::from("Ready"),
            should_quit: false,
        };
        app.refresh().await?;
        Ok(app)
    }

    async fn refresh(&mut self) -> AppResult<()> {
        let users = self.client.list_users(&self.query).await?;
        self.users = users.users;
        if self.selected_user >= self.users.len() {
            self.selected_user = self.users.len().saturating_sub(1);
        }
        self.bans = self.client.list_bans(false).await?;
        if self.selected_ban >= self.bans.len() {
            self.selected_ban = self.bans.len().saturating_sub(1);
        }
        self.load_selected_detail().await?;
        self.status = "Refreshed".to_string();
        Ok(())
    }

    async fn load_selected_detail(&mut self) -> AppResult<()> {
        let Some(user_id) = self.selected_user_id() else {
            self.detail = None;
            return Ok(());
        };
        self.detail = Some(self.client.get_user_detail(user_id).await?);
        Ok(())
    }

    fn selected_user_id(&self) -> Option<ProtoUuid> {
        self.users.get(self.selected_user)?.user.as_ref()?.id
    }

    fn selected_ban(&self) -> Option<&BanRecord> {
        self.bans.get(self.selected_ban)
    }

    async fn handle_key(&mut self, key: KeyEvent) -> AppResult<()> {
        match &mut self.mode {
            Mode::Search { input } => match key.code {
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Enter => {
                    self.query = input.trim().to_string();
                    self.mode = Mode::Normal;
                    self.refresh().await?;
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(ch) => input.push(ch),
                _ => {}
            },
            Mode::BanIp { input } => match key.code {
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Enter => {
                    let ip = input.trim().to_string();
                    if !ip.is_empty() {
                        self.mode = Mode::Confirm {
                            action: Action::BanIp(ip),
                            input: String::new(),
                        };
                    }
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(ch) => input.push(ch),
                _ => {}
            },
            Mode::Confirm { action, input } => match key.code {
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Enter => {
                    if input == action.expected() {
                        let action = action.clone();
                        self.mode = Mode::Normal;
                        self.perform(action).await?;
                    } else {
                        self.status = format!("Confirmation must be exactly {}", action.expected());
                    }
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(ch) => input.push(ch),
                _ => {}
            },
            Mode::Normal => self.handle_normal_key(key).await?,
        }
        Ok(())
    }

    async fn handle_normal_key(&mut self, key: KeyEvent) -> AppResult<()> {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => {
                self.tab = if self.tab == Tab::Users {
                    Tab::Bans
                } else {
                    Tab::Users
                }
            }
            KeyCode::Char('/') => {
                self.mode = Mode::Search {
                    input: self.query.clone(),
                };
            }
            KeyCode::Char('p') => {
                self.mode = Mode::BanIp {
                    input: String::new(),
                };
            }
            KeyCode::Char('r') => self.refresh().await?,
            KeyCode::Enter if self.tab == Tab::Users => self.load_selected_detail().await?,
            KeyCode::Char('b') if self.tab == Tab::Users => {
                if let Some(user_id) = self.selected_user_id() {
                    self.mode = Mode::Confirm {
                        action: Action::BanUser(user_id),
                        input: String::new(),
                    };
                }
            }
            KeyCode::Char('d') if self.tab == Tab::Users => {
                if let Some(user_id) = self.selected_user_id() {
                    self.mode = Mode::Confirm {
                        action: Action::DeleteData(user_id),
                        input: String::new(),
                    };
                }
            }
            KeyCode::Char('x') if self.tab == Tab::Users => {
                if let Some(user_id) = self.selected_user_id() {
                    self.mode = Mode::Confirm {
                        action: Action::DeleteAccount(user_id),
                        input: String::new(),
                    };
                }
            }
            KeyCode::Char('u') if self.tab == Tab::Bans => {
                if let Some(ban) = self.selected_ban() {
                    if let Some(ban_id) = ban.id {
                        let kind = BanKind::try_from(ban.kind).unwrap_or(BanKind::Unspecified);
                        self.mode = Mode::Confirm {
                            action: Action::RevokeBan(kind, ban_id),
                            input: String::new(),
                        };
                    }
                }
            }
            KeyCode::Up => self.move_selection(-1).await?,
            KeyCode::Down => self.move_selection(1).await?,
            _ => {}
        }
        Ok(())
    }

    async fn move_selection(&mut self, delta: isize) -> AppResult<()> {
        match self.tab {
            Tab::Users => {
                self.selected_user = move_index(self.selected_user, self.users.len(), delta);
                self.load_selected_detail().await?;
            }
            Tab::Bans => {
                self.selected_ban = move_index(self.selected_ban, self.bans.len(), delta);
            }
        }
        Ok(())
    }

    async fn perform(&mut self, action: Action) -> AppResult<()> {
        match action {
            Action::BanUser(user_id) => {
                self.client.create_user_ban(user_id).await?;
                self.status = "User banned".to_string();
            }
            Action::BanIp(ip_cidr) => {
                self.client.create_ip_ban(ip_cidr).await?;
                self.status = "IP/CIDR banned".to_string();
            }
            Action::RevokeBan(kind, ban_id) => {
                self.client.revoke_ban(kind, ban_id).await?;
                self.status = "Ban revoked".to_string();
            }
            Action::DeleteData(user_id) => {
                self.client.delete_user_time_data(user_id).await?;
                self.status = "Time data deleted".to_string();
            }
            Action::DeleteAccount(user_id) => {
                self.client.delete_user_account(user_id).await?;
                self.status = "Account deleted".to_string();
            }
        }
        self.refresh().await?;
        Ok(())
    }
}

struct AdminClient {
    inner: AdminServiceClient<Channel>,
    token: String,
}

impl AdminClient {
    async fn connect(endpoint: String, password: String) -> AppResult<Self> {
        let mut inner = AdminServiceClient::connect(endpoint).await?;
        let response = inner
            .admin_login(AdminLoginRequest { password })
            .await?
            .into_inner();
        Ok(Self {
            inner,
            token: response.admin_token,
        })
    }

    fn request<T>(&self, message: T) -> Result<Request<T>, tonic::Status> {
        let mut request = Request::new(message);
        let header = format!("Bearer {}", self.token)
            .parse::<MetadataValue<_>>()
            .map_err(|_| tonic::Status::internal("Invalid admin token"))?;
        request.metadata_mut().insert("authorization", header);
        Ok(request)
    }

    async fn list_users(
        &mut self,
        query: &str,
    ) -> AppResult<taptime_schema::services::ListUsersResponse> {
        Ok(self
            .inner
            .list_users(self.request(ListUsersRequest {
                query: query.to_string(),
                limit: 100,
                offset: 0,
            })?)
            .await?
            .into_inner())
    }

    async fn get_user_detail(&mut self, user_id: ProtoUuid) -> AppResult<AdminUserDetail> {
        Ok(self
            .inner
            .get_user_detail(
                self.request(taptime_schema::services::GetUserDetailRequest {
                    user_id: Some(user_id),
                })?,
            )
            .await?
            .into_inner())
    }

    async fn list_bans(&mut self, include_inactive: bool) -> AppResult<Vec<BanRecord>> {
        Ok(self
            .inner
            .list_bans(self.request(ListBansRequest {
                kind: BanKind::Unspecified as i32,
                include_inactive,
            })?)
            .await?
            .into_inner()
            .bans)
    }

    async fn create_user_ban(&mut self, user_id: ProtoUuid) -> AppResult<()> {
        self.inner
            .create_ban(self.request(CreateBanRequest {
                kind: BanKind::User as i32,
                user_id: Some(user_id),
                ip_cidr: String::new(),
                reason: "Admin ban".to_string(),
                expires_at: None,
            })?)
            .await?;
        Ok(())
    }

    async fn create_ip_ban(&mut self, ip_cidr: String) -> AppResult<()> {
        self.inner
            .create_ban(self.request(CreateBanRequest {
                kind: BanKind::Ip as i32,
                user_id: None,
                ip_cidr,
                reason: "Admin ban".to_string(),
                expires_at: None,
            })?)
            .await?;
        Ok(())
    }

    async fn revoke_ban(&mut self, kind: BanKind, ban_id: ProtoUuid) -> AppResult<()> {
        self.inner
            .revoke_ban(self.request(RevokeBanRequest {
                ban_id: Some(ban_id),
                kind: kind as i32,
            })?)
            .await?;
        Ok(())
    }

    async fn delete_user_time_data(&mut self, user_id: ProtoUuid) -> AppResult<()> {
        self.inner
            .delete_user_time_data(self.request(DeleteUserRequest {
                user_id: Some(user_id),
            })?)
            .await?;
        Ok(())
    }

    async fn delete_user_account(&mut self, user_id: ProtoUuid) -> AppResult<()> {
        self.inner
            .delete_user_account(self.request(DeleteUserRequest {
                user_id: Some(user_id),
            })?)
            .await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let args = Args::parse();
    match args.command {
        Some(Command::HashPassword { password }) => {
            let password = match password {
                Some(password) => password,
                None => read_password("Admin password: ")?,
            };
            println!("{}", hash_password(&password)?);
            Ok(())
        }
        None => {
            let password = read_password("Admin password: ")?;
            let client = AdminClient::connect(args.admin_api_url, password).await?;
            run_tui(client).await
        }
    }
}

async fn run_tui(client: AdminClient) -> AppResult<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new(client).await?;

    let result = loop {
        terminal.draw(|frame| draw(frame, &app))?;
        if app.should_quit {
            break Ok(());
        }
        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(key) = event::read()? {
                if let Err(err) = app.handle_key(key).await {
                    app.status = err.to_string();
                }
            }
        }
    };

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn draw(frame: &mut Frame, app: &App) {
    let [tabs_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let selected_tab = if app.tab == Tab::Users { 0 } else { 1 };
    let tabs = Tabs::new(["Users", "Bans"])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("TapTime Admin"),
        )
        .select(selected_tab)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, tabs_area);

    match app.tab {
        Tab::Users => draw_users(frame, body_area, app),
        Tab::Bans => draw_bans(frame, body_area, app),
    }
    draw_footer(frame, footer_area, app);
    draw_mode(frame, app);
}

fn draw_users(frame: &mut Frame, area: Rect, app: &App) {
    let [list_area, detail_area] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).areas(area);
    let items = app.users.iter().map(|item| {
        let user = item.user.as_ref();
        let marker = if item.user_banned { " banned" } else { "" };
        ListItem::new(vec![
            Line::from(format!(
                "{}{}",
                user.map(|u| u.email.as_str()).unwrap_or("<missing user>"),
                marker
            )),
            Line::from(Span::styled(
                user.map(|u| u.name.as_str()).unwrap_or(""),
                Style::default().fg(Color::DarkGray),
            )),
        ])
    });
    let mut state = ListState::default();
    if !app.users.is_empty() {
        state.select(Some(app.selected_user));
    }
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Users [{}]", app.query)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));
    frame.render_stateful_widget(list, list_area, &mut state);

    let detail = match &app.detail {
        Some(detail) => detail_lines(detail),
        None => vec![Line::from("No user selected")],
    };
    let paragraph = Paragraph::new(detail)
        .block(Block::default().borders(Borders::ALL).title("Detail"))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, detail_area);
}

fn draw_bans(frame: &mut Frame, area: Rect, app: &App) {
    let items = app.bans.iter().map(|ban| {
        let kind = ban_kind_label(ban);
        let subject = ban_subject(ban);
        ListItem::new(vec![
            Line::from(format!("{kind} {subject}")),
            Line::from(Span::styled(
                format!("{} {}", active_label(ban), ban.reason),
                Style::default().fg(Color::DarkGray),
            )),
        ])
    });
    let mut state = ListState::default();
    if !app.bans.is_empty() {
        state.select(Some(app.selected_ban));
    }
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Active Bans"))
        .highlight_style(Style::default().bg(Color::DarkGray));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let help = match app.tab {
        Tab::Users => {
            "q quit | tab bans | / search | r refresh | b ban user | p ban ip | d delete data | x delete account"
        }
        Tab::Bans => "q quit | tab users | r refresh | u revoke selected ban | p ban ip",
    };
    let text = vec![Line::from(help), Line::from(app.status.as_str())];
    frame.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Status")),
        area,
    );
}

fn draw_mode(frame: &mut Frame, app: &App) {
    let (title, text) = match &app.mode {
        Mode::Normal => return,
        Mode::Search { input } => ("Search users", format!("Query: {input}")),
        Mode::BanIp { input } => ("Ban IP/CIDR", format!("IP/CIDR: {input}")),
        Mode::Confirm { action, input } => (
            action.label(),
            format!("Type {} to confirm: {input}", action.expected()),
        ),
    };
    let area = centered_rect(62, 24, frame.area());
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(title))
            .alignment(Alignment::Center),
        area,
    );
}

fn detail_lines(detail: &AdminUserDetail) -> Vec<Line<'static>> {
    let Some(user) = &detail.user else {
        return vec![Line::from("Missing user")];
    };
    let mut lines = vec![
        Line::from(format!("Name: {}", user.name)),
        Line::from(format!("Email: {}", user.email)),
        Line::from(format!(
            "Organization: {}",
            user.organization.as_deref().unwrap_or("-")
        )),
        Line::from(format!("ID: {}", user_id(user))),
        Line::from(format!("Created: {}", fmt_ts(user.created_at))),
        Line::from(format!("Last seen: {}", fmt_ts(user.last_seen))),
        Line::from(format!("Events: {}", detail.event_count)),
        Line::from(format!("Flag days: {}", detail.day_flag_count)),
        Line::from(""),
        Line::from("Known IPs:"),
    ];
    if detail.known_ips.is_empty() {
        lines.push(Line::from("  none"));
    } else {
        for ip in &detail.known_ips {
            lines.push(Line::from(format!(
                "  {} seen {}x last {}",
                ip.ip,
                ip.request_count,
                fmt_ts(ip.last_seen)
            )));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from("User bans:"));
    if detail.bans.is_empty() {
        lines.push(Line::from("  none"));
    } else {
        for ban in &detail.bans {
            lines.push(Line::from(format!(
                "  {} {} {}",
                active_label(ban),
                fmt_ts(ban.created_at),
                ban.reason
            )));
        }
    }
    lines
}

fn user_id(user: &User) -> String {
    user.id
        .as_ref()
        .map(proto_uuid)
        .unwrap_or_else(|| "-".to_string())
}

fn proto_uuid(id: &ProtoUuid) -> String {
    let id: Uuid = id.into();
    id.to_string()
}

fn fmt_ts(value: Option<prost_types::Timestamp>) -> String {
    value
        .and_then(|ts| Utc.timestamp_opt(ts.seconds, ts.nanos as u32).single())
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn ban_kind_label(ban: &BanRecord) -> &'static str {
    match BanKind::try_from(ban.kind).unwrap_or(BanKind::Unspecified) {
        BanKind::User => "USER",
        BanKind::Ip => "IP",
        BanKind::Unspecified => "BAN",
    }
}

fn ban_subject(ban: &BanRecord) -> String {
    match BanKind::try_from(ban.kind).unwrap_or(BanKind::Unspecified) {
        BanKind::User => ban
            .user_id
            .as_ref()
            .map(proto_uuid)
            .unwrap_or_else(|| "-".to_string()),
        BanKind::Ip => ban.ip_cidr.clone(),
        BanKind::Unspecified => "-".to_string(),
    }
}

fn active_label(ban: &BanRecord) -> &'static str {
    if ban.active { "active" } else { "inactive" }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let [_, area, _] = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .areas(area);
    let [_, area, _] = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .areas(area);
    area
}

fn move_index(current: usize, len: usize, delta: isize) -> usize {
    if len == 0 {
        return 0;
    }
    (current as isize + delta).clamp(0, len as isize - 1) as usize
}

fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Hashing failed: {e}"))?
        .to_string())
}

fn read_password(prompt: &str) -> AppResult<String> {
    print!("{prompt}");
    use std::io::Write;
    io::stdout().flush()?;
    enable_raw_mode()?;
    let mut password = String::new();
    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => break,
                KeyCode::Backspace => {
                    password.pop();
                }
                KeyCode::Char(ch) => password.push(ch),
                KeyCode::Esc => {
                    disable_raw_mode()?;
                    return Err("Cancelled".into());
                }
                _ => {}
            }
        }
    }
    disable_raw_mode()?;
    println!();
    Ok(password)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_index_clamps_to_valid_range() {
        assert_eq!(move_index(0, 0, 1), 0);
        assert_eq!(move_index(0, 3, -1), 0);
        assert_eq!(move_index(1, 3, 1), 2);
        assert_eq!(move_index(2, 3, 1), 2);
    }

    #[test]
    fn confirmation_words_match_plan() {
        assert_eq!(
            Action::DeleteData(ProtoUuid::default()).expected(),
            "DELETE DATA"
        );
        assert_eq!(
            Action::DeleteAccount(ProtoUuid::default()).expected(),
            "DELETE ACCOUNT"
        );
        assert_eq!(Action::BanIp("127.0.0.1".into()).expected(), "BAN IP");
    }
}
