use iced::{Application, Command, Element, executor, Settings, Subscription, Theme};
use iced::widget::{Column, Text, Row, Container, canvas, Canvas};
use serde::Deserialize;
use std::time::{Duration, Instant};

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

struct Dashboard {
    telemetry: Option<TelemetryJsonFull>,
}

impl Application for Dashboard {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Dashboard { telemetry: None }, Command::none())
    }

    fn title(&self) -> String {
        String::from("System Telemetry Dashboard")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Telemetry(data) => {
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
        let mut col = Column::new().push(Text::new("System Telemetry Dashboard").size(40));
        if let Some(t) = &self.telemetry {
            col = col
                .push(Text::new(format!("Uptime: {:.0} s", t.uptime)))
                .push(Text::new(format!("CPU User: {:.2}s, System: {:.2}s, Idle: {:.2}s", t.user_time, t.system_time, t.idle_time)))
                .push(Text::new(format!("Memory Load: {}%", t.mem.memory_load)))
                .push(Text::new(format!("Physical: {} MB free / {} MB total", t.mem.free_phys / 1024 / 1024, t.mem.total_phys / 1024 / 1024)))
                .push(Text::new(format!("Virtual: {} MB free / {} MB total", t.mem.free_virtual / 1024 / 1024, t.mem.total_virtual / 1024 / 1024)));
            for disk in &t.disks {
                col = col.push(Text::new(format!(
                    "Disk {}: Used {} GB / Total {} GB",
                    disk.mount,
                    disk.used_space / 1024 / 1024 / 1024,
                    disk.total_space / 1024 / 1024 / 1024
                )));
            }
        } else {
            col = col.push(Text::new("Loading..."));
        }
        Container::new(col).center_x().center_y().into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_secs(1)).map(Message::Tick)
    }
}

async fn fetch_telemetry() -> Option<TelemetryJsonFull> {
    let resp = reqwest::get("http://localhost:8000/telemetry").await.ok()?;
    resp.json().await.ok()
}

#[tokio::main]
async fn main() -> iced::Result {
    Dashboard::run(Settings::default())
}