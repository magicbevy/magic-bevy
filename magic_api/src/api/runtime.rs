use std::any::Any;
use std::time::{Duration, Instant};

pub struct UpdateRegistry {
    callbacks: Vec<Box<dyn FnMut(f32)>>,
    target_fps: f32,
    time_scale: f32,
}

impl UpdateRegistry {
    pub fn new(target_fps: f32, time_scale: f32) -> Self {
        Self {
            callbacks: Vec::new(),
            target_fps,
            time_scale,
        }
    }

    pub fn add_update<F: FnMut(f32) + 'static>(&mut self, callback: F) {
        self.callbacks.push(Box::new(callback));
    }

    pub fn set_target_fps(&mut self, fps: f32) {
        if fps > 0.0 { self.target_fps = fps; }
    }

    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale;
    }

    pub fn run(mut self) {
        let mut last_time = Instant::now();

        loop {
            let current_fps = if self.target_fps > 0.0 { self.target_fps } else { 60.0 };
            let frame_duration = Duration::from_secs_f32(1.0 / current_fps);

            let now = Instant::now();
            let raw_dt = now.duration_since(last_time).as_secs_f32();
            last_time = now;

            let dt = raw_dt * self.time_scale;

            for callback in &mut self.callbacks {
                callback(dt);
            }

            let elapsed = now.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
    }
}

pub struct RuntimeRegistry<'a> {
    pub render_context: Option<Box<dyn Any + Send>>,
    pub log_fn: Option<fn(crate::log::LogLevel, &str, &str)>,
    pub updates: &'a mut UpdateRegistry,
}