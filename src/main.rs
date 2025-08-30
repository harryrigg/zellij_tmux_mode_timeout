use std::{collections::BTreeMap, time::Duration};

use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    // Config
    timeout: Duration,

    // State
    input_count_during_timeout: Option<usize>,
    timer_queue: usize,
}

impl ZellijPlugin for State {
    fn load(&mut self, config: BTreeMap<String, String>) {
        let timeout_ms = config
            .get("timeout_ms")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(3000);
        self.timeout = Duration::from_millis(timeout_ms);

        request_permission(&[
            PermissionType::ChangeApplicationState,
            PermissionType::ReadApplicationState,
        ]);

        subscribe(&[
            EventType::Timer,
            EventType::InputReceived,
            EventType::ModeUpdate,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::InputReceived => {
                if let Some(current_count) = self.input_count_during_timeout.as_mut() {
                    *current_count += 1;
                    if *current_count > 1 {
                        self.input_count_during_timeout = None;
                    }
                }
            }
            Event::ModeUpdate(mode_info)
                if mode_info.mode != InputMode::Tmux
                    && self.input_count_during_timeout.is_some() =>
            {
                self.input_count_during_timeout = None;
            }
            Event::Timer(_) => {
                self.timer_queue -= 1;

                if self.timer_queue == 0 {
                    if self.input_count_during_timeout.is_some() {
                        switch_to_input_mode(&InputMode::Normal);
                        self.input_count_during_timeout = None;
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        match pipe_message.name.as_str() {
            "switch_to_tmux_mode" if self.input_count_during_timeout.is_none() => {
                switch_to_input_mode(&InputMode::Tmux);

                self.input_count_during_timeout = Some(0);
                self.timer_queue += 1;
                set_timeout(self.timeout.as_secs_f64());
            }
            _ => {}
        }
        false
    }
}

register_plugin!(State);
