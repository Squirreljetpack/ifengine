use ifengine::view::{ImageVariant, Object, StampedObject};
use leptos::prelude::*;

use crate::components::choice::ChoiceView;
use crate::components::line::LineView;
use crate::components::span::SpanView;
use crate::context::StoryContext;
use crate::transition::{generate_view_transition_style, is_line_delayed, line_in_delay};

/// Renders a [`StampedObject`] from the resolved page [`View`](ifengine::View).
#[component]
pub fn ObjectView(stamped: StampedObject) -> impl IntoView {
    let ctx = expect_context::<StoryContext>();
    let id = stamped.id;
    let is_changed = ctx
        .transitions
        .write_untracked()
        .is_content_changed(id, stamped.content_hash());
    let vt_style = generate_view_transition_style(id, is_changed);

    match stamped.object {
        Object::Paragraph(line, render_data) => {
            let is_delayed = is_line_delayed(&line);
            let is_fresh = ctx.transitions.read_untracked().is_fresh;
            let should_delay = is_delayed && !is_fresh && is_changed;
            let (is_pending, set_pending) = signal(should_delay);
            if should_delay {
                let delay = line_in_delay(&line);
                leptos::prelude::set_timeout(
                    move || {
                        set_pending.set(false);
                    },
                    std::time::Duration::from_millis(delay),
                );
            }

            if render_data == "popup" || render_data == "modal" {
                view! {
                    <div class="passage-popup-backdrop" data-render=render_data>
                        <div class="passage-popup-dialog">
                            <div class="passage-text" data-render=render_data>
                                <LineView line=line />
                            </div>
                        </div>
                    </div>
                }
                .into_any()
            } else if let Some(speaker) = render_data.strip_prefix(':') {
                let speaker = speaker.trim().to_string();
                let vt_clone = vt_style.clone();
                let dialogue_style = move || {
                    if !is_pending.get() && !vt_clone.is_empty() {
                        vt_clone.clone()
                    } else {
                        String::new()
                    }
                };
                view! {
                    <p class="passage-paragraph passage-dialogue" data-render=render_data style=dialogue_style>
                        <span class="dialogue-speaker">{speaker}</span>
                        <LineView line=line />
                    </p>
                }
                .into_any()
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

                let vt_clone = vt_style.clone();
                let p_style = move || {
                    let mut parts = Vec::new();
                    if !margin_style.is_empty() {
                        parts.push(margin_style.clone());
                    }
                    if !is_pending.get() && !vt_clone.is_empty() {
                        parts.push(vt_clone.clone());
                    }
                    parts.join(" ")
                };

                if !render_data.is_empty() {
                    view! {
                        <p class="passage-paragraph" data-render=render_data style=p_style>
                            <LineView line=line />
                        </p>
                    }
                    .into_any()
                } else {
                    view! {
                        <p class="passage-paragraph" style=p_style>
                            <LineView line=line />
                        </p>
                    }
                    .into_any()
                }
            }
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
            let action_opt = img.action;

            if let Some(action) = action_opt {
                view! {
                    <div class="passage-image-wrapper" style=vt_style>
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
                    <div class="passage-image-wrapper" style=vt_style>
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
                1 => view! { <h1 class="passage-heading h1" style=vt_style>{heading_view}</h1> }
                    .into_any(),
                2 => view! { <h2 class="passage-heading h2" style=vt_style>{heading_view}</h2> }
                    .into_any(),
                3 => view! { <h3 class="passage-heading h3" style=vt_style>{heading_view}</h3> }
                    .into_any(),
                4 => view! { <h4 class="passage-heading h4" style=vt_style>{heading_view}</h4> }
                    .into_any(),
                5 => view! { <h5 class="passage-heading h5" style=vt_style>{heading_view}</h5> }
                    .into_any(),
                _ => view! { <h6 class="passage-heading h6" style=vt_style>{heading_view}</h6> }
                    .into_any(),
            }
        }

        Object::Break => view! {
            <hr class="passage-break" style=vt_style />
        }
        .into_any(),

        Object::Empty(n) => {
            let height_em = (n as f32) * 1.5;
            let style_str = format!("height: {height_em}em; {vt_style}");
            view! {
                <div class="passage-empty" style=style_str />
            }
            .into_any()
        }


        Object::Note(line, _indices) => view! {
            <aside class="passage-note" style=vt_style>
                <LineView line=line />
            </aside>
        }
        .into_any(),

        Object::Embed(embedded_view, render_data) => {
            let pid_str = embedded_view.pageid.0.to_string();
            let is_empty = embedded_view.inner.is_empty();

            if is_empty {
                view! {
                    <div class="passage-embed passage-custom" data-page=pid_str data-render=render_data style=vt_style />
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
                    <div class="passage-embed" data-page=pid_str data-render=render_data style=vt_style>
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
