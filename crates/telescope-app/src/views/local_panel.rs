use gpui_kit::{Context, IntoElement, Render, Window, div};

pub struct LocalPanel;

impl LocalPanel {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for LocalPanel {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}
