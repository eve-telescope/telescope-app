//! The annotation editor. Opened for a fixed target from the intel panel and
//! the network's annotation list, or with an entity search to pick one.

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState, Textarea, TextareaState};
use gpui_kit::component::{Disableable as _, Sizable as _, WindowExt as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    App, AppContext as _, Context, Entity, FontWeight, IntoElement, ParentElement as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Subscription, Window, div, px,
};
use telescope_core::view::annotations::{Annotation, EntityType, Target, normalize_annotation_tag};

use crate::state::Stores;
use crate::theme::{self, BORDER, TEXT_1, TEXT_3};
use crate::ui::IconName;
use crate::views::entity_search::{EntitySearch, EntitySearchEvent};
use crate::views::intel_card::{scope_pill, tag_options, target_avatar, toggle_chip};

pub struct AnnotationForm {
    network_id: i64,
    existing: Option<i64>,
    target: Option<Target>,
    search: Option<Entity<EntitySearch>>,
    tags: Vec<String>,
    customs: Vec<(String, String)>,
    new_tag: Entity<InputState>,
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
        let note_text = existing.and_then(|a| a.note.clone()).unwrap_or_default();
        let new_tag = cx.new(|cx| InputState::new(window, cx).placeholder("Add a tag"));
        let note = cx.new(|cx| {
            TextareaState::new(window, cx)
                .rows(4)
                .placeholder("What should your network know?")
                .default_value(note_text)
        });
        let mut subscriptions =
            vec![
                cx.subscribe_in(&new_tag, window, |this, _, event, window, cx| {
                    if matches!(event, InputEvent::PressEnter { .. }) {
                        this.add_typed_tag(window, cx);
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
        let customs = Stores::get(cx)
            .intel
            .read(cx)
            .network_custom_tags(network_id);
        Self {
            network_id,
            existing: existing.map(|a| a.id),
            target,
            search,
            tags: existing.map(|a| a.tags.clone()).unwrap_or_default(),
            customs,
            new_tag,
            note,
            _subscriptions: subscriptions,
        }
    }

    fn toggle(&mut self, tag: &str, cx: &mut Context<Self>) {
        match self.tags.iter().position(|t| t == tag) {
            Some(i) => {
                self.tags.remove(i);
            }
            None => self.tags.push(tag.to_string()),
        }
        cx.notify();
    }

    fn add_typed_tag(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let tag = normalize_annotation_tag(&self.new_tag.read(cx).value());
        if !tag.is_empty() && !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self.new_tag
            .update(cx, |state, cx| state.set_value("", window, cx));
        cx.notify();
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(target) = self.target.clone() else {
            return;
        };
        let note = Some(self.note.read(cx).value().to_string());
        let (network_id, existing, tags) = (self.network_id, self.existing, self.tags.clone());
        Stores::get(cx).intel.update(cx, |intel, cx| {
            intel.save_annotation(network_id, existing, target, tags, note, cx)
        });
        window.close_dialog(cx);
    }

    fn delete(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(entry_id) = self.existing {
            let network_id = self.network_id;
            Stores::get(cx)
                .intel
                .update(cx, |intel, cx| intel.remove_entry(network_id, entry_id, cx));
        }
        window.close_dialog(cx);
    }

    fn render_target(&self, cx: &mut Context<Self>) -> impl IntoElement {
        match (&self.target, &self.search) {
            (Some(target), search) => div()
                .flex()
                .items_center()
                .gap_2p5()
                .pb_3()
                .border_b_1()
                .border_color(theme::color(BORDER))
                .child(target_avatar(target, 32.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme::color(TEXT_1))
                        .child(target.name.clone()),
                )
                .child(scope_pill(target.entity_type))
                .when(search.is_some(), |el| {
                    el.child(
                        Button::new("change-target")
                            .ghost()
                            .xsmall()
                            .label("Change")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.target = None;
                                cx.notify();
                            })),
                    )
                })
                .into_any_element(),
            (None, Some(search)) => search.clone().into_any_element(),
            (None, None) => div().into_any_element(),
        }
    }
}

fn label(text: &'static str) -> impl IntoElement {
    div()
        .text_size(px(10.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme::color(TEXT_3))
        .child(text)
}

impl Render for AnnotationForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(self.render_target(cx))
            .child(label("TAGS"))
            .child(
                div().flex().flex_wrap().gap_1().children(
                    tag_options(&self.tags, &self.customs)
                        .into_iter()
                        .map(|(tag, color)| {
                            let active = self.tags.contains(&tag);
                            toggle_chip(
                                SharedString::from(format!("form-chip-{tag}")).into(),
                                &tag,
                                &color,
                                active,
                            )
                            .on_click(cx.listener(move |this, _, _, cx| this.toggle(&tag, cx)))
                        }),
                ),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().flex_1().child(Input::new(&self.new_tag).small()))
                    .child(
                        Button::new("add-tag")
                            .outline()
                            .small()
                            .icon(IconName::Plus)
                            .on_click(
                                cx.listener(|this, _, window, cx| this.add_typed_tag(window, cx)),
                            ),
                    ),
            )
            .child(label("NOTE"))
            .child(Textarea::new(&self.note).small().h(px(88.)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .when(self.existing.is_some(), |el| {
                        el.child(
                            Button::new("annotation-delete")
                                .ghost()
                                .danger()
                                .icon(IconName::Trash)
                                .label("Delete")
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.delete(window, cx)),
                                ),
                        )
                    })
                    .child(div().flex_1())
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

/// Opens the editor in a dialog. With `target` unset, it starts with an
/// entity search.
pub fn open(
    network_id: i64,
    target: Option<Target>,
    existing: Option<&Annotation>,
    window: &mut Window,
    cx: &mut App,
) {
    let title: SharedString = match existing {
        Some(_) => "Edit annotation".into(),
        None => "Add annotation".into(),
    };
    let form = cx.new(|cx| AnnotationForm::new(network_id, target, existing, window, cx));
    window.open_dialog(cx, move |dialog, _, _| {
        dialog.title(title.clone()).w(px(440.)).child(form.clone())
    });
}
