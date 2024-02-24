use eframe::CreationContext;

use egui::Response;
use egui_dock::{DockState, NodeIndex, Style, SurfaceIndex};

use std::collections::HashMap;

use egui_terminal::prelude::*;

struct TabViewer {
    handlers: HashMap<usize, TermHandler>,
    default_cmd: String,
    added_nodes: Vec<(SurfaceIndex, NodeIndex)>,
    closed_nodes: Vec<usize>,
    focus_follows_pointer: bool,
    last_focus: usize,
    current_focus: Option<usize>,
    responses: HashMap<usize, Response>,
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
        let term = self.handlers.entry(*tab).or_insert_with(|| TermHandler::new_from_str(&self.default_cmd));
        
        let gui = ui.terminal(
            term
        );

        if self.focus_follows_pointer && gui.hovered() {
            gui.request_focus();
        }

        if gui.has_focus() {
            self.current_focus = Some(*tab);
        }

        self.responses.insert(*tab, gui);

        // if term.is_closed() {
        //     self.closed_nodes.push(*tab);
        // }
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        true
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
                closed_nodes: vec!(),
                focus_follows_pointer: true,
                current_focus: None,
                last_focus: 0,
                responses: HashMap::new(),
            },
            tree: DockState::new(vec!(0)),
            counter: 1,
        }
    }

    pub fn setup (_cc: &CreationContext) -> Box<dyn eframe::App> {
        Box::new(Self::new())
    }

    pub fn exit (&self) {
        std::process::exit(0);
    }
}

impl eframe::App for App {
    fn update (&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.viewer.current_focus = None;
        self.viewer.responses.clear();

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

        self.viewer.closed_nodes.drain(..).for_each(|tab| {
            let tab = self.tree.find_tab(&tab);
            if let Some(tab) = tab {
                self.tree.remove_tab(tab);
            }
        });

        if self.viewer.handlers.len() == 0 {
            self.exit();
        }

        // hack to fix focus issues in egui-terminal
        if let Some(focus) = self.viewer.current_focus {
            self.viewer.last_focus = focus;
        } else {
            self.viewer.responses.get(&self.viewer.last_focus).map(|r| r.request_focus());
        }
    }
}
