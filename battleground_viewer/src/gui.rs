use three_d::egui;
use three_d::egui::*;

#[derive(Default, Debug)]
pub struct State {
    match_window: bool,
}

pub fn shadow_smaller_dark() -> epaint::Shadow {
    epaint::Shadow {
        extrusion: 4.0,
        color: Color32::from_black_alpha(96),
    }
}

pub fn window_match(
    ctx: &egui::Context,
    construct: &crate::Construct,
    viewer_state: &mut crate::ViewerState,
) {
    egui::Window::new("Match")
        .frame(Frame {
            inner_margin: ctx.style().spacing.window_margin,
            rounding: ctx.style().visuals.window_rounding,
            shadow: shadow_smaller_dark(),
            fill: ctx.style().visuals.window_fill,
            stroke: ctx.style().visuals.window_stroke,
            ..Frame::none()
        }).open(&mut viewer_state.gui.match_window)
        .show(ctx, |ui| {
            use battleground_construct::components::match_king_of_the_hill::MatchKingOfTheHill;
            use battleground_construct::components::match_time_limit::MatchTimeLimit;
            let progress_width = 200.0;
            // ui.scope(|ui| {
            // ui.label("Hello World!");
            // ui.spacing_mut().slider_width = 200.0; // Temporary change
            for (_e, limit) in construct.world.component_iter::<MatchTimeLimit>() {
                let current_time = limit.current_time();
                let time_limit = limit.time_limit();
                let ratio = current_time / limit.time_limit();
                ui.scope(|ui| {
                    // ui.visuals_mut().selection.bg_fill= Color32::RED; // Temporary change
                    ui.add(
                        ProgressBar::new(ratio)
                            .desired_width(progress_width)
                            .text(format!("Time: {current_time:.1}/{time_limit:.1}")),
                    );
                });
            }

            for (_e, match_koth) in construct.world.component_iter::<MatchKingOfTheHill>() {
                let report = match_koth.report();
                let limit = report.point_limit();
                for (team, points) in report.points() {
                    if let Some(ref max) = limit {
                        ui.scope(|ui| {
                            // progress bar, still needs to retrieve a team color.
                            // ui.visuals_mut().selection.bg_fill= Color32::RED; // Temporary change
                            let ratio = points / max;
                            ui.add(
                                ProgressBar::new(ratio)
                                    .desired_width(progress_width)
                                    .text(format!("{team:?}: {points:.1}/{max:.1}")),
                            );
                        });
                    } else {
                        // text.
                        ui.label(format!("{team:?}: {points:.1}"));
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
                viewer_state.gui.match_window = !viewer_state.gui.match_window;
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
