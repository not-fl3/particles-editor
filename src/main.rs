use macroquad::prelude::*;

use megaui_macroquad::{
    draw_megaui, draw_window,
    megaui::{self, hash},
    mouse_captured, mouse_over_ui, set_megaui_texture, WindowParams,
};

use macroquad_particles::{
    BlendMode, Curve, EmissionShape, Emitter, EmitterConfig, Interpolation, ParticleShape,
    PostProcessing,
};

fn color_picker_texture(w: usize, h: usize) -> (Texture2D, Image) {
    let ratio = 1.0 / h as f32;

    let mut image = Image::gen_image_color(w as u16, h as u16, WHITE);
    let image_data = image.get_image_data_mut();

    for j in 0..h {
        for i in 0..w {
            let lightness = 1.0 - i as f32 * ratio;
            let hue = j as f32 * ratio;

            image_data[i + j * w] = macroquad::color::hsl_to_rgb(hue, 1.0, lightness).into();
        }
    }

    (load_texture_from_image(&image), image)
}

fn color_picker(ui: &mut megaui::Ui, id: megaui::Id, data: &mut Color) -> bool {
    let mut canvas = ui.canvas();
    let cursor = canvas.request_space(megaui::Vector2::new(200., 220.));
    let mouse = mouse_position();

    let x = mouse.0 as i32 - cursor.x as i32;
    let y = mouse.1 as i32 - (cursor.y as i32 + 20);

    if x > 0 && x < 200 && y > 0 && y < 200 {
        let ratio = 1.0 / 200.0 as f32;
        let lightness = 1.0 - x as f32 * ratio;
        let hue = y as f32 * ratio;

        if is_mouse_button_down(MouseButton::Left) && mouse_captured() == false {
            *data = macroquad::color::hsl_to_rgb(hue, 1.0, lightness).into();
        }
    }

    canvas.rect(
        megaui::Rect::new(cursor.x - 5.0, cursor.y - 5.0, 210.0, 395.0),
        megaui::Color::new(0.7, 0.7, 0.7, 1.0),
        megaui::Color::new(0.9, 0.9, 0.9, 1.0),
    );

    canvas.rect(
        megaui::Rect::new(cursor.x, cursor.y, 200.0, 18.0),
        megaui::Color::new(0.0, 0.0, 0.0, 1.0),
        megaui::Color::new(data.r, data.g, data.b, 1.0),
    );
    canvas.image(
        megaui::Rect::new(cursor.x, cursor.y + 20.0, 200.0, 200.0),
        0,
    );

    let (h, _, l) = macroquad::color::rgb_to_hsl(*data);

    canvas.rect(
        megaui::Rect::new(
            cursor.x + (1.0 - l) * 200.0 - 3.5,
            cursor.y + h * 200. + 20.0 - 3.5,
            7.0,
            7.0,
        ),
        megaui::Color::new(0.3, 0.3, 0.3, 1.0),
        megaui::Color::new(1.0, 1.0, 1.0, 1.0),
    );

    ui.slider(hash!(id, "alpha"), "Alpha", 0.0..1.0, &mut data.a);
    ui.separator();
    ui.slider(hash!(id, "red"), "Red", 0.0..1.0, &mut data.r);
    ui.slider(hash!(id, "green"), "Green", 0.0..1.0, &mut data.g);
    ui.slider(hash!(id, "blue"), "Blue", 0.0..1.0, &mut data.b);
    ui.separator();
    let (mut h, mut s, mut l) = macroquad::color::rgb_to_hsl(*data);
    ui.slider(hash!(id, "hue"), "Hue", 0.0..1.0, &mut h);
    ui.slider(hash!(id, "saturation"), "Saturation", 0.0..1.0, &mut s);
    ui.slider(hash!(id, "lightess"), "Lightness", 0.0..1.0, &mut l);
    let Color { r, g, b, .. } = macroquad::color::hsl_to_rgb(h, s, l);
    data.r = r;
    data.g = g;
    data.b = b;

    ui.separator();
    if ui.button(None, "    ok    ")
        || is_key_down(KeyCode::Escape)
        || is_key_down(KeyCode::Enter)
        || (is_mouse_button_pressed(MouseButton::Left)
            && Rect::new(cursor.x - 10., cursor.y - 10.0, 230., 420.)
                .contains(vec2(mouse.0, mouse.1))
                == false)
    {
        return true;
    }

    false
}

