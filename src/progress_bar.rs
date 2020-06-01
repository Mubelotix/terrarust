use wasm_bindgen::JsValue;
use wasm_game_lib::graphics::canvas::{Canvas, LineStyle};
use wasm_game_lib::graphics::color::Color;
use wasm_game_lib::graphics::drawable::Drawable;

pub struct ProgressBar {
    pub coords: (f64, f64),
    pub dimensions: (f64, f64),
    pub style: LineStyle,
    pub current: usize,
    max: usize,
    pub background_color: Color,
    pub border_radius: f64,
    pub bar_color: Color,
}

impl ProgressBar {
    pub fn new(max: usize, coords: (f64, f64), dimensions: (f64, f64)) -> ProgressBar {
        ProgressBar {
            dimensions: (dimensions.0 - 3.0, dimensions.1 - 3.0),
            coords: (coords.0 + 1.5, coords.1 + 1.5),
            current: 0,
            style: LineStyle::default(),
            max,
            background_color: Color::white(),
            border_radius: 0.0,
            bar_color: Color::new(0x0f, 0x92, 0x00),
        }
    }

    pub fn inc(&mut self) -> bool {
        if self.current < self.max {
            self.current += 1;

            self.max == self.current
        } else {
            true
        }
    }
}

impl Drawable for ProgressBar {
    fn draw_on_canvas(&self, mut canvas: &mut Canvas) {
        self.style.apply_on_canvas(&mut canvas);

        let context = canvas.get_2d_canvas_rendering_context();

        context.begin_path();
        context
            .arc(
                self.coords.0 + self.border_radius,
                self.coords.1 + self.border_radius,
                self.border_radius,
                1.0 * std::f64::consts::PI,
                1.5 * std::f64::consts::PI,
            )
            .unwrap();
        //context.line_to(self.coords.0 + 2.0*self.border_radius + self.dimensions.0, self.coords.1);

        context
            .arc(
                self.coords.0 - self.border_radius + self.dimensions.0,
                self.coords.1 + self.border_radius,
                self.border_radius,
                1.5 * std::f64::consts::PI,
                0.0 * std::f64::consts::PI,
            )
            .unwrap();
        //context.line_to(self.coords.0 + 3.0*self.border_radius + self.dimensions.0, self.coords.1 + self.dimensions.1 + self.border_radius);

        context
            .arc(
                self.coords.0 - self.border_radius + self.dimensions.0,
                self.coords.1 + self.dimensions.1 - self.border_radius,
                self.border_radius,
                0.0 * std::f64::consts::PI,
                0.5 * std::f64::consts::PI,
            )
            .unwrap();
        //context.line_to(self.coords.0 + self.border_radius, self.coords.1 + self.dimensions.1 + 2.0*self.border_radius);

        context
            .arc(
                self.coords.0 + self.border_radius,
                self.coords.1 + self.dimensions.1 - self.border_radius,
                self.border_radius,
                0.5 * std::f64::consts::PI,
                1.0 * std::f64::consts::PI,
            )
            .unwrap();

        context.close_path();

        context.set_fill_style(&JsValue::from_str(&self.background_color.to_string()));
        context.fill();
        context.save();
        context.clip();

        context.set_fill_style(&JsValue::from_str(&self.bar_color.to_string()));
        context.fill_rect(
            self.coords.0,
            self.coords.1,
            self.dimensions.0 / self.max as f64 * self.current as f64,
            self.dimensions.1,
        );

        context.restore();
        context.stroke();
    }
}
