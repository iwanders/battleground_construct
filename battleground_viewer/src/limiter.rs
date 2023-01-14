use crate::time_provider;

pub struct Limiter {
    desired_speed: f32,
    real_speed: f32,
    is_paused: bool,
    last_update_time: time_provider::Instant,
    last_construct_time: f32,
    update_deadline: time_provider::Duration,
}

impl Default for Limiter {
    fn default() -> Self {
        Self::new()
    }
}

impl Limiter {
    pub fn new() -> Self {
        Limiter {
            desired_speed: 1.0,
            real_speed: 1.0,
            last_construct_time: 0.0,
            is_paused: false,
            last_update_time: time_provider::Instant::now(),
            update_deadline: time_provider::Duration::from_secs_f64(1.0 / 60.0),
        }
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.is_paused = paused;
    }

    // pub fn is_paused(&self) -> bool {
    // self.is_paused
    // }

    pub fn set_desired_speed(&mut self, speed: f32) {
        self.desired_speed = speed;
    }

    /// Real speed is always a bit off from the desired speed, that is expected as the construct
    /// uses constant steps.
    pub fn real_speed(&self) -> f32 {
        self.real_speed
    }

    pub fn update<F: FnMut() -> Option<f32>>(&mut self, mut v: F) {
        if self.is_paused {
            self.last_update_time = time_provider::Instant::now();
            self.real_speed = 0.0;
            return;
        }

        let start_of_update = time_provider::Instant::now();

        let time_since_last = (start_of_update - self.last_update_time).as_secs_f32();
        let desired_construct_change = self.desired_speed * time_since_last;
        let desired_construct_finish_time = self.last_construct_time + desired_construct_change;
        let start_construct_time = self.last_construct_time;

        if desired_construct_finish_time > start_construct_time {
            loop {
                if start_of_update.elapsed() >= self.update_deadline {
                    // We didn't meet the update deadline, well... bummer.
                    // println!("Didn't meet rate");
                    break;
                }
                if let Some(new_time) = v() {
                    self.last_construct_time = new_time;
                    if self.last_construct_time >= desired_construct_finish_time {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        // Calculate the real speed we achieved.
        self.real_speed = (self.last_construct_time - start_construct_time) / time_since_last;
        self.last_update_time = time_provider::Instant::now();
    }
}
