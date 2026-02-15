use crate::api::types::{Activity, Athlete, AthleteStats};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

pub struct App {
    athlete: Option<Athlete>,
    stats: Option<AthleteStats>,
    activities: Vec<Activity>,
    current_view: View,
    selected_activity_index: usize,
    activity_page: u32,
    is_loading: bool,
    has_more_activities: bool,
    scroll_offset: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Activities,
    ActivityDetail,
}

impl App {
    pub fn new() -> Self {
        Self {
            athlete: None,
            stats: None,
            activities: Vec::new(),
            current_view: View::Dashboard,
            selected_activity_index: 0,
            activity_page: 1,
            is_loading: false,
            has_more_activities: true,
            scroll_offset: 0,
        }
    }

    pub fn set_data(
        &mut self,
        athlete: Athlete,
        stats: AthleteStats,
        activities: Vec<Activity>,
        per_page: usize,
    ) {
        let count = activities.len();
        self.athlete = Some(athlete);
        self.stats = Some(stats);
        self.activities = activities;
        self.activity_page = 1;
        self.has_more_activities = count >= per_page;
    }

    pub fn set_view(&mut self, view: View) {
        self.current_view = view;
    }

    pub fn current_view(&self) -> View {
        self.current_view
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    pub fn should_load_more(&self) -> bool {
        !self.is_loading
            && self.has_more_activities
            && self.selected_activity_index >= self.activities.len().saturating_sub(5)
    }

    pub fn add_activities(&mut self, new_activities: Vec<Activity>, per_page: u32) {
        let count = new_activities.len();
        self.activities.extend(new_activities);
        self.activity_page += 1;
        self.has_more_activities = count >= per_page as usize;
        self.is_loading = false;
    }

    pub fn set_load_error(&mut self) {
        self.is_loading = false;
    }

    pub fn activity_page(&self) -> u32 {
        self.activity_page
    }

    pub fn scroll_left(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_right(&mut self) {
        self.scroll_offset += 1;
    }

    pub fn scroll_offset(&self) -> u32 {
        self.scroll_offset
    }

    fn compute_biggest_distance(&self) -> (f64, f64) {
        let all_time = self
            .activities
            .iter()
            .map(|a| a.distance / 1000.0)
            .fold(0.0f64, f64::max);

        let recent: f64 = self
            .activities
            .iter()
            .filter(|a| {
                let thirty_days_ago = chrono::Utc::now() - chrono::Duration::days(30);
                a.start_date_local > thirty_days_ago
            })
            .map(|a| a.distance / 1000.0)
            .sum();

        (all_time, recent)
    }

    fn compute_best_pace(&self) -> (String, String) {
        let all_time_best = self
            .activities
            .iter()
            .filter(|a| a.distance > 0.0 && (a.sport_type == "Run" || a.activity_type == "Run"))
            .map(|a| a.moving_time as f64 / (a.distance / 1000.0))
            .fold(f64::INFINITY, f64::min);

        let recent_activities: Vec<_> = self
            .activities
            .iter()
            .filter(|a| {
                let thirty_days_ago = chrono::Utc::now() - chrono::Duration::days(30);
                a.start_date_local > thirty_days_ago
            })
            .filter(|a| a.sport_type == "Run" || a.activity_type == "Run")
            .filter(|a| a.distance > 0.0)
            .collect();

        let recent_best = recent_activities
            .iter()
            .map(|a| a.moving_time as f64 / (a.distance / 1000.0))
            .fold(f64::INFINITY, f64::min);

        let format_pace = |secs: f64| {
            if secs.is_infinite() || secs == 0.0 {
                "--:--".to_string()
            } else {
                let min = (secs / 60.0) as u32;
                let rem_sec = (secs % 60.0) as u32;
                format!("{}:{:02}", min, rem_sec)
            }
        };

        (format_pace(all_time_best), format_pace(recent_best))
    }

    fn compute_monthly_count(&self) -> (u32, u32) {
        let now = chrono::Utc::now();
        let this_month = now.format("%Y-%m").to_string();
        let prev_month = (now - chrono::Duration::days(35))
            .format("%Y-%m")
            .to_string();

        let this_month_count = self
            .activities
            .iter()
            .filter(|a| a.start_date_local.format("%Y-%m").to_string() == this_month)
            .count() as u32;

        let prev_month_count = self
            .activities
            .iter()
            .filter(|a| a.start_date_local.format("%Y-%m").to_string() == prev_month)
            .count() as u32;

        (this_month_count, prev_month_count)
    }

    pub fn render(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        self.render_header(f, chunks[0]);

        match self.current_view {
            View::Dashboard => self.render_dashboard(f, chunks[1]),
            View::Activities => self.render_activities(f, chunks[1]),
            View::ActivityDetail => self.render_activity_detail(f, chunks[1]),
        }

        self.render_footer(f, chunks[2]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let title = match self.current_view {
            View::Dashboard => "SportFrei - Dashboard",
            View::Activities => "SportFrei - Activities",
            View::ActivityDetail => "SportFrei - Activity Details",
        };

        let block = Block::new().borders(Borders::ALL).title(title);

        f.render_widget(block, area);
    }

    fn render_dashboard(&self, f: &mut Frame, area: Rect) {
        if self.athlete.is_none() {
            let paragraph = Paragraph::new("No data available")
                .style(Style::default().fg(Color::Yellow))
                .block(Block::new().borders(Borders::ALL).title("Dashboard"));
            f.render_widget(paragraph, area);
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(area);

        let (all_time_dist, recent_dist) = self.compute_biggest_distance();
        let (best_pace_all, best_pace_recent) = self.compute_best_pace();
        let (this_month, prev_month) = self.compute_monthly_count();

        let name = self
            .athlete
            .as_ref()
            .map(|a| a.firstname.as_str())
            .unwrap_or("Athlete");
        let title = format!("Welcome, {}!", name);

        let dist_trend = if recent_dist > 0.0 { "↑" } else { "↓" };
        let dist_color = if recent_dist > 0.0 {
            Color::Green
        } else {
            Color::Red
        };

        let pace_all_secs: f64 = best_pace_all.split(':').fold(0.0, |acc, s| {
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.is_empty() {
                return acc;
            }
            let n: u32 = parts[0].parse().unwrap_or(0);
            acc * 60.0 + n as f64
        });
        let pace_recent_secs: f64 = best_pace_recent.split(':').fold(0.0, |acc, s| {
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.is_empty() {
                return acc;
            }
            let n: u32 = parts[0].parse().unwrap_or(0);
            acc * 60.0 + n as f64
        });
        let pace_trend = if pace_recent_secs > 0.0 && pace_recent_secs < pace_all_secs {
            "↑"
        } else {
            "↓"
        };
        let pace_color = if pace_recent_secs > 0.0 && pace_recent_secs < pace_all_secs {
            Color::Green
        } else {
            Color::Red
        };

        let count_trend = if this_month > prev_month {
            "↑"
        } else {
            "↓"
        };
        let count_color = if this_month > prev_month {
            Color::Green
        } else {
            Color::Red
        };

        let widget1 = format!(
            "Biggest Distance\n\n{:.1} km {}\n(last 30 days: {:.1} km)",
            all_time_dist, dist_trend, recent_dist
        );
        let widget2 = format!(
            "Best Pace\n\n{} /km {}\n(vs {})",
            best_pace_recent, pace_trend, best_pace_all
        );
        let widget3 = format!(
            "This Month\n\n{} {}\n(vs {} last month)",
            this_month, count_trend, prev_month
        );

        let block1 = Block::new()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Cyan));
        let block2 = Block::new()
            .borders(Borders::ALL)
            .title("Best Pace")
            .border_style(Style::default().fg(Color::Green));
        let block3 = Block::new()
            .borders(Borders::ALL)
            .title("Activities this month")
            .border_style(Style::default().fg(Color::Yellow));

        let p1 = Paragraph::new(widget1).style(Style::default().fg(dist_color));
        let p2 = Paragraph::new(widget2).style(Style::default().fg(pace_color));
        let p3 = Paragraph::new(widget3).style(Style::default().fg(count_color));

        f.render_widget(block1, chunks[0]);
        f.render_widget(
            p1,
            chunks[0].inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            }),
        );

        f.render_widget(block2, chunks[1]);
        f.render_widget(
            p2,
            chunks[1].inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            }),
        );

        f.render_widget(block3, chunks[2]);
        f.render_widget(
            p3,
            chunks[2].inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            }),
        );
    }

    fn get_activity_color(activity: &Activity) -> Color {
        match activity.sport_type.as_str() {
            "Run" => Color::Green,
            "Ride" => Color::Blue,
            "Swim" => Color::Cyan,
            "Hike" => Color::Yellow,
            "Walk" => Color::Yellow,
            _ => Color::Magenta,
        }
    }

    fn render_activities(&mut self, f: &mut Frame, area: Rect) {
        if self.activities.is_empty() {
            let paragraph = Paragraph::new("No activities found")
                .style(Style::default().fg(Color::White))
                .block(Block::new().borders(Borders::ALL).title("Activities"));
            f.render_widget(paragraph, area);
            return;
        }

        let rows: Vec<Row> = self
            .activities
            .iter()
            .enumerate()
            .map(|(i, activity)| {
                let selected = i == self.selected_activity_index;
                let activity_color = Self::get_activity_color(activity);

                let date = activity.start_date_local.format("%m-%d %H:%M").to_string();
                let name: String = activity.name.chars().take(25).collect();
                let distance = format!("{:.1}", activity.distance / 1000.0);
                let elevation = format!("{:.0}", activity.total_elevation_gain);

                let duration = format!(
                    "{}:{:02}:{:02}",
                    activity.moving_time / 3600,
                    (activity.moving_time % 3600) / 60,
                    activity.moving_time % 60
                );

                let pace = if activity.distance > 0.0 {
                    let pace_seconds = activity.moving_time as f64 / (activity.distance / 1000.0);
                    let pace_min = (pace_seconds / 60.0) as u32;
                    let pace_rem_sec = (pace_seconds % 60.0) as u32;
                    format!("{}:{:02}", pace_min, pace_rem_sec)
                } else {
                    "--:--".to_string()
                };

                let hr = activity
                    .average_heartrate
                    .map(|h| format!("{:.0}", h))
                    .unwrap_or_else(|| "---".to_string());
                let calories = activity
                    .calories
                    .map(|c| format!("{:.0}", c))
                    .unwrap_or_else(|| "---".to_string());

                let rel_perf = if let (Some(avg_speed), Some(avg_hr)) =
                    (activity.average_speed, activity.average_heartrate)
                {
                    if avg_speed > 0.0 {
                        let rp = (activity.distance / avg_speed) / avg_hr;
                        format!("{:.0}", rp)
                    } else {
                        "---".to_string()
                    }
                } else {
                    "---".to_string()
                };

                let row_style = if selected {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default().fg(Color::White)
                };

                Row::new(vec![
                    Cell::from(date).style(row_style),
                    Cell::from(name).style(row_style.fg(activity_color)),
                    Cell::from(distance).style(row_style.fg(Color::Cyan)),
                    Cell::from(elevation).style(row_style),
                    Cell::from(duration).style(row_style.fg(Color::Green)),
                    Cell::from(pace).style(row_style.fg(Color::Yellow)),
                    Cell::from(hr).style(row_style.fg(Color::Red)),
                    Cell::from(calories).style(row_style),
                    Cell::from(rel_perf).style(row_style.fg(Color::Magenta)),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(12),
                Constraint::Length(25),
                Constraint::Length(8),
                Constraint::Length(7),
                Constraint::Length(8),
                Constraint::Length(7),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(7),
            ],
        )
        .header(
            Row::new(vec![
                "Date", "Name", "Distance", "Elev", "Duration", "Pace", "HR", "Cal", "RelPerf",
            ])
            .style(Style::default().fg(Color::White).bg(Color::Black)),
        )
        .block(Block::new().borders(Borders::ALL).title(format!(
            "Activities ({} total) - h/l scroll, j/k nav)",
            self.activities.len()
        )))
        .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

        f.render_widget(table, area);
    }

    fn render_activity_detail(&self, f: &mut Frame, area: Rect) {
        let activity = self.activities.get(self.selected_activity_index);

        let content = if let Some(activity) = activity {
            format!(
                "{}\n\nType: {}\nDistance: {:.2} km\nMoving Time: {}h {}m\nElevation Gain: {:.0} m\nAverage Speed: {:.2} km/h",
                activity.name,
                activity.activity_type,
                activity.distance / 1000.0,
                activity.moving_time / 3600,
                (activity.moving_time % 3600) / 60,
                activity.total_elevation_gain,
                activity.average_speed.unwrap_or(0.0) * 3.6
            )
        } else {
            "No activity selected".to_string()
        };

        let paragraph = Paragraph::new(content)
            .style(Style::default().fg(Color::White))
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .title("Details (Esc to go back)"),
            );

        f.render_widget(paragraph, area);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let nav = "[D]ashboard | [A]ctivities | [Q]uit";

        let block = Block::new().borders(Borders::ALL).title(nav);

        f.render_widget(block, area);
    }

    pub fn select_next_activity(&mut self) {
        if self.activities.is_empty() {
            return;
        }
        self.selected_activity_index =
            (self.selected_activity_index + 1).min(self.activities.len() - 1);
    }

    pub fn select_prev_activity(&mut self) {
        if self.activities.is_empty() {
            return;
        }
        self.selected_activity_index = self.selected_activity_index.saturating_sub(1);
    }

    pub fn get_selected_activity(&self) -> Option<&Activity> {
        self.activities.get(self.selected_activity_index)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
