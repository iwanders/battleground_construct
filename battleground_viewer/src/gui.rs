use battleground_construct::components;
use components::team::TeamId;
use three_d::egui;
use three_d::egui::*;

pub fn shadow_smaller_dark() -> epaint::Shadow {
    epaint::Shadow {
        extrusion: 4.0,
        color: Color32::from_black_alpha(96),
    }
}

#[derive(Debug)]
pub struct State {
    match_window: std::cell::RefCell<bool>,
    teams: std::collections::HashMap<TeamId, components::team::Team>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            match_window: true.into(),
            teams: Default::default(),
        }
    }
}

impl State {
    pub fn update(&mut self, construct: &crate::Construct) {
        for (_e, team) in construct.world.component_iter::<components::team::Team>() {
            self.teams.insert(team.id(), team.clone());
        }
    }

    pub fn ui_team_color(color: &battleground_construct::display::Color) -> Color32 {
        let c = Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a);
        let mut h: crate::gui::ecolor::Hsva = c.into();
        // Modify a bit to get more 'gui' colors.
        h.s = (h.s - 0.2).clamp(0.0, 1.0);
        h.v = (h.v - 0.5).clamp(0.0, 1.0);
        h.into()
    }

    pub fn get_team_color(&self, team_id: TeamId) -> Color32 {
        // self.teams.get(&team_id).map(|t| t.color()).map(|x| Color32::from_rgba_unmultiplied(x.r / 2, x.g / 2, x.b / 2, x.a)).unwrap_or(Color32::GRAY)
        let c = self.teams.get(&team_id).map(|t| t.color());
        if let Some(c) = c {
            Self::ui_team_color(c)
        } else {
            Color32::GRAY
        }
    }
    pub fn get_team_name(&self, team_id: TeamId) -> String {
        self.teams
            .get(&team_id)
            .map(|t| t.name().to_owned())
            .unwrap_or(format!("{team_id:?}"))
    }
}

pub fn window_match(ctx: &egui::Context, construct: &crate::Construct, state: &mut State) {
    let mut open = state.match_window.borrow_mut();
    // let open = open.unwrap();
    egui::Window::new("Match")
        .frame(Frame {
            inner_margin: ctx.style().spacing.window_margin,
            rounding: ctx.style().visuals.window_rounding,
            shadow: shadow_smaller_dark(),
            fill: ctx.style().visuals.window_fill,
            stroke: ctx.style().visuals.window_stroke,
            ..Frame::none()
        })
        .open(&mut open)
        .show(ctx, |ui| {
            use components::capturable::Capturable;
            use components::match_finished::MatchFinished;
            use components::match_king_of_the_hill::MatchKingOfTheHill;
            use components::match_time_limit::MatchTimeLimit;
            let progress_width = 200.0;

            ui.heading("General");

            for (_e, limit) in construct.world.component_iter::<MatchTimeLimit>() {
                let current_time = limit.current_time();
                let time_limit = limit.time_limit();
                let ratio = current_time / limit.time_limit();
                ui.scope(|ui| {
                    ui.add(
                        ProgressBar::new(ratio)
                            .desired_width(progress_width)
                            .text(format!("Time: {current_time:.1}/{time_limit:.1}")),
                    );
                });
            }

            for (_e, capturable) in construct.world.component_iter::<Capturable>() {
                let color = capturable.owner().map(|t| state.get_team_color(t)).unwrap();
                let ratio = capturable.strength();
                ui.scope(|ui| {
                    ui.visuals_mut().selection.bg_fill = color;
                    ui.add(
                        ProgressBar::new(ratio)
                            .desired_width(progress_width)
                            .text(format!("Capturable: {ratio:.1}")),
                    );
                });
            }

            for (_e, match_finished) in construct.world.component_iter::<MatchFinished>() {
                if let Some(report) = match_finished.report() {
                    if let Some(winner) = report.winner {
                        let team_name = state.get_team_name(winner);
                        ui.label(format!(
                            "{team_name:} won by {:?} in {:.1}s",
                            report.conclusion, report.duration
                        ));
                    }
                }
            }

            use components::unit::UnitType;
            #[derive(Default, Debug)]
            struct TeamInfo {
                units: std::collections::HashMap<UnitType, (usize, usize)>,
            }
            let mut team_info = std::collections::HashMap::<TeamId, TeamInfo>::new();
            for (unit_entity, unit) in construct.world.component_iter::<components::unit::Unit>() {
                if let Some(unit_team) = construct
                    .world
                    .component::<components::team_member::TeamMember>(unit_entity)
                {
                    let unit_info = team_info
                        .entry(unit_team.team())
                        .or_default()
                        .units
                        .entry(unit.unit_type())
                        .or_default();
                    if let Some(health) = construct
                        .world
                        .component::<components::health::Health>(unit_entity)
                    {
                        if !health.is_destroyed() {
                            unit_info.0 += 1;
                        } else {
                            unit_info.1 += 1;
                        }
                    } else {
                        unit_info.1 += 1;
                    }
                }
            }

            let koth_report = construct
                .world
                .component_iter::<MatchKingOfTheHill>()
                .next()
                .map(|v| v.1.report());

            for (team_id, team) in state.teams.iter() {
                ui.heading(format!("Team - {}", team.name()));

                if let Some(comment) = team.comment() {
                    ui.label(format!("Comment: {comment}",));
                }

                if let Some(ref koth_report) = koth_report {
                    let limit = koth_report.point_limit();

                    let points = koth_report
                        .points()
                        .iter()
                        .filter(|x| x.0 == *team_id)
                        .map(|x| x.1)
                        .last()
                        .unwrap_or(0.0);

                    if let Some(ref max) = limit {
                        ui.scope(|ui| {
                            ui.visuals_mut().selection.bg_fill = State::ui_team_color(team.color()); // Temporary change
                            let ratio = points / max;
                            ui.add(
                                ProgressBar::new(ratio)
                                    .desired_width(progress_width)
                                    .text(format!("Points: {points:.1}/{max:.1}")),
                            );
                        });
                    } else {
                        // No limit, lets just make some text.
                        ui.label(format!("KotH: {points:.1}"));
                    }
                }

                // Show the units.
                if let Some(entry) = team_info.get(team_id) {
                    for (unit, count) in entry.units.iter() {
                        let alive = count.0;
                        let dead = count.1;
                        ui.label(format!("{unit:?}: {alive} ({dead:?})"));
                    }
                }
            }
        });
}

