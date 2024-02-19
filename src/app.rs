use eframe::CreationContext;

use egui_dock::{DockState, Style, NodeIndex, SurfaceIndex};

use std::collections::HashMap;

use egui_terminal::prelude::*;

struct TabViewer {
    handlers: HashMap<usize, TermHandler>,
    default_cmd: String,
    added_nodes: Vec<(SurfaceIndex, NodeIndex)>
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = usize;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!(
            "{} [{}]", 
            self.handlers.get(tab)
                .map(|t| t.title("eguitty"))
                .unwrap_or(String::from("")),
            tab,
        ).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.terminal(
            self.handlers.entry(*tab).or_insert(TermHandler::new_from_str(&self.default_cmd))
        );
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        self.handlers.len() > 1
    }

    fn on_add(&mut self, surface: SurfaceIndex, node: NodeIndex) {
        self.added_nodes.push((surface, node));
    }
}

pub struct App {
    viewer: TabViewer,
    tree: egui_dock::DockState<usize>,
    counter: usize,
}

impl App {
    pub fn new () -> Self {
        Self {
            viewer: TabViewer {
                handlers: HashMap::new(),
                default_cmd: String::from("zsh"),
                added_nodes: vec!(),
            },
            tree: DockState::new(vec!(0)),
            counter: 1,
        }
    }

    pub fn setup (_cc: &CreationContext) -> Box<dyn eframe::App> {
        Box::new(Self::new())
    }
}

impl eframe::App for App {
    fn update (&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_dock::DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show_add_buttons(true)
            .show(ctx, &mut self.viewer);

        let dead_handlers = self.viewer.handlers.keys()
            .filter(|k| self.tree.find_tab(k).is_none())
            .map(|k| k.clone())
            .collect::<Vec<usize>>();

        dead_handlers.iter().for_each(|k| {
            self.viewer.handlers.remove(k).unwrap();
        });


        self.viewer.added_nodes.drain(..).for_each(|(surface, node)| {
            self.tree.set_focused_node_and_surface((surface, node));
            self.tree.push_to_focused_leaf(self.counter);
            self.counter += 1;
        });
    }
}
