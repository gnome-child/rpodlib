pub mod fmt;

pub trait Progress {
    fn on_stage(&mut self, stage: &'static str);
    fn on_ratio(&mut self, ratio: f32);
}
