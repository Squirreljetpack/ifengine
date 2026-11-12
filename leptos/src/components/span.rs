use ifengine::view::{Span, SpanVariant};
use leptos::prelude::*;

use crate::components::choice::ActiveChoiceContext;
use crate::context::StoryContext;
use crate::render::{span_classes, span_to_css_style};
use crate::transition::{
    ItemKey, ItemKind, TransitionPhase, compute_initial_phase, generate_active_transition_style,
    parse_transition_classes, setup_transition_timers_keyed,
};

/// Renders a single [`Span`] element with styling, modifiers, transition animation,
/// and interactive action handling.
#[component]
pub fn SpanView(
    span: Span,
    #[prop(optional_no_strip)] choice: Option<ActiveChoiceContext>,
    #[prop(default = false)] is_changed: bool,
    #[prop(optional_no_strip)] inherited_config: Option<crate::transition::TransitionConfig>,
    #[prop(optional_no_strip)] inherited_classes: Option<Vec<String>>,
    #[prop(default = 0)] waited_delay: u64,
) -> impl IntoView {
    let ctx = expect_context::<StoryContext>();
    let own_config = parse_transition_classes(&span.classes);
    let mut config = if own_config.has_transition() {
        own_config
    } else if let Some(parent_cfg) = inherited_config {
        parent_cfg
    } else {
        own_config
    };
    if waited_delay > 0
        && let Some(d) = config.in_delay
    {
        config.in_delay = Some(d.saturating_sub(waited_delay));
    }

    let attrs = ctx.transitions.item_attrs(ItemKey {
        id: span.id,
        hash: span.content_hash(),
        changed: is_changed,
        kind: ItemKind::Span,
        classes: &span.classes,
        inherited_classes: inherited_classes.as_deref(),
    });
    let is_changed = attrs.changed;
    let should_animate = config.has_transition() && is_changed;

    let (phase, set_phase) = signal(compute_initial_phase(&config, should_animate));
    if should_animate {
        let transitions = ctx.transitions;
        let id = span.id;
        let hash = span.content_hash();
        if matches!(phase.get_untracked(), TransitionPhase::Active) {
            transitions.mark_seen(id, hash);
        }
        setup_transition_timers_keyed(
            &config,
            &ctx.transitions,
            id.map(|i| (i, hash)),
            set_phase,
        );
    }

    let vt_style = attrs.style;
    let base_style = span_to_css_style(&span);
    let out_fade_style = config
        .fade_out
        .map(|ms| format!("--vt-fade-out: {ms}ms;"))
        .unwrap_or_default();

    let span_style = move || {
        let mut styles = Vec::new();
        if !base_style.is_empty() {
            styles.push(base_style.clone());
        }
        match phase.get() {
            TransitionPhase::Pending | TransitionPhase::Removed => {}
            TransitionPhase::Active => {
                if !vt_style.is_empty() {
                    styles.push(vt_style.clone());
                }
                let active_style = generate_active_transition_style(&config, should_animate);
                if !active_style.is_empty() {
                    styles.push(active_style);
                }
            }
            TransitionPhase::Removing => {
                if !out_fade_style.is_empty() {
                    styles.push(out_fade_style.clone());
                }
            }
        }
        styles.join(" ")
    };

    let span_for_class = span.clone();
    let class_str = move || {
        let mut extra = Vec::new();
        match phase.get() {
            TransitionPhase::Pending => extra.push("delayed-hidden"),
            TransitionPhase::Removing => extra.push("fade-out-removed"),
            TransitionPhase::Removed => extra.push("transition-removed"),
            TransitionPhase::Active => {
                if should_animate {
                    extra.push("transition-active");
                }
            }
        }
        span_classes(&span_for_class, &extra)
    };

    let content = span.content().to_string();
    let vt_name = attrs.name;
    let vt_hash = attrs.hash.to_string();
    let vt_effect = attrs.effect.as_str();
    let vt_kind = attrs.kind.as_str();

    if span.active() {
        let action = span.action.clone();
        let base_class = if matches!(span.variant, SpanVariant::Link) {
            "passage-link"
        } else {
            "passage-action"
        };
        view! {
            <a
                href="javascript:void(0)"
                class=move || format!("{base_class} {}", class_str())
                style=span_style
                data-vt-name=vt_name.clone()
                data-vt-hash=vt_hash.clone()
                data-vt-effect=vt_effect
                data-vt-kind=vt_kind
                on:click=move |e: leptos::ev::MouseEvent| {
                    e.prevent_default();
                    e.stop_propagation();
                    if let Some(c) = choice {
                        ctx.dispatch_choice_action.run(((c.key, c.index), action.clone()));
                    } else {
                        ctx.dispatch_action.run(action.clone());
                    }
                }
            >
                {content}
            </a>
        }
        .into_any()
    } else if matches!(span.variant, SpanVariant::Link) {
        view! {
            <span
                class=move || format!("passage-link-static {}", class_str())
                style=span_style
                data-vt-name=vt_name.clone()
                data-vt-hash=vt_hash.clone()
                data-vt-effect=vt_effect
                data-vt-kind=vt_kind
            >
                {content}
            </span>
        }
        .into_any()
    } else {
        view! {
            <span
                class=class_str
                style=span_style
                data-vt-name=vt_name
                data-vt-hash=vt_hash
                data-vt-effect=vt_effect
                data-vt-kind=vt_kind
            >
                {content}
            </span>
        }
        .into_any()
    }
}
