use crate::components::pose::Pose;
use engine::prelude::*;

pub type Callback = std::rc::Rc<Box<dyn Fn(EntityId, &mut World)>>;

#[derive()]
pub struct TimedFunctionTrigger {
    fun: Callback,
    trigger_time: Option<f32>,
    interval: Option<f32>,
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
        }
    }

    pub fn callback(&self) -> Callback {
        self.fun.clone()
    }

    pub fn should_call(&mut self, time: f32) -> Option<Callback> {
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
