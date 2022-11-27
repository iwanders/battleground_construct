use engine::prelude::*;

#[derive(Debug, Clone)]
pub struct Expiry {
    pub expiry_time: Option<f32>,
    pub lifetime: f32,
}

impl Expiry {
    /// On next update, calculate the expiry time from the provided lifetime.
    pub fn lifetime(lifetime: f32) -> Self {
        Expiry {
            lifetime,
            expiry_time: None,
        }
    }

    /// Directly set the expiry time.
    pub fn at(expiry_time: f32) -> Self {
        Expiry {
            lifetime: 0.0,
            expiry_time: Some(expiry_time),
        }
    }

    pub fn update_expiry(&mut self, time: f32) {
        if self.expiry_time.is_none() {
            self.expiry_time = Some(time + self.lifetime)
        }
    }
    /// Check if this is expired, false if expiry time isn't set yet.
    pub fn is_expired(&self, time: f32) -> bool {
        if let Some(exp_time) = self.expiry_time {
            exp_time < time
        } else {
            false
        }
    }
}
impl Component for Expiry {}
