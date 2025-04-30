use iced::{Application, Command, Element, executor, Settings, Subscription, Theme};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartWidget};
use iced::widget::{Row, Container};
use iced::Color;
use serde::Deserialize;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const HISTORY_LEN: usize = 60;

#[derive(Debug, Clone, Deserialize)]
struct MemoryJson {
    total_phys: u64,
    free_phys: u64,
    memory_load: u32,
    total_virtual: u64,
    free_virtual: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct DiskJson {
    mount: String,
    total_space: u64,
    free_space: u64,
    used_space: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct TelemetryJsonFull {
    uptime: f64,
    user_time: f64,
    system_time: f64,
    idle_time: f64,
    mem: MemoryJson,
    disks: Vec<DiskJson>,
}

#[derive(Debug, Clone)]
enum Message {
    Telemetry(Option<TelemetryJsonFull>),
    Tick(Instant),
}

struct History {
    cpu_user: VecDeque<f64>,
    cpu_system: VecDeque<f64>,
    cpu_idle: VecDeque<f64>,
    mem_load: VecDeque<u32>,
}

impl History {
    fn new() -> Self {
        Self {
            cpu_user: VecDeque::from(vec![0.0; HISTORY_LEN]),
            cpu_system: VecDeque::from(vec![0.0; HISTORY_LEN]),
            cpu_idle: VecDeque::from(vec![0.0; HISTORY_LEN]),
            mem_load: VecDeque::from(vec![0; HISTORY_LEN]),
        }
    }
    fn push(&mut self, t: &TelemetryJsonFull) {
        if self.cpu_user.len() >= HISTORY_LEN { self.cpu_user.pop_front(); }
        if self.cpu_system.len() >= HISTORY_LEN { self.cpu_system.pop_front(); }
        if self.cpu_idle.len() >= HISTORY_LEN { self.cpu_idle.pop_front(); }
        if self.mem_load.len() >= HISTORY_LEN { self.mem_load.pop_front(); }
        self.cpu_user.push_back(t.user_time);
        self.cpu_system.push_back(t.system_time);
        self.cpu_idle.push_back(t.idle_time);
        self.mem_load.push_back(t.mem.memory_load);
    }
}

struct Dashboard {
    telemetry: Option<TelemetryJsonFull>,
    history: History,
}

impl Application for Dashboard {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Dashboard { telemetry: None, history: History::new() }, Command::none())
    }

