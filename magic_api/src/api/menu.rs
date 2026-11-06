use crate::*;

pub type CommandCallback = Box<dyn Fn() + Send + Sync + 'static>;

pub enum MenuItem {
    Action {
        label: String,
        callback: CommandCallback,
    },
    Submenu {
        label: String,
        items: Vec<MenuItem>,
    },
    Separator,
}

pub struct MenuEntry {
    pub name: String,
    pub label: String,
    pub items: Vec<MenuItem>,
    pub reserved: bool,
}

pub struct MenuRegistry {
    pub menus: Vec<MenuEntry>,
}

impl MenuRegistry {
    pub fn new() -> Self {
        let menus = crate::default_menus::CORE_MENU_ORDER
            .iter()
            .map(|(name, label)| MenuEntry {
                name: name.to_string(),
                label: label.to_string(),
                items: Vec::new(),
                reserved: true,
            })
            .collect();
        Self { menus }
    }

    pub fn find_menu_index(&self, menu_name: &str) -> Option<usize> {
        self.menus.iter().position(|m| m.name == menu_name)
    }

    pub fn register_action(&mut self, path: &[&str], label: &str, callback: CommandCallback) {
        if path.is_empty() {
            return;
        }

        let menu_name = path[0];

        let menu_idx = {
            let names: Vec<String> = self.menus.iter().map(|m| m.name.clone()).collect();
            match names.iter().position(|n| n == menu_name) {
                Some(idx) => idx,
                None => {
                    self.menus.push(MenuEntry {
                        name: menu_name.to_string(),
                        label: menu_name.to_string(),
                        items: Vec::new(),
                        reserved: false,
                    });
                    self.menus.len() - 1
                }
            }
        };

        add_item_to_menu_items(&mut self.menus[menu_idx].items, &path[1..], MenuItem::Action {
            label: label.to_string(),
            callback,
        });
    }

    pub fn register_submenu(&mut self, path: &[&str], submenu_label: &str) {
        if path.is_empty() {
            return;
        }

        let menu_name = path[0];

        let menu_idx = {
            let names: Vec<String> = self.menus.iter().map(|m| m.name.clone()).collect();
            match names.iter().position(|n| n == menu_name) {
                Some(idx) => idx,
                None => {
                    self.menus.push(MenuEntry {
                        name: menu_name.to_string(),
                        label: menu_name.to_string(),
                        items: Vec::new(),
                        reserved: false,
                    });
                    self.menus.len() - 1
                }
            }
        };

        add_item_to_menu_items(&mut self.menus[menu_idx].items, &path[1..], MenuItem::Submenu {
            label: submenu_label.to_string(),
            items: Vec::new(),
        });
    }

    pub fn register_in_core(&mut self, core_menu_name: &str, label: &str, callback: CommandCallback) {
        self.register_action(&[core_menu_name], label, callback);
    }

    pub fn clear(&mut self) {
        for menu in &mut self.menus {
            if menu.reserved {
                menu.items.clear();
            }
        }
        self.menus.retain(|m| m.reserved);
        mgprint!("MenuRegistry", "All top editor menus have been successfully reset and cleared.");
    }
}

fn add_item_to_menu_items(items: &mut Vec<MenuItem>, remaining_path: &[&str], item: MenuItem) {
    if remaining_path.is_empty() {
        items.push(item);
        return;
    }

    let next_name = remaining_path[0];

    let existing_idx = items.iter().position(|item| match item {
        MenuItem::Submenu { label, .. } => label == next_name,
        _ => false,
    });

    match existing_idx {
        Some(idx) => {
            if let MenuItem::Submenu { items: sub_items, .. } = &mut items[idx] {
                add_item_to_menu_items(sub_items, &remaining_path[1..], item);
            }
        }
        None => {
            let mut new_items = Vec::new();
            add_item_to_menu_items(&mut new_items, &remaining_path[1..], item);
            items.push(MenuItem::Submenu {
                label: next_name.to_string(),
                items: new_items,
            });
        }
    }
}
