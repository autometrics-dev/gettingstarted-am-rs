use anyhow::{anyhow, Result};
use gettingstarted_am_rs::tui::{Event, Tui};
use rand::Rng;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::join;
use tokio::spawn;

#[tokio::main]
async fn main() -> Result<()> {
    // Concurrency: num

    let data = vec![];
    let shared_data = Arc::new(RwLock::new(data));

    let copied_data = shared_data.clone();
    let data_generator = spawn(async move {
        let start = std::time::Instant::now();
        let shared_data = copied_data;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            let elapsed = start.elapsed();

            let mut rng = rand::thread_rng();

            let mut data = shared_data.write().unwrap();
            data.push((elapsed.as_secs_f64(), rng.gen_range(0.0..10.0)));
        }
    });

    let mut app = LoadApp::new(shared_data);

    join!(data_generator, app.run());

    Ok(())
}

#[derive(Debug)]
struct LoadApp {
    state: AppState,
    shared_data: Arc<RwLock<Vec<(f64, f64)>>>,
}

impl LoadApp {
    fn new(shared_data: Arc<RwLock<Vec<(f64, f64)>>>) -> Self {
        Self {
            state: AppState::default(),
            shared_data,
        }
    }

    async fn run(&mut self) -> Result<()> {
        let tui = Tui::new()?;
        let mut tui = tui.tick_rate(0.3);
        tui.enter()?;
        while !self.state.is_quitting() {
            tui.draw(|f| self.ui(f).expect("Unexpected error during drawing"))?;
            let event = tui.next().await.ok_or_else(|| anyhow!("blargh"))?; // blocks until next event
            let message = self.handle_event(event)?;
            self.update(message)?;
        }
        tui.exit()?;
        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) -> Result<()> {
        let layout = self.layout(f.size());
        f.render_widget(
            Paragraph::new("Load Example").block(Block::default().borders(Borders::ALL)),
            layout[0],
        );
        let data = self.shared_data.read().unwrap();

        let datasets = vec![Dataset::default()
            .name("data2")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Magenta))
            .data(&data)];

        let chart_widget = Chart::new(datasets)
            .block(Block::default().title("Chart"))
            .x_axis(
                Axis::default()
                    .title(Span::styled("X Axis", Style::default().fg(Color::Red)))
                    .style(Style::default().fg(Color::White))
                    .bounds([0.0, 10.0])
                    .labels(
                        ["0.0", "5.0", "10.0"]
                            .iter()
                            .cloned()
                            .map(Span::from)
                            .collect(),
                    ),
            )
            .y_axis(
                Axis::default()
                    .title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
                    .style(Style::default().fg(Color::White))
                    .bounds([0.0, 10.0])
                    .labels(
                        ["0.0", "5.0", "10.0"]
                            .iter()
                            .cloned()
                            .map(Span::from)
                            .collect(),
                    ),
            );

        f.render_widget(chart_widget, layout[1]);

        // f.render_widget(self.fps_paragraph(), layout[1]);
        // f.render_widget(self.timer_paragraph(), layout[2]);
        // f.render_widget(Paragraph::new("Splits:"), layout[3]);
        // f.render_widget(self.splits_paragraph(), layout[4]);
        // f.render_widget(self.help_paragraph(), layout[5]);
        Ok(())
    }

    fn layout(&self, area: Rect) -> Vec<Rect> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),  // top bar
                Constraint::Length(20), // psuedo panel
            ])
            .split(area);

        // return a new vec with the top_layout rects and then rest of layout
        layout[..].iter().copied().collect()
    }

    fn handle_event(&self, event: Event) -> Result<Message> {
        let msg = match event {
            Event::Key(key) => match key.code {
                crossterm::event::KeyCode::Char('q') => Message::Quit,
                crossterm::event::KeyCode::Char(' ') => Message::StartOrSplit,
                crossterm::event::KeyCode::Char('s') | crossterm::event::KeyCode::Enter => {
                    Message::Stop
                }
                _ => Message::Tick,
            },
            _ => Message::Tick,
        };
        Ok(msg)
    }

    fn update(&mut self, message: Message) -> Result<()> {
        match message {
            Message::StartOrSplit => (),
            Message::Stop => (),
            Message::Tick => (),
            Message::Quit => self.quit(),
        }
        Ok(())
    }

    fn quit(&mut self) {
        self.state = AppState::Quitting;
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Paused,

    Running,

    Quitting,
}
impl AppState {
    fn is_quitting(&self) -> bool {
        match self {
            Self::Quitting => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Message {
    StartOrSplit,
    Stop,
    Tick,
    Quit,
}
