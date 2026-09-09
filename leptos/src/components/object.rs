use ifengine::view::{ImageVariant, Object};
use leptos::prelude::*;

use crate::components::choice::ChoiceView;
use crate::components::line::LineView;
use crate::components::span::SpanView;
use crate::context::StoryContext;
use crate::transition::generate_view_transition_style;

/// Renders an [`Object`] variant from the resolved page [`View`](ifengine::View).
#[component]
pub fn ObjectView(object: Object) -> impl IntoView {
    match object {
        Object::Paragraph(line) => view! {
            <p class="passage-paragraph">
                <LineView line=line />
            </p>
        }
        .into_any(),

        Object::Text(line, render_data) => view! {
            <div class="passage-text" data-render=render_data>
                <LineView line=line />
            </div>
        }
        .into_any(),

        Object::Choice(key, choices) => view! {
            <ChoiceView key=key choices=choices />
        }
        .into_any(),

        Object::Image(img) => {
            let ctx = expect_context::<StoryContext>();
            let src = match &img.variant {
                ImageVariant::Url(url) => url.clone(),
                ImageVariant::Local(uri, _bytes) => uri.to_string(),
            };
            let mut hasher = std::hash::DefaultHasher::new();
            std::hash::Hash::hash(&src, &mut hasher);
            let img_hash = match std::hash::Hasher::finish(&hasher) {
                0 => 1,
                h => h,
            };
            let is_changed = ctx
                .transitions
                .write_untracked()
                .is_content_changed(img.id, img_hash);
            let vt_style = generate_view_transition_style(img.id, is_changed);

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
                1 => view! { <h1 class="passage-heading h1">{heading_view}</h1> }.into_any(),
                2 => view! { <h2 class="passage-heading h2">{heading_view}</h2> }.into_any(),
                3 => view! { <h3 class="passage-heading h3">{heading_view}</h3> }.into_any(),
                4 => view! { <h4 class="passage-heading h4">{heading_view}</h4> }.into_any(),
                5 => view! { <h5 class="passage-heading h5">{heading_view}</h5> }.into_any(),
                _ => view! { <h6 class="passage-heading h6">{heading_view}</h6> }.into_any(),
            }
        }

        Object::Break => view! {
            <hr class="passage-break" />
        }
        .into_any(),

        Object::Empty(n) => {
            let height_em = (n as f32) * 1.5;
            view! {
                <div class="passage-empty" style=format!("height: {height_em}em;") />
            }
            .into_any()
        }

        Object::Quote(line, render_data) => view! {
            <blockquote class="passage-quote" data-render=render_data>
                <LineView line=line />
            </blockquote>
        }
        .into_any(),

        Object::Note(line, _indices) => view! {
            <aside class="passage-note">
                <LineView line=line />
            </aside>
        }
        .into_any(),

        Object::Custom(render_data) => view! {
            <div class="passage-custom" data-custom=render_data />
        }
        .into_any(),

        Object::Embed(embedded_view) => {
            let pid_str = embedded_view.pageid.0.to_string();
            view! {
                <div class="passage-embed" data-page=pid_str>
                    {embedded_view.inner.into_iter().map(move |obj| {
                        view! {
                            <ObjectView object=obj />
                        }
                    }).collect_view()}
                </div>
            }
            .into_any()
        }
    }
}
