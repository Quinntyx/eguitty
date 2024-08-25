use eframe::CreationContext;

use egui_dock::{DockState, NodeIndex, Style as DockStyle, SurfaceIndex};
use serde::{Serialize, Deserialize};

use std::collections::HashMap;

use egui_terminal::prelude::*;
use egui_terminal::Style as TermStyle;

struct TabViewer {
    handlers: HashMap<usize, TermHandler>,
    config_file: ConfigFile,
    tab_profiles: HashMap<usize, String>,
    added_nodes: Vec<(SurfaceIndex, NodeIndex)>,
    closed_nodes: Vec<usize>,
    focus_follows_pointer: bool,
    last_focus: usize,
    current_focus: Option<usize>,
}

#[derive(Serialize, Deserialize)]//, Clone)] no clue why this doesnt work figure it out later
struct ConfigFile {
    profiles: HashMap<String, Profile>,
    default_profile: String,
    preferred_config_format: String,
    dock_style: DockStyle,
}

impl Default for ConfigFile {
    fn default () -> Self {
        let mut profiles = HashMap::new();
        let default = Profile {
            terminal_configuration: TermStyle::default(),
            shell_command: String::from("zsh"),
        };
        profiles.insert(String::from("default"), default);

        Self {
            profiles,
            default_profile: "default".to_string(),
            preferred_config_format: "ron".to_string(),
            dock_style: DockStyle::default()
        }
    }
}

#[derive(Serialize, Deserialize)]//, Clone)] (same here see ConfigFile comment)
struct Profile {
    terminal_configuration: TermStyle,
    shell_command: String,
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = usize;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!(
            "{} [{}]", 
            self.handlers.get_mut(tab)
                .map(|t| t.title("eguitty"))
                .unwrap_or(String::from("")),
            tab,
        ).into()
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let cmd = &self.config_file.profiles.get(
            self.tab_profiles.entry(*tab).or_insert_with(|| self.config_file.default_profile.clone())
        ).unwrap().shell_command;

        let term = self.handlers.entry(*tab).or_insert_with(|| TermHandler::new_from_str(&cmd));
        
        let gui = ui.terminal(
            term
        );


        if self.focus_follows_pointer && gui.hovered() {
            gui.request_focus();
        }

        if gui.has_focus() {
            self.current_focus = Some(*tab);
        }

        if term.exit_status().is_some() {
            self.closed_nodes.push(*tab);
        }
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
        let mut path = format!("{}/.config/eguitty", std::env::var("HOME").unwrap());

        if cfg!(profile = "dev") {
            path = String::from("eguitty");
        }

        let conf: ConfigFile = std::fs::read_to_string(format!("{path}.ron"))
            .map(|c| ron::de::from_str(&c).unwrap())
            .ok()
            .unwrap_or_else(
                || std::fs::read_to_string(format!("{path}.json"))
                    .map(|c| serde_json::de::from_str(&c).unwrap())
                    .unwrap_or(ConfigFile::default())
            );
        

        Self {
            viewer: TabViewer {
                handlers: HashMap::new(),
                config_file: conf,
                tab_profiles: HashMap::new(),
                added_nodes: vec!(),
                closed_nodes: vec!(),
                focus_follows_pointer: true,
                current_focus: None,
                last_focus: 0,
            },
            tree: DockState::new(vec!(0)),
            counter: 1,
        }
    }

    pub fn setup<E> (_cc: &CreationContext) -> Result<Box<dyn eframe::App>, E> {
        Ok(Box::new(Self::new()))
    }

    pub fn exit (&self) {
        let mut path = format!("{}/.config/eguitty", std::env::var("HOME").unwrap());

        if cfg!(profile = "dev") {
            path = String::from("eguitty");
        }

        dbg!("called exit");

        if self.viewer.config_file.preferred_config_format == "ron" {
            std::fs::write(&format!("{}.ron", path), ron::ser::to_string_pretty(&self.viewer.config_file, Default::default()).unwrap()).unwrap();
        } else {
            std::fs::write(&format!("{}.json", path), serde_json::ser::to_string_pretty(&self.viewer.config_file).unwrap()).unwrap();
        }

        std::process::exit(0);
    }
}

impl eframe::App for App {
    fn update (&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.viewer.current_focus = None;

        egui_dock::DockArea::new(&mut self.tree)
            .style(DockStyle::from_egui(ctx.style().as_ref()))
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

        if self.viewer.handlers.len() == 0 || ctx.input(|i| i.viewport().close_requested()) {
            self.exit();
        }

        // hack to fix focus issues in egui-terminal
        if let Some(focus) = self.viewer.current_focus {
            self.viewer.last_focus = focus;
        } else {
            self.tree.find_tab(&self.viewer.last_focus).map(|(a, b, _)| self.tree.set_focused_node_and_surface((a, b)));
        }
    }
}
