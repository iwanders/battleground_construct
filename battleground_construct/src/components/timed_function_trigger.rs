use engine::prelude::*;

pub type Callback = std::rc::Rc<Box<dyn Fn(EntityId, &mut World)>>;

#[derive()]
pub struct TimedFunctionTrigger {
    /// Callback for this trigger.
    fun: Callback,
    /// Next trigger time.
    trigger_time: Option<f32>,
    /// Periodically called.
    interval: Option<f32>,
    /// After duration is used to calculate trigger_time on first cycle.
    after_duration: Option<f32>,
}

impl TimedFunctionTrigger {
    pub fn new<F>(trigger_time: f32, interval: Option<f32>, fun: F) -> Self
    where
        F: Fn(EntityId, &mut World) + 'static,
    {
        TimedFunctionTrigger {
            fun: std::rc::Rc::new(Box::new(fun)),
            trigger_time: Some(trigger_time),
            interval,
            after_duration: None,
        }
    }

    pub fn after<F>(after_duration: f32, fun: F) -> Self
    where
        F: Fn(EntityId, &mut World) + 'static,
    {
        TimedFunctionTrigger {
            fun: std::rc::Rc::new(Box::new(fun)),
            after_duration: Some(after_duration),
            interval: None,
            trigger_time: None,
        }
    }

    pub fn callback(&self) -> Callback {
        self.fun.clone()
    }

    pub fn should_call(&mut self, time: f32) -> Option<Callback> {
        if let Some(after_duration) = self.after_duration.take() {
            self.trigger_time = Some(time + after_duration);
        }
        if let Some(trigger_time) = self.trigger_time {
            if trigger_time <= time {
                self.trigger_time = None;
                if let Some(interval) = self.interval {
                    self.trigger_time = Some(interval + time);
                }
                return Some(self.callback());
            }
        }
        None
    }

    pub fn is_done(&self) -> bool {
        self.trigger_time.is_none() && self.interval.is_none()
    }
}
impl Component for TimedFunctionTrigger {}
