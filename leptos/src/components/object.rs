use ifengine::view::{ImageVariant, Object, StampedObject};
use leptos::prelude::*;

use crate::components::choice::ChoiceView;
use crate::components::line::LineView;
use crate::components::span::SpanView;
use crate::context::StoryContext;
use crate::transition::{ItemKey, ItemKind, is_line_delayed, line_in_delay};

/// Renders a [`StampedObject`] from the resolved page [`View`](ifengine::View).
#[component]
pub fn ObjectView(stamped: StampedObject) -> impl IntoView {
    let ctx = expect_context::<StoryContext>();
    let id = stamped.id;
    let content_hash = stamped.content_hash();
    let kind = match &stamped.object {
        Object::Image(_) => ItemKind::Image,
        Object::Break | Object::Empty(_) => ItemKind::Spacer,
        _ => ItemKind::Block,
    };
    let attrs = ctx.transitions.item_attrs(ItemKey {
        id,
        hash: content_hash,
        changed: false,
        kind,
        classes: &[],
        inherited_classes: None,
    });
    let is_changed = attrs.changed;
    let is_delayed = match &stamped.object {
        Object::Paragraph(line, _) | Object::Note(line, _) => is_line_delayed(line),
        _ => false,
    };
    let vt_style = attrs.style.clone();
    let vt_name = attrs.name.clone();
    let vt_hash = attrs.hash.to_string();
    let vt_effect = attrs.effect.as_str();
    let vt_kind = attrs.kind.as_str();

    match stamped.object {
        Object::Paragraph(line, render_data) => {
            let should_delay = is_delayed && is_changed;
            let delay = if should_delay {
                line_in_delay(&line)
            } else {
                0
            };
            let (is_pending, set_pending) = signal(should_delay);
            if should_delay {
                let transitions = ctx.transitions;
                ctx.transitions.schedule(delay, move || {
                    transitions.mark_seen(id, content_hash);
                    set_pending.set(false);
                });
            }

            let render_content = move || {
                if is_pending.get() {
                    return None;
                }

                if render_data == "popup" || render_data == "modal" {
                    Some(view! {
                        <div class="passage-popup-backdrop" data-render=render_data>
                            <div class="passage-popup-dialog">
                                <div class="passage-text" data-render=render_data>
                                    <LineView line=line.clone() parent_id=id is_changed=is_changed waited_delay=delay />
                                </div>
                            </div>
                        </div>
                    }.into_any())
                } else if let Some(speaker) = render_data.strip_prefix(':') {
                    let speaker = speaker.trim().to_string();
                    let speaker_span = ifengine::view::Span::from(speaker).cls("dialogue-speaker");
                    Some(view! {
                        <p class="passage-paragraph passage-dialogue" data-render=render_data>
                            <SpanView
                                span=speaker_span
                                is_changed=is_changed
                                inherited_classes=Some(line.classes.clone())
                                waited_delay=delay
                            />
                            <LineView line=line.clone() parent_id=id is_changed=is_changed waited_delay=delay />
                        </p>
                    }.into_any())
                } else {
                    let margin_style = if let Some(m_str) = render_data
                        .strip_prefix("m-")
                        .or_else(|| render_data.strip_prefix("mt-"))
                    {
                        if let Ok(val) = m_str.parse::<f32>() {
                            if val == 0.0 {
                                "margin-top: 0;".to_string()
                            } else {
                                format!("margin-top: {val}rem;")
                            }
                        } else {
                            format!("margin-top: {m_str};")
                        }
                    } else {
                        String::new()
                    };

                    let p_style = margin_style;

                    if !render_data.is_empty() {
                        Some(view! {
                            <p class="passage-paragraph" data-render=render_data style=p_style>
                                <LineView line=line.clone() parent_id=id is_changed=is_changed waited_delay=delay />
                            </p>
                        }.into_any())
                    } else {
                        Some(view! {
                            <p class="passage-paragraph" style=p_style>
                                <LineView line=line.clone() parent_id=id is_changed=is_changed waited_delay=delay />
                            </p>
                        }.into_any())
                    }
                }
            };

            view! {
                {render_content}
            }
            .into_any()
        }

        Object::Choice(choices) => {
            let key = id.unwrap_or(0);
            view! {
                <ChoiceView key=key choices=choices />
            }
            .into_any()
        }

        Object::Image(img) => {
            let src = match &img.variant {
                ImageVariant::Url(url) => url.clone(),
                ImageVariant::Local(uri, _bytes) => uri.to_string(),
            };

            let mut style_parts = Vec::new();
            let [w, h] = img.size;
            if w > 0 {
                style_parts.push(format!("max-width: {w}px; width: 100%;"));
            }
            if h > 0 {
                style_parts.push(format!("max-height: {h}px;"));
            }
            let img_style = style_parts.join(" ");

            let alt_text = img.alt.clone();
            let action = img.action;

            if !action.is_empty() {
                view! {
                    <div
                        class="passage-image-wrapper"
                        style=vt_style
                        data-vt-name=vt_name.clone()
                        data-vt-hash=vt_hash.clone()
                        data-vt-effect=vt_effect
                        data-vt-kind=vt_kind
                    >
                        <button
                            type="button"
                            class="image-action-button"
                            on:click=move |e: leptos::ev::MouseEvent| {
                                e.prevent_default();
                                ctx.dispatch_action.run(action.clone());
                            }
                        >
                            <img src=src alt=alt_text style=img_style class="passage-image" />
                        </button>
                    </div>
                }
                .into_any()
            } else {
                view! {
                    <div
                        class="passage-image-wrapper"
                        style=vt_style
                        data-vt-name=vt_name.clone()
                        data-vt-hash=vt_hash.clone()
                        data-vt-effect=vt_effect
                        data-vt-kind=vt_kind
                    >
                        <img src=src alt=alt_text style=img_style class="passage-image" />
                    </div>
                }
                .into_any()
            }
        }

        Object::Heading(span, level) => {
            let heading_view = view! {
                <SpanView span=span />
            };

            match level {
                1 => view! { <h1 class="passage-heading h1">{heading_view}</h1> }.into_any(),
                2 => view! { <h2 class="passage-heading h2">{heading_view}</h2> }.into_any(),
                3 => view! { <h3 class="passage-heading h3">{heading_view}</h3> }.into_any(),
                4 => view! { <h4 class="passage-heading h4">{heading_view}</h4> }.into_any(),
                5 => view! { <h5 class="passage-heading h5">{heading_view}</h5> }.into_any(),
                _ => view! { <h6 class="passage-heading h6">{heading_view}</h6> }.into_any(),
            }
        }

        Object::Break => view! {
            <hr
                class="passage-break"
                style=vt_style
                data-vt-name=vt_name.clone()
                data-vt-hash=vt_hash.clone()
                data-vt-effect=vt_effect
                data-vt-kind=vt_kind
            />
        }
        .into_any(),

        Object::Empty(n) => {
            let height_em = (n as f32) * 1.5;
            let style_str = format!("height: {height_em}em; {vt_style}");
            view! {
                <div
                    class="passage-empty"
                    style=style_str
                    data-vt-name=vt_name.clone()
                    data-vt-hash=vt_hash.clone()
                    data-vt-effect=vt_effect
                    data-vt-kind=vt_kind
                />
            }
            .into_any()
        }

        Object::Note(line, _indices) => {
            let is_delayed = is_line_delayed(&line);
            let should_delay = is_delayed && is_changed;
            let delay = if should_delay {
                line_in_delay(&line)
            } else {
                0
            };
            let (is_pending, set_pending) = signal(should_delay);
            if should_delay {
                let transitions = ctx.transitions;
                ctx.transitions.schedule(delay, move || {
                    transitions.mark_seen(id, content_hash);
                    set_pending.set(false);
                });
            }

            let render_content = move || {
                if is_pending.get() {
                    return None;
                }
                Some(
                    view! {
                        <aside class="passage-note">
                            <LineView line=line.clone() parent_id=id is_changed=is_changed waited_delay=delay />
                        </aside>
                    }
                    .into_any(),
                )
            };
            view! {
                {render_content}
            }
            .into_any()
        }

        Object::Embed(embedded_view, render_data) => {
            let pid_str = embedded_view.pageid.0.to_string();
            let is_empty = embedded_view.inner.is_empty();

            if is_empty {
                view! {
                    <div class="passage-embed passage-custom" data-page=pid_str data-render=render_data />
                }
                .into_any()
            } else if render_data == "popup" || render_data == "modal" {
                view! {
                    <div class="passage-popup-backdrop" data-render=render_data>
                        <div class="passage-popup-dialog">
                            <div class="passage-embed" data-page=pid_str>
                                {embedded_view.inner.into_iter().map(move |stamped| {
                                    view! {
                                        <ObjectView stamped=stamped />
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    </div>
                }
                .into_any()
            } else {
                view! {
                    <div class="passage-embed" data-page=pid_str data-render=render_data>
                        {embedded_view.inner.into_iter().map(move |stamped| {
                            view! {
                                <ObjectView stamped=stamped />
                            }
                        }).collect_view()}
                    </div>
                }
                .into_any()
            }
        }
    }
}