    fn title(&self) -> String {
        String::from("System Telemetry Dashboard")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Telemetry(data) => {
                if let Some(ref t) = data {
                    self.history.push(t);
                }
                self.telemetry = data;
                Command::none()
            }
            Message::Tick(_) => {
                // Fetch telemetry every second
                Command::perform(fetch_telemetry(), Message::Telemetry)
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let bg = Color::from_rgb(0.13, 0.15, 0.18);
        let mut col = Column::new().push(Text::new("System Telemetry Dashboard").size(40).style(iced::theme::Text::Color(Color::from_rgb(0.7,0.8,1.0))));
        if let Some(t) = &self.telemetry {
            col = col
                .push(Text::new(format!("Uptime: {:.0} s", t.uptime)).style(iced::theme::Text::Color(Color::from_rgb(0.7,0.8,1.0))))
                .push(Row::new()
                    .push(Container::new(ChartWidget::new(CpuChart { history: self.history.cpu_user.clone(), label: "CPU User", color: RGBColor(0x7b,0x7b,0xff) })).width(iced::Length::FillPortion(1)).padding(10))
                    .push(Container::new(ChartWidget::new(CpuChart { history: self.history.cpu_system.clone(), label: "CPU System", color: RGBColor(0x7b,0xff,0x7b) })).width(iced::Length::FillPortion(1)).padding(10))
                    .push(Container::new(ChartWidget::new(CpuChart { history: self.history.cpu_idle.clone(), label: "CPU Idle", color: RGBColor(0xff,0x7b,0x7b) })).width(iced::Length::FillPortion(1)).padding(10))
                )
                .push(Container::new(ChartWidget::new(MemChart { history: self.history.mem_load.clone() })).width(iced::Length::FillPortion(1)).padding(10))
                .push(Text::new(format!("Memory Load: {}%", t.mem.memory_load)).style(iced::theme::Text::Color(Color::from_rgb(0x7b as f32/255.0,0xff as f32/255.0,0xff as f32/255.0))))
                .push(Text::new(format!("Physical: {} MB free / {} MB total", t.mem.free_phys / 1024 / 1024, t.mem.total_phys / 1024 / 1024)).style(iced::theme::Text::Color(Color::from_rgb(0.7,0.8,1.0))))
                .push(Text::new(format!("Virtual: {} MB free / {} MB total", t.mem.free_virtual / 1024 / 1024, t.mem.total_virtual / 1024 / 1024)).style(iced::theme::Text::Color(Color::from_rgb(0.7,0.8,1.0))));
            for disk in &t.disks {
                col = col.push(Text::new(format!(
                    "Disk {}: Used {} GB / Total {} GB",
                    disk.mount,
                    disk.used_space / 1024 / 1024 / 1024,
                    disk.total_space / 1024 / 1024 / 1024
                )).style(iced::theme::Text::Color(Color::from_rgb(0.7,0.8,1.0))));
            }
        } else {
            col = col.push(Text::new("Loading...").style(iced::theme::Text::Color(Color::from_rgb(0.7,0.8,1.0))));
        }
        Container::new(col).center_x().center_y().style(iced::theme::Container::BoxShadow { color: bg, offset: (0.0, 0.0), blur_radius: 0.0, spread_radius: 0.0 }).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_secs(1)).map(Message::Tick)
    }
}

async fn fetch_telemetry() -> Option<TelemetryJsonFull> {
    let resp = reqwest::get("http://localhost:8000/telemetry").await.ok()?;
    resp.json().await.ok()
}

struct CpuChart {
    history: VecDeque<f64>,
    label: &'static str,
    color: RGBColor,
}

impl Chart<Message> for CpuChart {
    fn build_chart<DB: DrawingBackend>(&self, builder: ChartBuilder<DB>) {
        let max = self.history.iter().cloned().fold(0./0., f64::max);
        let min = self.history.iter().cloned().fold(0./0., f64::min);
        let mut chart = builder
            .caption(self.label, ("sans-serif", 16).into_font().color(&self.color))
            .margin(10)
            .x_label_area_size(10)
            .y_label_area_size(30)
            .build_cartesian_2d(0..HISTORY_LEN, min..max)
            .unwrap();
        chart.configure_mesh().disable_mesh().draw().unwrap();
        chart.draw_series(LineSeries::new(
            self.history.iter().enumerate().map(|(i, v)| (i, *v)),
            &self.color,
        )).unwrap();
    }
}

struct MemChart {
    history: VecDeque<u32>,
}

impl Chart<Message> for MemChart {
    fn build_chart<DB: DrawingBackend>(&self, builder: ChartBuilder<DB>) {
        let max = *self.history.iter().max().unwrap_or(&100);
        let min = *self.history.iter().min().unwrap_or(&0);
        let mut chart = builder
            .caption("Memory Load %", ("sans-serif", 16).into_font().color(&RGBColor(0xff,0x7b,0xff)))
            .margin(10)
            .x_label_area_size(10)
            .y_label_area_size(30)
            .build_cartesian_2d(0..HISTORY_LEN, min..max)
            .unwrap();
        chart.configure_mesh().disable_mesh().draw().unwrap();
        chart.draw_series(LineSeries::new(
            self.history.iter().enumerate().map(|(i, v)| (i, *v)),
            &RGBColor(0xff,0x7b,0xff),
        )).unwrap();
    }
}

#[tokio::main]
async fn main() -> iced::Result {
    Dashboard::run(Settings::default())
}