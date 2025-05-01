use chrono::{DateTime, Utc};
use iced::{
    widget::{canvas::{Cache, Frame, Geometry}, Column, Container, Scrollable, Text}, 
    Alignment, Application, Command, Element, Length, Size, Settings, Theme, Renderer
};
use iced_core::Length as IcedLength;
use plotters::{chart, prelude::*};
use plotters_iced::{Chart, ChartWidget};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
use sysinfo::{CpuRefreshKind, RefreshKind, System};

const PLOT_SECONDS: usize = 60; // 1 min
const TITLE_FONT_SIZE: u16 = 22;
const SAMPLE_EVERY: Duration = Duration::from_millis(1000);
const CHART_HEIGHT: f32 = 200.0;

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

struct TelemetryApp {
    system: System,
    last_update: Instant,
    cpu_data: VecDeque<(DateTime<Utc>, f32)>,
    cache: Cache,
}

impl Application for TelemetryApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                system: System::new_with_specifics(
                    RefreshKind::new().with_cpu(CpuRefreshKind::new().with_cpu_usage()),
                ),
                last_update: Instant::now(),
                cpu_data: VecDeque::with_capacity(PLOT_SECONDS),
                cache: Cache::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("System Telemetry")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick => {
                if self.last_update.elapsed() >= SAMPLE_EVERY {
                    self.system.refresh_cpu();
                    let now = Utc::now();
                    
                    // Average all CPU usages
                    let cpu_avg = self.system.cpus().iter()
                        .map(|cpu| cpu.cpu_usage())
                        .sum::<f32>() / self.system.cpus().len() as f32;
                    
                    self.cpu_data.push_back((now, cpu_avg));
                    
                    // Keep only PLOT_SECONDS worth of data
                    while self.cpu_data.len() > PLOT_SECONDS {
                        self.cpu_data.pop_front();
                    }
                    
                    self.cache.clear();
                    self.last_update = Instant::now();
                }
            }
        }
        
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let chart: Element<Message> = ChartWidget::<Message, Theme, Renderer, _>::new(self)
            .width(IcedLength::Fill)
            .height(IcedLength::Fixed(CHART_HEIGHT))
            .into();
            
        let content = Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .padding(20)
            .width(Length::Fill)
            .push(Text::new("CPU Usage").size(TITLE_FONT_SIZE))
            .push(chart);

        Container::new(
            Scrollable::new(content).height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(Duration::from_millis(100))
            .map(|_| Message::Tick)
    }
    
    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

impl Chart<Message> for TelemetryApp {
    type State = ();
    
    fn draw<F: Fn(&mut Frame)>(
        &self,
        bounds: Size,
        draw_fn: F,
    ) -> Geometry {
        self.cache.draw((), bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut chart_builder: ChartBuilder<DB>) {
        if self.cpu_data.is_empty() {
            return;
        }

        let newest_time = self.cpu_data.back().unwrap().0;
        let oldest_time = newest_time - chrono::Duration::seconds(PLOT_SECONDS as i64);
        
        let mut chart = chart_builder
            .margin(10)
            .caption("CPU %", ("sans-serif", 15).into_font())
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(oldest_time..newest_time, 0f32..100f32)
            .expect("Failed to build chart");

        chart
            .configure_mesh()
            .y_labels(5)
            .disable_x_mesh()
            .draw()
            .expect("Failed to draw chart mesh");

        // Draw the line series
        chart
            .draw_series(LineSeries::new(
                self.cpu_data.iter().map(|(time, usage)| (*time, *usage)),
                &RGBColor(0, 175, 255),
            ))
            .expect("Failed to draw");
    }
}

fn main() -> iced::Result {
    TelemetryApp::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}