
/// Sample flow direction for [`render_graph`] and [`RenderGraphInfo`]
pub enum GraphDirection {
    LeftToRight,
    RightToLeft,
}

/// Settings for [`render_graph`]
pub struct RenderGraphInfo {
    /// Value at the highest point of the graph
    pub max: f32,
    /// Height of the graph in logical pixels
    pub height: f32,
    /// Color to be used for columns <= max
    pub under_color: egui::Color32,
    /// Color to be used for (clamped) columns > max
    pub over_color: egui::Color32,
    /// Direction of data flow in the graph
    pub direction: GraphDirection,
}

/// Renders a graph in specified UI, at full width and height specified in info.
/// Values are taken from an iterator, with each value taking 1 logical pixel in
/// width.
pub fn render_graph(ui: &mut egui::Ui, info: RenderGraphInfo, values: impl Iterator<Item = f32>) {
    let (rect, _response) = ui.allocate_at_least(
        egui::vec2(ui.available_size().x, info.height),
        egui::Sense::hover(),
    );

    let over_stroke = egui::Stroke::new(1.0, info.over_color);
    let under_stroke = egui::Stroke::new(1.0, info.under_color);

    let mut shapes = vec![];

    let (mut current, delta) = match info.direction {
        GraphDirection::LeftToRight => (rect.left_bottom(), 1.0),
        GraphDirection::RightToLeft => (rect.right_bottom(), -1.0),
    };

    let samples = rect.width().floor() as usize;
    for sample in values.take(samples) {
        let stroke = if sample > info.max {
            over_stroke
        } else {
            under_stroke
        };

        let clamped = sample.clamp(0.0, info.max);
        let sample_height = (clamped / info.max) * info.height;
        shapes.push(egui::Shape::LineSegment {
            points: [
                egui::pos2(current.x, current.y),
                egui::pos2(current.x, current.y - sample_height),
            ],
            stroke,
        });

        current.x += delta;
    }

    ui.painter().extend(shapes);
}
