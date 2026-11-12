// View Transitions strategy for ifengine_leptos.
//
// Receives the resolved plan from Rust, snapshots pre-update geometry and hashes from
// live `[data-vt-name]` elements, normalizes each old element to the class its
// counterpart will use, runs the state update inside `document.startViewTransition`,
// then classifies each named element after the update:
//   no old counterpart / hash change / size delta / x-shift -> fade
//   otherwise                                               -> reflow
// The plan's `changed` flag is deliberately not consulted: it reports first sight, which
// also covers elements that are already on screen and only need to move. Durations still
// come from the plan (class names are never parsed here).
export function run_view_transition(planJson, isSecondary, updateFn, onFinished) {
    let plan;
    try {
        plan = JSON.parse(planJson);
    } catch (err) {
        // Degrade to a plain update rather than leaving the batch half-applied.
        console.warn('ifengine: unreadable transition plan', err);
        updateFn();
        onFinished();
        return;
    }
    if (!isSecondary) {
        if (plan.primaryInDelay != null) {
            document.documentElement.style.setProperty('--primary-in-delay', plan.primaryInDelay + 'ms');
        }
        if (plan.primaryBlockInDelay != null) {
            document.documentElement.style.setProperty('--primary-block-in-delay', plan.primaryBlockInDelay + 'ms');
        }
    }

    const byName = new Map();
    for (const item of plan.items || []) {
        if (item.name) byName.set(item.name, item);
    }

    const oldMap = new Map();
    for (const el of document.querySelectorAll('[data-vt-name]')) {
        if (isHidden(el)) continue;
        const name = el.dataset.vtName;
        const hash = el.dataset.vtHash || '';
        oldMap.set(name, {
            rect: el.getBoundingClientRect(),
            hash
        });

        // The old snapshot is captured with the class currently on the element, so
        // normalize it against the plan: leaving or rewritten items fade, stable items
        // only slide.
        const next = byName.get(name);
        if (next && next.hash && hash && next.hash === hash) patchToReflow(el);
        else patchToFade(el);
    }

    const vt = document.startViewTransition(() => {
        updateFn();
        return new Promise(resolve => {
            setTimeout(() => {
                classifyAndTag(plan, byName, oldMap, isSecondary);
                resolve();
            }, 0);
        });
    });

    function cleanup() {
        document.documentElement.style.removeProperty('--primary-in-delay');
        document.documentElement.style.removeProperty('--primary-block-in-delay');
        onFinished();
    }

    if (vt && vt.finished && vt.finished.then) {
        vt.finished.then(cleanup, cleanup);
    } else {
        cleanup();
    }
}

function isHidden(el) {
    return !el.dataset.vtName
        || el.closest('.delayed-hidden')
        || el.classList.contains('delayed-hidden')
        || el.classList.contains('fade-out-removed')
        || el.classList.contains('transition-removed');
}

function patchToFade(el, isBlockIn) {
    const vtClass = el.style.viewTransitionClass || '';
    const cleaned = vtClass.replace(/\breflow\b/g, '').trim();
    let nextClass = cleaned.includes('fade') ? cleaned : (cleaned ? cleaned + ' fade' : 'fade');
    if (isBlockIn && !nextClass.includes('block-in')) {
        nextClass += ' block-in';
    }
    el.style.setProperty('view-transition-class', nextClass);
}

function patchToReflow(el) {
    const vtClass = el.style.viewTransitionClass || '';
    const cleaned = vtClass.replace(/\bfade\b/g, '').replace(/\bblock-in\b/g, '').trim();
    el.style.setProperty(
        'view-transition-class',
        cleaned.includes('reflow') ? cleaned : (cleaned ? cleaned + ' reflow' : 'reflow')
    );
}

function classifyAndTag(plan, byName, oldMap, isSecondary) {
    const tolerancePx = plan.tolerancePx || 2.5;
    let maxDur = 0;

    for (const el of document.querySelectorAll('[data-vt-name]')) {
        if (isHidden(el)) continue;
        const name = el.dataset.vtName;
        const entry = byName.get(name);
        const old = oldMap.get(name);
        const hash = el.dataset.vtHash || '';
        const hashChanged = !!(old && old.hash && hash && old.hash !== hash);

        if (!old || hashChanged || geometryChanged(old, el, tolerancePx)) {
            if (entry) {
                if (entry.fadeInMs) maxDur = Math.max(maxDur, entry.fadeInMs);
                if (entry.fadeOutMs) maxDur = Math.max(maxDur, entry.fadeOutMs);
            }
            const isBlockIn = !!(entry && entry.timing && entry.timing.inDelay && (!entry.tokens || entry.tokens.length === 0));
            patchToFade(el, isBlockIn);
        } else {
            patchToReflow(el);
        }
    }

    // Stretching is opt-in (consts::STRETCH_SECONDARY_TO_AUTHORED_FADE). Otherwise the
    // property is never written and every consumer falls back to --secondary-reflow-duration,
    // so the slide and the cross-fades share the reflow duration.
    if (isSecondary && plan.stretchSecondary && maxDur > 0) {
        const effectiveDur = Math.max(plan.secondaryReflowMs || 1500, maxDur);
        document.documentElement.style.setProperty('--secondary-duration', effectiveDur + 'ms');
    }
}

// Size and horizontal shifts mean the element no longer occupies its old box; pure
// vertical movement is the normal case when content above it grows or shrinks.
function geometryChanged(old, el, tolerancePx) {
    const newRect = el.getBoundingClientRect();
    const dw = Math.abs(old.rect.width - newRect.width);
    const dh = Math.abs(old.rect.height - newRect.height);
    const dx = Math.abs(old.rect.left - newRect.left);
    return dw > tolerancePx || dh > tolerancePx || dx > tolerancePx;
}
