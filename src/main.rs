use eframe::egui;
use egui_plot::{Legend, Line, PlotPoint, PlotPoints};
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use std::{collections::VecDeque, thread};

// Data stuff
type Measurement = PlotPoint;

#[derive(Default)]
struct SensorData {
    values: VecDeque<Measurement>,
    time_period: f64,
}

impl SensorData {
    fn new(time_period: f64) -> Self {
        Self {
            values: VecDeque::default(),
            time_period,
        }
    }

    fn append_values(&mut self, v: Measurement) {
        if let Some(last) = self.values.back() {
            if last.x >= v.x {
                self.values = VecDeque::default();
            }
        }

        let min_x = v.x - self.time_period;
        self.values.push_back(v);

        while let Some(value) = self.values.front() {
            if value.x < min_x {
                self.values.pop_front();
            } else {
                break;
            }
        }
    }

    fn get_values(&self) -> PlotPoints {
        // WARN: Do not understand this
        PlotPoints::Owned(Vec::from_iter(self.values.iter().copied()))
    }

    fn append_str(&mut self, s: &str) {
        let parts = s.split(' ').collect::<Vec<&str>>();
        if parts.len() != 2 {
            return;
        }

        let x = parts.first().unwrap();
        let y = parts.last().unwrap();

        let x = match x.parse::<f64>() {
            Ok(value) => value,
            Err(_) => return,
        };

        let y = match y.parse::<f64>() {
            Ok(value) => value,
            Err(_) => return,
        };

        self.append_values(Measurement::new(x, y));
    }
}

// Application stuff
#[derive(Default)]
struct Visualizer {
    sensor_data: Arc<Mutex<SensorData>>,
}

impl Visualizer {
    fn new(time_period: f64) -> Self {
        Self {
            sensor_data: Arc::new(Mutex::new(SensorData::new(time_period))),
        }
    }
}

impl eframe::App for Visualizer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            egui_plot::Plot::new("Plot")
                .legend(Legend::default())
                .show(ui, |plot_ui| {
                    plot_ui.line(Line::new(self.sensor_data.lock().unwrap().get_values()));
                });
            ctx.request_repaint();
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let app = Visualizer::new(400.0);

    let io_measurements = app.sensor_data.clone();
    thread::spawn(move || {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(s) => io_measurements.lock().unwrap().append_str(s.as_str()),
                Err(_) => return,
            };
        }
    });

    let options = eframe::NativeOptions::default();
    eframe::run_native("My Plotting App", options, Box::new(|_cc| Box::new(app)))
}
