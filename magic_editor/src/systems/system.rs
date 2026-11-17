use magic_api_editor::Ui;

pub trait EditorSystem: Send + Sync {
    fn on_enable(&mut self);

    fn on_update(&mut self, ui: &mut dyn Ui);

    fn on_disable(&mut self);
}