use crate::utils::graph::*;
use egui::{
    emath::RectTransform, epaint::CubicBezierShape, Color32, Painter, Pos2, Rect, Sense, Stroke,
    Vec2,
};
use std::collections::BTreeMap;

const ARROW_TIP_LENGHT: f32 = 10.;
const ARROW_WIDTH: f32 = 3.;
const COLOR_EDGE: Color32 = Color32::BLUE;
const COLOR_NODES: Color32 = Color32::WHITE;
const COLOR_LABEL_EDGE: Color32 = Color32::GRAY;
const COLOR_LABEL_NODE: Color32 = Color32::BLACK;

// edge type
pub enum EdgeType {
    SelfLoop,
    Directed,
    Colliding,
}

struct DisplayGraph {
    graph: Graph,
    nodes_pos: BTreeMap<IndNode, Pos2>,
    edges_type: BTreeMap<IndEdge, EdgeType>,
    explorer_order: Vec<Vec<IndNode>>,
    last_parameter: DisplayGraphParameter,
}

#[derive(PartialEq, Copy, Clone)]
pub struct DisplayGraphParameter {
    pub padding_x: f32,
    pub padding_y: f32,
    pub node_size: f32,
    // we can add more parameters here as we need them or like the
}

impl DisplayGraphParameter {
    pub fn invalid() -> Self {
        DisplayGraphParameter {
            padding_x: -1.,
            padding_y: -1.,
            node_size: -1.,
        }
    }
}

impl From<Graph> for DisplayGraph {
    fn from(graph: Graph) -> Self {
        Self {
            graph,
            nodes_pos: graph
                .get_nodes_ids()
                .into_iter()
                .map(|node_id| (node_id, Pos2::new(0., 0.)))
                .collect(),
            edges_type: graph
                .get_edges_ids()
                .into_iter()
                .map(|edge_id| (edge_id, EdgeType::Directed))
                .collect(),
            explorer_order: graph.bfs(graph.start_node),
            last_parameter: DisplayGraphParameter::invalid(),
        }
    }
}

impl DisplayGraph {
    fn calculate_nodes_position(&mut self, bfs_max_width: f32) {
        let params = self.last_parameter;
        let width_painting_area =
            bfs_max_width as f32 * (params.node_size + params.padding_x) + params.padding_x;

        for (current_bfs_depth, nodes_level) in self.explorer_order.iter().enumerate() {
            for (index, node) in nodes_level.iter().enumerate() {
                self.nodes_pos[node] = Pos2 {
                    x: (index as f32 + 1.)
                        * (width_painting_area / (nodes_level.len() as f32 + 1.)),
                    y: (current_bfs_depth as f32) * (params.node_size + params.padding_y),
                };
            }
        }
    }

    pub fn position(&mut self, params: DisplayGraphParameter) -> Vec2 {
        let bfs_max_width = self
            .explorer_order
            .iter()
            .map(|nodes| nodes.len())
            .max()
            .unwrap_or(0) as f32;

        let bfs_depth = self.explorer_order.len();
        // check if the graph has to be re-positioned (only if the params change)
        if params != self.last_parameter {
            self.last_parameter = params;
            self.calculate_nodes_position(bfs_max_width);
        }
        let width_painting_area =
            bfs_max_width as f32 * (params.node_size + params.padding_x) + params.padding_x;
        let height_painting_area = bfs_depth as f32 * (params.node_size + params.padding_y);

        Vec2 {
            x: width_painting_area,
            y: height_painting_area,
        }
    }

    pub fn drag_nodes(
        &mut self,
        to_screen: RectTransform,
        ui: &egui::Ui,
        response: &mut egui::Response,
    ) {
        for (index, current_pos) in self.nodes_pos.iter_mut() {
            let screen_pos = to_screen.transform_pos(*current_pos);
            let size = self.last_parameter.node_size / 2.;
            let point_rect = Rect::from_center_size(screen_pos, Vec2 { x: size, y: size });
            let point_id = response.id.with(index);
            let point_response = ui.interact(point_rect, point_id, Sense::drag());

            *current_pos += point_response.drag_delta();
            *current_pos = to_screen.from().clamp(*current_pos);
        }
    }