fn colorbox(ui: &mut megaui::Ui, id: megaui::Id, label: &str, data: &mut Color) {
    ui.label(None, label);
    let mut canvas = ui.canvas();
    let cursor = canvas.cursor();

    canvas.rect(
        megaui::Rect::new(cursor.x + 20.0, cursor.y, 50.0, 18.0),
        megaui::Color::new(0.2, 0.2, 0.2, 1.0),
        megaui::Color::new(data.r, data.g, data.b, 1.0),
    );
    if ui.last_item_clicked() {
        *ui.get_bool(hash!(id, "color picker opened")) ^= true;
    }
    if *ui.get_bool(hash!(id, "color picker opened")) {
        ui.popup(
            hash!(id, "color popup"),
            megaui::Vector2::new(200., 400.),
            |ui| {
                if color_picker(ui, id, data) {
                    *ui.get_bool(hash!(id, "color picker opened")) = false;
                }
            },
        );
    }
}

fn curvebox(ui: &mut megaui::Ui, curve: &mut Curve) {
    let mut canvas = ui.canvas();
    let w = 200.0;
    let h = 50.0;
    let min = 0.0;
    let max = 2.0;
    let (mouse_x, mouse_y) = mouse_position();
    let pos = canvas.request_space(megaui::Vector2::new(w, h));

    canvas.rect(
        megaui::Rect::new(pos.x, pos.y, w, h),
        megaui::Color::new(0.5, 0.5, 0.5, 1.0),
        None,
    );

    let t = ((mouse_x - pos.x) / w).max(0.0).min(1.0);

    for (_, line) in curve.points.windows(2).enumerate() {
        let (x0, value0) = line[0];
        let (x1, value1) = line[1];
        let y0 = (1.0 - value0 / (max - min)) * h;
        let y1 = (1.0 - value1 / (max - min)) * h;

        canvas.line(
            megaui::Vector2::new(pos.x + x0 * w, pos.y + y0),
            megaui::Vector2::new(pos.x + x1 * w, pos.y + y1),
            megaui::Color::new(0.5, 0.5, 0.5, 1.0),
        );
    }
    for (x, value) in &curve.points {
        let y = (1.0 - value / (max - min)) * h;

        let color = if (x - t).abs() < 0.1 {
            megaui::Color::new(0.9, 0.5, 0.5, 1.0)
        } else {
            megaui::Color::new(0.5, 0.5, 0.5, 1.0)
        };
        canvas.rect(
            megaui::Rect::new(pos.x + x * w - 2., pos.y + y - 2., 4., 4.),
            color,
            color,
        );
    }

    if is_mouse_button_down(MouseButton::Left) {
        let rect = Rect::new(pos.x, pos.y, w, h);

        let new_value = ((1.0 - (mouse_y - pos.y) / h) * (max - min))
            .min(max)
            .max(min);
        let dragging_point = ui.get_any::<Option<usize>>(hash!("dragging point"));

        if let Some(ix) = dragging_point {
            let (x, value) = curve.points.get_mut(*ix).unwrap();
            *x = t;
            *value = new_value;
        } else {
            if rect.contains(vec2(mouse_x, mouse_y)) {
                let closest_point = curve
                    .points
                    .iter_mut()
                    .position(|(x, _)| (*x - t).abs() < 0.1);

                if let Some(ix) = closest_point {
                    let (_, value) = curve.points.get_mut(ix).unwrap();
                    *value = new_value;
                    *ui.get_any::<Option<usize>>(hash!("dragging point")) = Some(ix);
                } else {
                    curve.points.push((t, new_value));
                    curve
                        .points
                        .sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
                }
            }
        }
    } else {
        *ui.get_any::<Option<usize>>(hash!("dragging point")) = None;
    }
}