pub fn top_bar(
    ctx: &egui::Context,
    construct: &crate::Construct,
    viewer_state: &mut crate::ViewerState,
    limiter: &mut crate::Limiter,
) {
    egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
        menu::bar(ui, |ui| {
            ui.menu_button("Construct", |ui| {
                if ui.button("Quit").clicked() {
                    viewer_state.exiting = true;
                }
            });
            if ui.button("Match").clicked() {
                let new_state = (!*viewer_state.gui.match_window.borrow()).into();
                viewer_state.gui.match_window = new_state;
            };
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    ui.label(if viewer_state.selected.is_empty() {
                        "select with middle click".to_owned()
                    } else {
                        format!("{:?}", viewer_state.selected)
                    });
                },
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.menu_button(
                    format!(
                        "{:.2} x {:.2}",
                        construct.elapsed_as_f32(),
                        limiter.real_speed()
                    ),
                    |ui| {
                        let label = if viewer_state.paused {
                            "Resume"
                        } else {
                            "Pause"
                        };
                        if ui.button(label).clicked() {
                            viewer_state.paused = !viewer_state.paused;
                            limiter.set_paused(viewer_state.paused);
                            ui.close_menu();
                        }
                        if ui.button("x0.05").clicked() {
                            limiter.set_desired_speed(0.05);
                            ui.close_menu();
                        }
                        if ui.button("x0.1").clicked() {
                            limiter.set_desired_speed(0.1);
                            ui.close_menu();
                        }
                        if ui.button("x0.25").clicked() {
                            limiter.set_desired_speed(0.25);
                            ui.close_menu();
                        }
                        if ui.button("x1.0").clicked() {
                            limiter.set_desired_speed(1.0);
                            ui.close_menu();
                        }
                        if ui.button("x2.0").clicked() {
                            limiter.set_desired_speed(2.0);
                            ui.close_menu();
                        }
                        if ui.button("x5.0").clicked() {
                            limiter.set_desired_speed(5.0);
                            ui.close_menu();
                        }
                    },
                );

                if let Some(v) = construct.recording_max_time() {
                    // https://github.com/emilk/egui/issues/1850
                    ui.scope(|ui| {
                        ui.spacing_mut().slider_width = 200.0; // Temporary change
                        ui.add(
                            egui::Slider::new(&mut viewer_state.playback, 0.0..=v)
                                .text("Seek")
                                .clamp_to_range(true)
                                .show_value(true),
                        );
                    });
                };
            });
        });
    });
}
