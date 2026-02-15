use crate::api::types::{Activity, Athlete, AthleteStats};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
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

    pub fn set_data(&mut self, athlete: Athlete, stats: AthleteStats, activities: Vec<Activity>, per_page: usize) {
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
        !self.is_loading && 
        self.has_more_activities && 
        self.selected_activity_index >= self.activities.len().saturating_sub(5)
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
            View::Dashboard => "Strava TUI - Dashboard",
            View::Activities => "Strava TUI - Activities",
            View::ActivityDetail => "Strava TUI - Activity Details",
        };
        
        let block = Block::new()
            .borders(Borders::ALL)
            .title(title);
        
        f.render_widget(block, area);
    }

    fn render_dashboard(&self, f: &mut Frame, area: Rect) {
        if let (Some(stats), Some(athlete)) = (&self.stats, &self.athlete) {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(area);

            let run_stats = format!(
                "Recent Running Stats\n\n\
                 Distance: {:.1} km\n\
                 Time: {}h {}m\n\
                 Activities: {}\n\
                 Elevation: {:.0} m",
                stats.recent_run_totals.distance / 1000.0,
                stats.recent_run_totals.moving_time / 3600,
                (stats.recent_run_totals.moving_time % 3600) / 60,
                stats.recent_run_totals.count,
                stats.recent_run_totals.elevation_gain
            );

            let ride_stats = format!(
                "Recent Cycling Stats\n\n\
                 Distance: {:.1} km\n\
                 Time: {}h {}m\n\
                 Activities: {}\n\
                 Elevation: {:.0} m",
                stats.recent_ride_totals.distance / 1000.0,
                stats.recent_ride_totals.moving_time / 3600,
                (stats.recent_ride_totals.moving_time % 3600) / 60,
                stats.recent_ride_totals.count,
                stats.recent_ride_totals.elevation_gain
            );

            let run_block = Block::new()
                .borders(Borders::ALL)
                .title(format!("Welcome, {}!", athlete.firstname));
            
            let ride_block = Block::new()
                .borders(Borders::ALL)
                .title("Year Totals");

            let run_paragraph = Paragraph::new(run_stats)
                .style(Style::default().fg(Color::Cyan));
            let ride_paragraph = Paragraph::new(ride_stats)
                .style(Style::default().fg(Color::Green));

            f.render_widget(run_block, chunks[0]);
            f.render_widget(run_paragraph, chunks[0].inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            }));
            
            f.render_widget(ride_block, chunks[1]);
            f.render_widget(ride_paragraph, chunks[1].inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            }));
        } else {
            let paragraph = Paragraph::new("No data available")
                .style(Style::default().fg(Color::Yellow))
                .block(Block::new().borders(Borders::ALL).title("Dashboard"));
            f.render_widget(paragraph, area);
        }
    }

    fn render_activities(&mut self, f: &mut Frame, area: Rect) {
        let mut content = String::new();
        
        // Header with proper column widths - match exactly with data row formatting
        content.push_str(" Date   | Time  | Name                       | Distance | Elev   | Pace   | HR    | Cal   | RelPerf\n");
        content.push_str("-------+-------+---------------------------+----------+--------+--------+-------+-------+--------\n");
        
        for (i, activity) in self.activities.iter().enumerate() {
            let selected = if i == self.selected_activity_index { ">" } else { " " };
            
            let date = activity.start_date_local.format("%m-%d").to_string();
            let time = activity.start_date_local.format("%H:%M").to_string();
            let name = activity.name.chars().take(25).collect::<String>();
            let distance = format!("{:.1}", activity.distance / 1000.0);
            let elevation = format!("{:.0}", activity.total_elevation_gain);
            
            let pace = if activity.distance > 0.0 {
                let pace_seconds = activity.moving_time as f64 / (activity.distance / 1000.0);
                let pace_min = (pace_seconds / 60.0) as u32;
                let pace_rem_sec = (pace_seconds % 60.0) as u32;
                format!("{}:{:02}", pace_min, pace_rem_sec)
            } else {
                "--:--".to_string()
            };
            
            let hr = activity.average_heartrate.map(|h| format!("{:.0}", h)).unwrap_or_else(|| "---".to_string());
            let calories = activity.calories.map(|c| format!("{:.0}", c)).unwrap_or_else(|| "---".to_string());
            
            let rel_perf = if let (Some(avg_speed), Some(avg_hr)) = (activity.average_speed, activity.average_heartrate) {
                if avg_speed > 0.0 {
                    // (distance / speed) / heartrate = seconds / heartrate
                    let rp = (activity.distance / avg_speed) / avg_hr;
                    format!("{:.0}", rp)
                } else {
                    "---".to_string()
                }
            } else {
                "---".to_string()
            };
            
            // Format with exact widths to match header
            content.push_str(&format!(
                "{} {:>5} | {:>5} | {:<25} | {:>8} | {:>7} | {:>7} | {:>5} | {:>5} | {:>7}\n",
                selected,
                date,
                time,
                name,
                distance,
                elevation,
                pace,
                hr,
                calories,
                rel_perf
            ));
        }

        if self.is_loading {
            content.push_str("\n  Loading more activities...");
        } else if !self.has_more_activities && !self.activities.is_empty() {
            content.push_str("\n  -- End of activities --");
        }

        if self.activities.is_empty() {
            content = "No activities found".to_string();
        }

        let total = self.activities.len();
        let paragraph = Paragraph::new(content)
            .style(Style::default().fg(Color::White))
            .block(Block::new().borders(Borders::ALL).title(format!("Activities ({} total) - h/l scroll, j/k nav)", total)))
            .scroll((0, self.scroll_offset as u16));

        f.render_widget(paragraph, area);
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
            .block(Block::new().borders(Borders::ALL).title("Details (Esc to go back)"));
        
        f.render_widget(paragraph, area);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let nav = "[D]ashboard | [A]ctivities | [Q]uit";

        let block = Block::new()
            .borders(Borders::ALL)
            .title(nav);
        
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