#[macroquad::main("Particles editor")]
async fn main() {
    let (color_picker_texture, _) = color_picker_texture(200, 200);
    set_megaui_texture(0, color_picker_texture);

    let mut background_color = BLACK;
    let mut emitter = Emitter::new(EmitterConfig {
        lifetime: 0.5,
        amount: 2,
        initial_velocity: 50.0,
        size: 2.0,
        size_curve: None,
        blend_mode: BlendMode::Alpha,
        ..Default::default()
    });
    let mut emitter_position = vec2(50.0, 50.0);
    let mut emitter_speed = None;
    let mut lissajous_a = 1.0;
    let mut lissajous_b = 1.0;
    let mut mouse_pos_control = false;
    let mut circle_subdivisions = 20u32;
    let mut emission_rect_width = 0.0;
    let mut emission_rect_height = 0.0;
    let mut emission_sphere_radius = 0.0;
    let size_curve = Curve {
        points: vec![(0.0, 1.0), (1.0, 1.0)],
        interpolation: Interpolation::Linear,
        resolution: 30,
    };
    let mut config_serialized = String::new();
    let mut mouse_drag_available = true;
    let mut camera_width: f32 = 100.0;
    let mut camera_height: f32 = 100.0;

    loop {
        clear_background(background_color);

        set_default_camera();

        draw_window(
            hash!(),
            vec2(20., 20.),
            vec2(420., 500.),
            WindowParams {
                label: "Particles".to_string(),
                close_button: false,
                ..Default::default()
            },
            |ui| {
                ui.checkbox(hash!(), "Emitting", &mut emitter.config.emitting);
                ui.drag(hash!(), "Amount", (0, 1000), &mut emitter.config.amount);

                ui.tree_node(hash!(), "Time", |ui| {
                    ui.drag(
                        hash!(),
                        "Lifetime",
                        (0.0, 100.0),
                        &mut emitter.config.lifetime,
                    );
                    ui.drag(
                        hash!(),
                        "Lifetime randomness",
                        (0., 1.),
                        &mut emitter.config.lifetime_randomness,
                    );

                    ui.checkbox(hash!(), "One shot", &mut emitter.config.one_shot);
                    ui.drag(
                        hash!(),
                        "Explosiveness",
                        (0., 1.),
                        &mut emitter.config.explosiveness,
                    );
                });

                ui.tree_node(hash!(), "Drawing", |ui| {
                    let mut n = match emitter.config.shape {
                        ParticleShape::Rectangle => 0,
                        ParticleShape::Circle { .. } => 1,
                        _ => {
                            return;
                        }
                    };
                    let old_n = n;
                    ui.combo_box(hash!(), "Shape ", &["rectangle", "circle"], &mut n);
                    match n {
                        0 => {
                            emitter.config.shape = ParticleShape::Rectangle;
                        }
                        1 => {
                            emitter.config.shape = ParticleShape::Circle {
                                subdivisions: circle_subdivisions,
                            };
                            let old_subdivisions = circle_subdivisions;
                            ui.drag(
                                hash!(),
                                "Circle subdivisions",
                                (0, 60),
                                &mut circle_subdivisions,
                            );
                            if old_subdivisions != circle_subdivisions {
                                emitter.update_particle_mesh();
                            }
                        }
                        _ => unreachable!(),
                    }

                    if old_n != n {
                        emitter.update_particle_mesh();
                    }
                    ui.checkbox(hash!(), "Local coords", &mut emitter.config.local_coords);
                    let mut n = match emitter.config.blend_mode {
                        BlendMode::Alpha => 0,
                        BlendMode::Additive => 1,
                    };
                    ui.combo_box(hash!(), "Blend mode", &["alpha", "additive"], &mut n);
                    match n {
                        0 => {
                            emitter.config.blend_mode = BlendMode::Alpha;
                        }
                        1 => {
                            emitter.config.blend_mode = BlendMode::Additive;
                        }
                        _ => unreachable!(),
                    }

                    let mut postprocess = emitter.config.post_processing.is_some();
                    ui.checkbox(hash!(), "Downscale", &mut postprocess);
                    if postprocess {
                        emitter.config.post_processing = Some(PostProcessing);
                    } else {
                        emitter.config.post_processing = None;
                    }
                });

                ui.tree_node(hash!(), "Emission shape", |ui| {
                    let mut n = match emitter.config.emission_shape {
                        EmissionShape::Point => 0,
                        EmissionShape::Rect { .. } => 1,
                        EmissionShape::Sphere { .. } => 2,
                    };
                    ui.combo_box(hash!(), "Shape", &["Point", "Rectangle", "Circle"], &mut n);
                    match n {
                        0 => emitter.config.emission_shape = EmissionShape::Point,
                        1 => {
                            emitter.config.emission_shape = EmissionShape::Rect {
                                width: emission_rect_width,
                                height: emission_rect_height,
                            };
                            ui.drag(hash!(), "Rectangle width", None, &mut emission_rect_width);
                            ui.drag(hash!(), "Rectangle height", None, &mut emission_rect_height);
                        }
                        2 => {
                            emitter.config.emission_shape = EmissionShape::Sphere {
                                radius: emission_sphere_radius,
                            };
                            ui.drag(
                                hash!(),
                                "Circle radius",
                                (0., 1000.0),
                                &mut emission_sphere_radius,
                            );
                        }
                        _ => unreachable!(),
                    }
                });
                ui.tree_node(hash!(), "Velocity", |ui| {
                    ui.drag(
                        hash!(),
                        "Initial velocity",
                        (0., 1000.),
                        &mut emitter.config.initial_velocity,
                    );
                    ui.drag(
                        hash!(),
                        "Initial velocity randomness",
                        (0., 1.),
                        &mut emitter.config.initial_velocity_randomness,
                    );

                    ui.drag(
                        hash!(),
                        "Linear acceleration",
                        (-100., 100.),
                        &mut emitter.config.linear_accel,
                    );
                    ui.drag(
                        hash!(),
                        "Gravity x",
                        (-100., 100.),
                        &mut emitter.config.gravity.x,
                    );
                    ui.drag(
                        hash!(),
                        "Gravity y",
                        (-100., 100.),
                        &mut emitter.config.gravity.y,
                    );
                });
                ui.tree_node(hash!(), "Direction", |ui| {
                    ui.drag(hash!(), "x", None, &mut emitter.config.initial_direction.x);
                    ui.drag(hash!(), "y", None, &mut emitter.config.initial_direction.y);
                    ui.drag(
                        hash!(),
                        "spread",
                        (0.0, 2. * std::f32::consts::PI),
                        &mut emitter.config.initial_direction_spread,
                    );
                });
                ui.tree_node(hash!(), "Scale", |ui| {
                    ui.drag(hash!(), "Size", (0.0, 100.), &mut emitter.config.size);
                    ui.drag(
                        hash!(),
                        "Size random",
                        (0.0, 1.0),
                        &mut emitter.config.size_randomness,
                    );
                    let mut size_curve_enabled = emitter.config.size_curve.is_some();
                    ui.checkbox(hash!(), "Size curve", &mut size_curve_enabled);
                    if size_curve_enabled {
                        let size_curve =
                            emitter.config.size_curve.get_or_insert(size_curve.clone());
                        curvebox(ui, size_curve);
                        emitter.rebuild_size_curve();
                    } else {
                        emitter.config.size_curve = None;
                        emitter.rebuild_size_curve();
                    }
                });
                ui.tree_node(hash!(), "Colors", |ui| {
                    let curve = &mut emitter.config.colors_curve;
                    colorbox(ui, hash!(), "Start color", &mut curve.start);
                    colorbox(ui, hash!(), "Mid color", &mut curve.mid);
                    colorbox(ui, hash!(), "End color", &mut curve.end);
                });
                ui.tree_node(hash!(), "Scene", |ui| {
                    ui.drag(hash!(), "screen width", None, &mut camera_width);
                    ui.drag(hash!(), "screen height", None, &mut camera_height);

                    colorbox(ui, hash!(), "Background color", &mut background_color);
                    let mut n = *ui.get_any::<usize>(hash!("emitter position selection"));
                    ui.combo_box(
                        hash!(),
                        "Emitter position",
                        &["fixed", "flying", "mouse"],
                        &mut n,
                    );
                    *ui.get_any::<usize>(hash!("emitter position selection")) = n;
                    match n {
                        0 => {
                            mouse_pos_control = false;
                            emitter_speed = None;
                        }
                        1 => {
                            mouse_pos_control = false;
                            ui.drag(
                                hash!(),
                                "Flying speed",
                                None,
                                emitter_speed.get_or_insert(1.0),
                            );
                            ui.drag(hash!(), "Lissajous A", None, &mut lissajous_a);
                            ui.drag(hash!(), "Lissajous B", None, &mut lissajous_b);
                        }
                        2 => {
                            mouse_pos_control = true;
                            emitter_speed = None;
                            ui.label(None, "Right click to move the emitter");
                        }
                        _ => unreachable!(),
                    }
                });
                ui.tree_node(hash!(), "Export/import", |ui| {
                    if ui.button(None, "export") {
                        config_serialized = nanoserde::SerJson::serialize_json(&emitter.config);
                    }
                    if ui.button(None, "import") {
                        if let Ok(config) = nanoserde::DeJson::deserialize_json(&config_serialized)
                        {
                            emitter.config = config;
                            emitter.rebuild_size_curve();
                            emitter.update_particle_mesh();
                        }
                    }
                    ui.editbox(
                        hash!(),
                        megaui::Vector2::new(400.0, 50.0),
                        &mut config_serialized,
                    );
                });
            },
        );

        draw_megaui();

        set_camera(Camera2D::from_display_rect(Rect::new(
            0.0,
            0.0,
            camera_width,
            camera_height,
        )));

        if is_mouse_button_down(MouseButton::Left) && (mouse_over_ui() || mouse_captured()) {
            mouse_drag_available = false;
        }

        if is_mouse_button_down(MouseButton::Left) == false {
            mouse_drag_available = true;
        }

        if mouse_pos_control {
            let (x, y) = mouse_position();
            if mouse_drag_available && is_mouse_button_down(MouseButton::Left) {
                emitter_position = vec2(
                    x / screen_width() * camera_width,
                    y / screen_height() * camera_height,
                )
            }
        }
        emitter.draw(emitter_position);

        if let Some(flying) = emitter_speed {
            emitter_position = vec2(
                (get_time() as f32 * flying * lissajous_a).sin() * 20.0 + 50.0,
                (get_time() as f32 * flying * lissajous_b).cos() * 20.0 + 50.0,
            );
        }

        next_frame().await;
    }
}
