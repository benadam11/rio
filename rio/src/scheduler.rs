use std::collections::VecDeque;
use std::time::{Duration, Instant};
use winit::event_loop::EventLoopProxy;

use crate::event::EventP;

/// ID uniquely identifying a timer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TimerId {
    topic: Topic,
    tab_id: u8,
}

impl TimerId {
    pub fn new(topic: Topic, tab_id: u8) -> Self {
        Self { topic, tab_id }
    }
}

/// Available timer topics.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Topic {
    #[allow(dead_code)]
    SelectionScrolling,
    Frame,
}

/// Event scheduled to be emitted at a specific time.
pub struct Timer {
    pub deadline: Instant,
    pub event: EventP,
    pub id: TimerId,

    interval: Option<Duration>,
}

/// Scheduler tracking all pending timers.
pub struct Scheduler {
    timers: VecDeque<Timer>,
    event_proxy: EventLoopProxy<EventP>,
}

impl Scheduler {
    pub fn new(event_proxy: EventLoopProxy<EventP>) -> Self {
        Self {
            timers: VecDeque::new(),
            event_proxy,
        }
    }

    /// Process all pending timers.
    ///
    /// If there are still timers pending after all ready events have been processed, the closest
    /// pending deadline will be returned.
    pub fn update(&mut self) -> Option<Instant> {
        let now = Instant::now();

        while !self.timers.is_empty() && self.timers[0].deadline <= now {
            if let Some(timer) = self.timers.pop_front() {
                // Automatically repeat the event.
                if let Some(interval) = timer.interval {
                    self.schedule(timer.event.clone(), interval, true, timer.id);
                }

                let _ = self.event_proxy.send_event(timer.event);
            }
        }

        self.timers.get(0).map(|timer| timer.deadline)
    }

    /// Schedule a new event.
    pub fn schedule(
        &mut self,
        event: EventP,
        interval: Duration,
        repeat: bool,
        timer_id: TimerId,
    ) {
        let deadline = Instant::now() + interval;

        // Get insert position in the schedule.
        let index = self
            .timers
            .iter()
            .position(|timer| timer.deadline > deadline)
            .unwrap_or(self.timers.len());

        // Set the automatic event repeat rate.
        let interval = if repeat { Some(interval) } else { None };

        self.timers.insert(
            index,
            Timer {
                interval,
                deadline,
                event,
                id: timer_id,
            },
        );
    }

    /// Cancel a scheduled event.
    #[allow(dead_code)]
    pub fn unschedule(&mut self, id: TimerId) -> Option<Timer> {
        let index = self.timers.iter().position(|timer| timer.id == id)?;
        self.timers.remove(index)
    }

    /// Check if a timer is already scheduled.
    #[allow(dead_code)]
    pub fn scheduled(&mut self, id: TimerId) -> bool {
        self.timers.iter().any(|timer| timer.id == id)
    }

    /// Remove all timers scheduled for a tab.
    ///
    /// This must be called when a tab is removed to ensure that timers on intervals do not
    /// stick around forever and cause a memory leak.
    #[allow(dead_code)]
    pub fn unschedule_tab(&mut self, tab_id: u8) {
        self.timers.retain(|timer| timer.id.tab_id != tab_id);
    }
}
