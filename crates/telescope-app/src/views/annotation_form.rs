//! The annotation editor: preset tags, free-text tags, a note and a preview.
//! Used from the pilot context menu (fixed target) and the network manager
//! (target picked with the entity search).

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState, Textarea, TextareaState};
use gpui_kit::component::{Disableable as _, Sizable as _, WindowExt as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    App, AppContext as _, Context, Entity, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _,
    Subscription, Window, div, px,
};
use telescope_core::view::annotations::{
    Annotation, DEFAULT_ANNOTATION_COLOR, EntityType, PRESET_ANNOTATION_TAGS, annotation_color,
    parse_annotation_tags,
};
use telescope_core::view::network::toggle_annotation_tag;

use crate::state::Stores;
use crate::theme::{self, BG_3, TEXT_2, TEXT_3};
use crate::views::entity_search::{EntitySearch, EntitySearchEvent};

#[derive(Clone)]
pub struct Target {
    pub entity_type: EntityType,
    pub id: i64,
    pub name: String,
}

pub struct AnnotationForm {
    network_id: i64,
    existing: Option<i64>,
    target: Option<Target>,
    search: Option<Entity<EntitySearch>>,
    tags: Entity<InputState>,
    note: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

impl AnnotationForm {
    fn new(
        network_id: i64,
        target: Option<Target>,
        existing: Option<&Annotation>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let tag_text = existing.map(|a| a.tags.join(" | ")).unwrap_or_default();
        let note_text = existing.and_then(|a| a.note.clone()).unwrap_or_default();
        let tags = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Tags separated by |")
                .default_value(tag_text)
        });
        let note = cx.new(|cx| {
            TextareaState::new(window, cx)
                .rows(4)
                .placeholder("Note")
                .default_value(note_text)
        });
        let mut subscriptions = vec![
            cx.subscribe(&tags, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify()
                }
            }),
            cx.subscribe(&note, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify()
                }
            }),
        ];
        let search = target.is_none().then(|| {
            let search = cx.new(|cx| EntitySearch::new(None, window, cx));
            subscriptions.push(cx.subscribe(&search, |this, _, event, cx| {
                let EntitySearchEvent::Selected(result) = event;
                this.target = EntityType::parse(&result.category).map(|entity_type| Target {
                    entity_type,
                    id: result.id,
                    name: result.name.clone(),
                });
                cx.notify();
            }));
            search
        });
        Self {
            network_id,
            existing: existing.map(|a| a.id),
            target,
            search,
            tags,
            note,
            _subscriptions: subscriptions,
        }
    }

    fn tag_list(&self, cx: &App) -> Vec<String> {
        parse_annotation_tags(&self.tags.read(cx).value())
    }

    fn toggle_preset(&mut self, tag: &str, window: &mut Window, cx: &mut Context<Self>) {
        let next = toggle_annotation_tag(&self.tags.read(cx).value(), tag);
        self.tags
            .update(cx, |state, cx| state.set_value(next, window, cx));
        cx.notify();
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(target) = self.target.clone() else {
            return;
        };
        let tags = self.tag_list(cx);
        let note = Some(self.note.read(cx).value().to_string());
        let (network_id, existing) = (self.network_id, self.existing);
        Stores::get(cx).intel.update(cx, |intel, cx| {
            intel.save_annotation(
                network_id,
                existing,
                target.entity_type,
                target.id,
                target.name,
                tags,
                note,
                cx,
            )
        });
        window.close_dialog(cx);
    }
}

impl Render for AnnotationForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tags = self.tag_list(cx);
        let color = annotation_color(&tags, None)
            .unwrap_or(DEFAULT_ANNOTATION_COLOR)
            .to_string();

        div()
            .flex()
            .flex_col()
            .gap_3()
            .when_some(self.search.clone(), |el, search| {
                el.child(label("TARGET")).child(search)
            })
            .when_some(
                self.target.clone().filter(|_| self.search.is_none()),
                |el, target| {
                    el.child(
                        div()
                            .text_xs()
                            .text_color(theme::color(TEXT_2))
                            .child(format!("{} · {}", target.name, target.entity_type)),
                    )
                },
            )
            .child(label("TAGS"))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_1p5()
                    .children(PRESET_ANNOTATION_TAGS.iter().map(|preset| {
                        let active = tags.iter().any(|t| t == preset.tag);
                        let tag = preset.tag;
                        div()
                            .id(SharedString::from(format!("preset-{}", preset.tag)))
                            .px_2()
                            .py_1()
                            .rounded_sm()
                            .border_1()
                            .text_size(px(10.))
                            .font_weight(FontWeight::BOLD)
                            .cursor_pointer()
                            .text_color(theme::hex_or(preset.color, TEXT_2))
                            .bg(theme::hex_tint(
                                preset.color,
                                if active { 0x33 } else { 0x11 },
                                TEXT_3,
                            ))
                            .border_color(if active {
                                theme::hex_or(preset.color, TEXT_2)
                            } else {
                                gpui_kit::transparent_black()
                            })
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.toggle_preset(tag, window, cx)
                            }))
                            .child(preset.tag)
                    })),
            )
            .child(Input::new(&self.tags).small())
            .child(label("NOTE"))
            .child(Textarea::new(&self.note).small())
            .when(!tags.is_empty(), |el| {
                el.child(label("PREVIEW"))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_1()
                            .children(tags.iter().map(|tag| {
                                div()
                                    .px_1p5()
                                    .rounded_sm()
                                    .text_size(px(10.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .bg(theme::hex_tint(&color, 0x22, BG_3))
                                    .text_color(theme::hex_or(&color, TEXT_2))
                                    .child(tag.clone())
                            })),
                    )
            })
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("annotation-cancel")
                            .ghost()
                            .label("Cancel")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("annotation-save")
                            .primary()
                            .label("Save")
                            .disabled(self.target.is_none())
                            .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
                    ),
            )
    }
}

fn label(text: &'static str) -> impl IntoElement {
    div()
        .text_size(px(10.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme::color(TEXT_3))
        .child(text)
}

/// Opens the editor in a dialog. With `target` unset, the dialog starts with
/// an entity search.
pub fn open(
    network_id: i64,
    target: Option<Target>,
    existing: Option<&Annotation>,
    window: &mut Window,
    cx: &mut App,
) {
    let title: SharedString = match (&target, existing) {
        (Some(t), _) => format!("Annotate {}", t.name).into(),
        (None, Some(_)) => "Edit annotation".into(),
        (None, None) => "Add annotation".into(),
    };
    let form = cx.new(|cx| AnnotationForm::new(network_id, target, existing, window, cx));
    window.open_dialog(cx, move |dialog, _, _| {
        dialog.title(title.clone()).w(px(420.)).child(form.clone())
    });
}
