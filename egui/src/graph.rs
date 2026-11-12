use std::{collections::HashMap, fmt::Display};

use super::app::new;
use egui::Ui;
use egui_snarl::{
    InPin, InPinId, NodeId, OutPin, OutPinId, Snarl,
    ui::{SnarlPin, SnarlViewer},
};
use ifengine::core::PageId;
use ifengine::run::PageRecord;
use rand::Rng;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Node {
    run: PageId,
    record: PageRecord,
    output_count: usize,
    display_width: usize,
}

/// Drops the graph-wide common prefix and a following `::` separator from a page id.
/// Ids shorter than the prefix or split mid-char are returned untouched.
fn short<'a>(id: &'a str, prefix_len: usize) -> &'a str {
    let Some(rest) = id.get(prefix_len..) else {
        return id;
    };
    rest.strip_prefix("::").unwrap_or(rest)
}

// widest body line, so titles can pad the node out to a rectangle
fn body_width(record: &PageRecord, prefix_len: usize) -> usize {
    record
        .ends
        .iter()
        .map(|end| format!("{end:?}").len())
        .chain(record.tags.iter().map(|tag| tag.len()))
        .chain(
            record
                .outgoing_tunnels
                .iter()
                .map(|tunnel| short(tunnel, prefix_len).len()),
        )
        .fold(6, usize::max)
}

// todo: layout, connecting to same pin, disable body showing for empty body
pub fn new_snarl(width: f32, height: f32) -> (Snarl<Node>, usize) {
    let app = new();
    let sim = app.simulate(|s| s.depth <= 20);

    // node_id -> snarl_node_id, output_count
    let mut seen = HashMap::new();
    let mut queue = vec![];
    let mut snarl = Snarl::new();
    let mut lcp: Option<String> = None;
    let run_count = sim.runs.len() as f32;

    for (run_idx, (run_id, run)) in sim.runs.into_iter().enumerate() {
        let run_depth = (run.depth() + 1) as f32;
        for record in run.0.into_iter() {
            match &mut lcp {
                None => {
                    lcp = Some(record.id.to_string());
                }
                Some(prefix) => {
                    let new_len = prefix
                        .char_indices()
                        .zip(record.id.chars())
                        .take_while(|((_, a), b)| *a == *b)
                        .map(|((i, c), _)| i + c.len_utf8())
                        .last()
                        .unwrap_or(0);

                    prefix.truncate(new_len);
                }
            }

            let incoming = record.incoming.clone();
            let record_id = record.id.clone();

            let node = Node {
                run: run_id.clone(),
                record: record.clone(),
                output_count: 0,
                display_width: 0,
            };

            let p_x = record.min_depth as f32 / run_depth;
            let p_y = run_idx as f32 / run_count;
            let pos: egui::Pos2 = random_pos(
                p_x,
                p_y,
                width,
                height,
                egui::Margin {
                    left: 0,
                    right: 100,
                    top: 0,
                    bottom: 10,
                },
            );
            let node_id = if record.is_empty() {
                snarl.insert_node_collapsed(pos, node)
            } else {
                snarl.insert_node(pos, node)
            };
            queue.push((node_id, run_id.clone(), incoming));
            seen.insert((run_id.clone(), record_id), (node_id, 0));
        }
    }
    for (node, run_id, incoming) in queue.into_iter() {
        for (input, pageid) in incoming.into_iter().enumerate() {
            if let Some((incoming_node, output)) = seen.get_mut(&(run_id.clone(), pageid.clone())) {
                snarl.connect(
                    OutPinId {
                        node: *incoming_node,
                        output: *output,
                    },
                    InPinId { node, input },
                );
                *output += 1;
            } else {
                eprintln!("Missing page id {:?} in run {:?}", pageid, run_id);
            };
        }
    }

    let prefix_len = lcp.map(|prefix| prefix.len()).unwrap_or_default();

    for Node {
        run,
        record,
        output_count,
        display_width,
    } in snarl.nodes_mut()
    {
        *output_count = seen
            .get(&(run.clone(), record.id.clone()))
            .map(|x| x.1)
            .unwrap_or_default();
        *display_width = body_width(record, prefix_len);
    }

    (snarl, prefix_len)
}

pub const SCALING: f32 = 1.5; // snarl elements scale

// todo: egui maintains a margin on the initialization positions, which cannot be shifted left
fn random_pos(p_x: f32, _p_y: f32, width: f32, height: f32, margin: egui::Margin) -> egui::Pos2 {
    let mut rng = rand::rng();

    let half_width = width / 2.0;
    let half_height = height / 2.0;

    let x =
        -half_width + margin.left as f32 + p_x * (width - margin.left as f32 - margin.right as f32);
    let y = rng.random_range(-half_height + margin.top as f32..=half_height - margin.bottom as f32);

    egui::pos2(x * SCALING, y * SCALING)
}

#[derive(Default, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GraphViewer {
    /// Length of the graph-wide common page id prefix, stripped from titles and labels.
    pub prefix_len: usize,
    pub init_transform: Option<egui::emath::TSTransform>,
}

#[allow(unused)]
impl SnarlViewer<Node> for GraphViewer {
    fn title(&mut self, node: &Node) -> String {
        let mut s = short(&node.record.id, self.prefix_len).to_string();
        while s.len() < node.display_width {
            s.push(' ');
        }
        s
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<Node>,
    ) -> impl SnarlPin + 'static {
        egui_snarl::ui::PinInfo::circle()
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<Node>,
    ) -> impl SnarlPin + 'static {
        egui_snarl::ui::PinInfo::circle()
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<Node>) -> bool {
        false
    }

    // right click (i.e. remove)
    fn has_node_menu(&mut self, _node: &Node) -> bool {
        false
    }

    fn inputs(&mut self, node: &Node) -> usize {
        node.record.incoming.len()
    }

    fn outputs(&mut self, node: &Node) -> usize {
        node.output_count
    }

    fn show_body(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<Node>,
    ) {
        let Some(Node { run, record, .. }) = snarl.get_node(node) else {
            return;
        };
        ui.vertical(|ui| {
            if !record.tags.is_empty() {
                ui.label(format_list("Tags: ", record.tags.iter()));
            };
            if !record.ends.is_empty() {
                ui.label(format_list("Ends: ", record.ends.iter()));
            };
            if !record.outgoing_tunnels.is_empty() {
                ui.label(format_list(
                    "Tunnels: ",
                    record
                        .outgoing_tunnels
                        .iter()
                        .map(|tunnel| short(tunnel, self.prefix_len)),
                ));
            };
        });
    }

    fn has_body(&mut self, node: &Node) -> bool {
        !node.record.is_empty()
    }

    fn current_transform(
        &mut self,
        to_global: &mut egui::emath::TSTransform,
        _snarl: &mut Snarl<Node>,
    ) {
        if self.init_transform.is_none() {
            to_global.scaling = 1.0 / SCALING;
            self.init_transform = Some(*to_global);
        }
    }
}

fn format_list<T: Display>(prefix: &str, items: impl Iterator<Item = T>) -> String {
    let mut out = String::new();
    out.push_str(prefix);
    for (i, item) in items.enumerate() {
        if i > 0 {
            out.push_str(",\n  ");
        }
        out.push_str(&item.to_string());
    }

    out
}
