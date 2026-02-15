# Dashboard Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace two-box dashboard with three compact widgets showing Biggest Distance, Best Pace, and Monthly Activity Count with trends.

**Architecture:** Modify `render_dashboard()` in `src/ui/app.rs` to use horizontal layout with 3 equal widgets. Add helper functions to compute metrics from activities.

**Tech Stack:** Rust, ratatui (TUI library)

---

### Task 1: Add helper methods to App for computing dashboard metrics

**Files:**
- Modify: `src/ui/app.rs`

**Step 1: Read existing tests to understand structure**

Run: `ls /Users/dennis/coding/strava-tui/tests/`
Expected: ui_test.rs exists

**Step 2: Add helper methods to App struct**

Add these methods after line 100 in `src/ui/app.rs`:

```rust
fn compute_biggest_distance(&self) -> (f64, f64) {
    let stats = match &self.stats {
        Some(s) => s,
        None => return (0.0, 0.0),
    };
    
    let all_time = stats.biggest_ride_distance.unwrap_or(0.0) / 1000.0;
    
    let recent: f64 = self.activities
        .iter()
        .filter(|a| a.sport_type == "Ride" || a.activity_type == "Ride")
        .filter(|a| {
            let thirty_days_ago = chrono::Utc::now() - chrono::Duration::days(30);
            a.start_date_local > thirty_days_ago
        })
        .map(|a| a.distance / 1000.0)
        .sum();
    
    (all_time, recent)
}

fn compute_best_pace(&self) -> (String, String) {
    let all_time_best = self.activities
        .iter()
        .filter(|a| a.distance > 0.0 && (a.sport_type == "Run" || a.activity_type == "Run"))
        .map(|a| a.moving_time as f64 / (a.distance / 1000.0))
        .fold(f64::INFINITY, f64::min);
    
    let recent_activities: Vec<_> = self.activities
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
    let prev_month = (now - chrono::Duration::days(35)).format("%Y-%m").to_string();
    
    let this_month_count = self.activities
        .iter()
        .filter(|a| a.start_date_local.format("%Y-%m").to_string() == this_month)
        .count() as u32;
    
    let prev_month_count = self.activities
        .iter()
        .filter(|a| a.start_date_local.format("%Y-%m").to_string() == prev_month)
        .count() as u32;
    
    (this_month_count, prev_month_count)
}
```

**Step 3: Run cargo check to verify**

Run: `cd /Users/dennis/coding/strava-tui && cargo check`
Expected: No errors (may need to add chrono to Cargo.toml)

**Step 4: Add chrono dependency if needed**

If cargo check fails with "cannot find crate chrono", add to Cargo.toml:
```toml
chrono = "0.4"
```

Run: `cd /Users/dennis/coding/strava-tui && cargo check`
Expected: PASS

---

### Task 2: Rewrite render_dashboard to use 3-widget layout

**Files:**
- Modify: `src/ui/app.rs:137-203`

**Step 1: Replace render_dashboard method**

Replace the entire `render_dashboard` function (lines 137-203) with:

```rust
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

    let name = self.athlete.as_ref().map(|a| &a.firstname).unwrap_or("Athlete");
    let title = format!("Welcome, {}!", name);

    let dist_trend = if recent_dist > all_time_dist / 12.0 { "▲" } else { "▼" };
    let dist_color = if recent_dist > all_time_dist / 12.0 { Color::Green } else { Color::Red };
    
    let pace_all_secs: f64 = best_pace_all.split(':').fold(0, |acc, s| {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() { return acc; }
        let n: u32 = parts[0].parse().unwrap_or(0);
        acc * 60 + n
    });
    let pace_recent_secs: f64 = best_pace_recent.split(':').fold(0, |acc, s| {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() { return acc; }
        let n: u32 = parts[0].parse().unwrap_or(0);
        acc * 60 + n
    });
    let pace_trend = if pace_recent_secs > 0.0 && pace_recent_secs < pace_all_secs { "▲" } else { "▼" };
    let pace_color = if pace_recent_secs > 0.0 && pace_recent_secs < pace_all_secs { Color::Green } else { Color::Red };

    let count_trend = if this_month > prev_month { "▲" } else { "▼" };
    let count_color = if this_month > prev_month { Color::Green } else { Color::Red };

    let widget1 = format!(
        "Biggest Distance\n\n{:.1} km {}\n(vs {:.1} km avg)",
        recent_dist, dist_trend, all_time_dist / 12.0
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
        .title("This Month")
        .border_style(Style::default().fg(Color::Yellow));

    let p1 = Paragraph::new(widget1).style(Style::default().fg(dist_color));
    let p2 = Paragraph::new(widget2).style(Style::default().fg(pace_color));
    let p3 = Paragraph::new(widget3).style(Style::default().fg(count_color));

    f.render_widget(block1, chunks[0]);
    f.render_widget(p1, chunks[0].inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 }));
    
    f.render_widget(block2, chunks[1]);
    f.render_widget(p2, chunks[1].inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 }));
    
    f.render_widget(block3, chunks[2]);
    f.render_widget(p3, chunks[2].inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 }));
}
```

**Step 2: Run cargo check**

Run: `cd /Users/dennis/coding/strava-tui && cargo check`
Expected: PASS

**Step 3: Run tests**

Run: `cd /Users/dennis/coding/strava-tui && cargo test`
Expected: All tests pass

---

### Task 3: Update dashboard tests

**Files:**
- Modify: `tests/ui_test.rs:263-365`

**Step 1: Read existing dashboard test**

Run: `rg -n "test_dashboard" /Users/dennis/coding/strava-tui/tests/ui_test.rs`
Expected: Find test functions

**Step 2: Update test expectations**

The old tests check for "Recent Running Stats" and "Recent Cycling Stats" - these no longer exist. Update or add new tests that verify:
- Dashboard renders 3 widgets
- Widget titles are present
- Values are displayed correctly

Run: `cd /Users/dennis/coding/strava-tui && cargo test`
Expected: Tests pass (may need to adjust expectations)

---

### Task 4: Run full test suite and verify

**Step 1: Run all tests**

Run: `cd /Users/dennis/coding/strava-tui && cargo test`
Expected: All tests pass

**Step 2: Run clippy**

Run: `cd /Users/dennis/coding/strava-tui && cargo clippy`
Expected: No warnings

---

### Task 5: Commit changes

Run:
```bash
cd /Users/dennis/coding/strava-tui
git add -A
git commit -m "feat: redesign dashboard with 3 metric widgets"
```

Expected: Commit created successfully
