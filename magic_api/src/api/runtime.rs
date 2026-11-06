use std::any::Any;

pub struct RuntimeRegistry {
    pub data: String,
    pub registered: usize,
    pub render_context: Option<Box<dyn Any + Send>>,
}

impl RuntimeRegistry {
    pub fn new() -> Self {
        Self {
            data: String::new(),
            registered: 0,
            render_context: None,
        }
    }
}