    fn draw_arrow(painter: &Painter, origin: Pos2, vec: Vec2, stroke: Stroke) {
        use egui::emath::*;
        let rot = Rot2::from_angle(std::f32::consts::TAU / 10.0);
        let tip_length = ARROW_TIP_LENGHT;
        let tip = origin + vec;
        let dir = vec.normalized();

        painter.line_segment([origin, tip], stroke);
        painter.line_segment([tip, tip - tip_length * (rot * dir)], stroke);
        painter.line_segment([tip, tip - tip_length * (rot.inverse() * dir)], stroke);
    }

    fn draw_edge(&self, painter: &egui::Painter, to_screen: RectTransform, ui: &egui::Ui) {
        for (from, to, _) in &self.edges {
            if from == to {
                let origin = self.nodes_pos[*from];
                let rotation = egui::emath::Rot2::from_angle(std::f32::consts::PI / 8.);

                let direction_vec =
                    Vec2::new(self.last_parameter.node_size, self.last_parameter.node_size);
                let mut points = [
                    origin,
                    origin + rotation.inverse() * direction_vec,
                    origin + rotation * direction_vec,
                    origin,
                ];
                for pos in &mut points {
                    *pos = to_screen.transform_pos(*pos);
                }
                painter.add(CubicBezierShape::from_points_stroke(
                    points,
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new(ARROW_WIDTH, COLOR_EDGE),
                ));
            } else {
                let origin = to_screen.transform_pos(self.nodes_pos[*from]);
                let end = to_screen.transform_pos(self.nodes_pos[*to]);
                let displacement_vec = (end - origin.to_vec2()).to_vec2();

                let node_radius = Pos2 {
                    x: self.last_parameter.node_size / 2.,
                    y: self.last_parameter.node_size / 2.,
                };
                let node_radius_vec = displacement_vec.normalized() * node_radius.to_vec2();

                Self::draw_arrow(
                    painter,
                    origin + node_radius_vec,
                    displacement_vec - node_radius_vec * 2.,
                    Stroke::new(ARROW_WIDTH, COLOR_EDGE),
                );
            }
        }

        for (from, to, label) in &self.edges {
            if let Some(label) = label {
                if *to == *from {
                    let origin = self.nodes_pos[*from];

                    let direction_vec =
                        Vec2::new(self.last_parameter.node_size, self.last_parameter.node_size);
                    painter.text(
                        to_screen.transform_pos(origin + direction_vec),
                        egui::Align2::CENTER_CENTER,
                        label.to_string(),
                        egui::TextStyle::Body.resolve(ui.style()),
                        COLOR_LABEL_EDGE,
                    );
                } else {
                    let displacement_vec =
                        (self.nodes_pos[*to] - self.nodes_pos[*from].to_vec2()).to_vec2();
                    let middle_point = self.nodes_pos[*from] + displacement_vec / 2.;

                    let displacement_dir = displacement_vec.normalized();
                    let rotation = egui::emath::Rot2::from_angle(std::f32::consts::PI / 2.);
                    let padding_from_arrow: f32 = 5.;

                    // label is drawed perpendicularly to the arrow, from middle point and little displacement,
                    // at padding_from_arrow distance from the arrow
                    let pos = middle_point - displacement_dir * 5.;
                    let pos = pos - padding_from_arrow * (rotation * displacement_dir);
                    let pos = to_screen.transform_pos(pos);

                    painter.text(
                        pos,
                        egui::Align2::CENTER_CENTER,
                        label.to_string(),
                        egui::TextStyle::Body.resolve(ui.style()),
                        COLOR_LABEL_EDGE,
                    );
                }
            }
        }
    }

    fn draw_nodes(&self, painter: &egui::Painter, to_screen: RectTransform, ui: &egui::Ui) {
        use egui::{Align2, TextStyle::Body};

        for (index, pos) in self.nodes_pos.iter() {
            let pos = to_screen.transform_pos(*pos);

            painter.circle_filled(pos, self.last_parameter.node_size / 2., COLOR_NODES);

            if let Some(label) = self.graph.get_node_label(*index) {
                painter.text(
                    pos,
                    Align2::CENTER_CENTER,
                    label,
                    Body.resolve(ui.style()),
                    COLOR_LABEL_NODE,
                );
            }
        }
    }

    pub fn draw(&self, painter: &egui::Painter, to_screen: RectTransform, ui: &egui::Ui) {
        self.draw_edge(painter, to_screen, ui);
        self.draw_nodes(painter, to_screen, ui);
    }
}
