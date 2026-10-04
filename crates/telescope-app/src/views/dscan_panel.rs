use gpui_kit::{Context, IntoElement, Render, Window, div};

pub struct DscanPanel;

impl DscanPanel {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for DscanPanel {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}
