use ifengine::PageKey;
use ifengine::view::Line;
use leptos::prelude::*;

use crate::components::choice::ActiveChoiceContext;
use crate::components::span::SpanView;
use crate::consts::AUTO_SPAN_TRANSITIONS;
use crate::transition::{
    TransitionPhase, compute_initial_phase, generate_active_transition_style,
    parse_transition_classes, setup_transition_timers,
};

/// Renders a [`Line`] consisting of multiple spans, with line-level animation and class support.
#[component]
pub fn LineView(
    line: Line,
    #[prop(optional_no_strip)] choice: Option<ActiveChoiceContext>,
    #[prop(default = false)] is_changed: bool,
    #[prop(optional_no_strip)] parent_id: Option<PageKey>,
) -> impl IntoView {
    let config = parse_transition_classes(&line.classes);
    let should_animate = config.has_transition() && is_changed;

    let (phase, set_phase) = signal(compute_initial_phase(&config, should_animate));
    if should_animate {
        setup_transition_timers(&config, set_phase);
    }

    let line_config = config.clone();
    let line_style = move || {
        let mut styles = Vec::new();
        match phase.get() {
            TransitionPhase::Pending | TransitionPhase::Removed => {}
            TransitionPhase::Active => {
                let (trans_style, _) =
                    generate_active_transition_style(&line_config, should_animate);
                if !trans_style.is_empty() {
                    styles.push(trans_style);
                }
            }
        }
        styles.join(" ")
    };

    let line_classes_base = line.classes.clone();
    let class_str = move || {
        let mut classes = line_classes_base.clone();
        classes.push("passage-line".into());
        match phase.get() {
            TransitionPhase::Pending => classes.push("delayed-hidden".into()),
            TransitionPhase::Removed => classes.push("fade-out-removed".into()),
            TransitionPhase::Active => {
                if should_animate {
                    classes.push("transition-active".into());
                } else {
                    classes.push("skip-animation".into());
                }
            }
        }
        classes.join(" ")
    };

    let ctx = expect_context::<crate::context::StoryContext>();
    let parent = parent_id.unwrap_or(0);
    let mut nodes = build_group_tree(&line.spans);
    if AUTO_SPAN_TRANSITIONS {
        assign_auto_keys(&mut nodes, parent);
    }

    let rendered_nodes = render_nodes(nodes, choice, &ctx);

    view! {
        <span class=class_str style=line_style>
            {rendered_nodes}
        </span>
    }
}

#[derive(Clone, Debug)]
enum GroupNode {
    Span(ifengine::view::Span),
    Group {
        id: PageKey,
        children: Vec<GroupNode>,
    },
}

fn build_group_tree(spans: &[ifengine::view::Span]) -> Vec<GroupNode> {
    let mut nodes = Vec::new();
    let mut i = 0;
    while i < spans.len() {
        let span = &spans[i];
        if let Some(id) = span.id {
            let last_idx = spans[i..]
                .iter()
                .rposition(|s| s.id == Some(id))
                .map(|pos| i + pos)
                .unwrap_or(i);

            if last_idx > i {
                let mut inner = Vec::new();
                if !spans[i].content.is_empty() {
                    inner.push(spans[i].clone());
                }
                if last_idx > i + 1 {
                    inner.extend_from_slice(&spans[i + 1..last_idx]);
                }
                if last_idx > i && !spans[last_idx].content.is_empty() {
                    inner.push(spans[last_idx].clone());
                }

                for s in &mut inner {
                    if s.id == Some(id) {
                        s.id = None;
                    }
                }

                let children = build_group_tree(&inner);
                nodes.push(GroupNode::Group { id, children });
                i = last_idx + 1;
            } else {
                nodes.push(GroupNode::Span(span.clone()));
                i += 1;
            }
        } else {
            nodes.push(GroupNode::Span(span.clone()));
            i += 1;
        }
    }
    nodes
}

fn assign_auto_keys(nodes: &mut [GroupNode], parent: PageKey) {
    for (index, node) in nodes.iter_mut().enumerate() {
        let slot = (index as u64).saturating_add(1);
        match node {
            GroupNode::Span(span) => {
                // Action spans without IDs need transition names too; an
                // existing ID always remains the element's explicit identity.
                if span.id.is_none() && !span.content.trim().is_empty() {
                    let auto_key = (parent.rotate_left(7) ^ slot) | (1u64 << 63);
                    span.id = Some(auto_key);
                }
            }
            GroupNode::Group { id, children } => {
                assign_auto_keys(children, *id);
            }
        }
    }
}

fn render_nodes(
    nodes: Vec<GroupNode>,
    choice: Option<ActiveChoiceContext>,
    ctx: &crate::context::StoryContext,
) -> Vec<AnyView> {
    nodes
        .into_iter()
        .map(|node| match node {
            GroupNode::Span(span) => view! {
                <SpanView span=span choice=choice />
            }
            .into_any(),
            GroupNode::Group { id, children } => {
                let is_changed = ctx
                    .transitions
                    .write_untracked()
                    .is_revealed(Some(id));
                let vt_style =
                    crate::transition::generate_view_transition_style(Some(id), is_changed);

                let child_views = render_nodes(children, choice, ctx);
                view! {
                    <span class="passage-inline-group" style=vt_style>
                        {child_views}
                    </span>
                }
                .into_any()
            }
        })
        .collect()
}
