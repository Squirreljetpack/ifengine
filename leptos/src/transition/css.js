// CSS clone backend for ifengine_leptos.
//
// The update is applied immediately; old rendering comes from a DOM clone snapshot
// placed in a fixed overlay, not from prediction. Per item:
//   removed                      -> clone copy fades out at old geometry
//   changed / new / moved inline -> clone copy (if any) fades out; live node fades in
//   moved image                  -> live node transform-FLIPs; clone copy stays hidden
//   unchanged + unmoved          -> nothing (clone content is hidden by default)
//   spacers                      -> never animated
// Inline spans fade rather than slide: `transform` is inert on `display: inline`.
const DEFAULT_TOLERANCE_PX = 2.5;
const CONTAINER_SELECTOR = '.passage-paragraph, .choice-item, .passage-image-wrapper, .passage-heading, .passage-break, .passage-empty, .passage-note';

let generation = 0;
let overlay = null;
let cleanupEls = [];
let timers = [];

export function run_css_transition(planJson, isSecondary, updateFn, onFinished) {
    let plan;
    try {
        plan = JSON.parse(planJson);
    } catch (err) {
        console.warn('ifengine: unreadable transition plan', err);
        updateFn();
        onFinished();
        return;
    }
    const byName = new Map();
    for (const item of plan.items || []) {
        if (item.name) byName.set(item.name, item);
    }

    cleanupEffects();
    const gen = ++generation;

    if (!isSecondary) {
        if (plan.primaryInDelay != null) {
            document.documentElement.style.setProperty('--primary-in-delay', plan.primaryInDelay + 'ms');
        }
        if (plan.primaryBlockInDelay != null) {
            document.documentElement.style.setProperty('--primary-block-in-delay', plan.primaryBlockInDelay + 'ms');
        }
    }

    const oldMap = snapshotNamed();
    overlay = buildOverlay();
    const cloneByName = new Map();
    if (overlay) {
        for (const el of overlay.querySelectorAll('[data-vt-name]')) {
            cloneByName.set(el.dataset.vtName, el);
        }
    }

    // Swap immediately: focus, selection, typing and click latency behave exactly as
    // with no transition. The authored sequence is preserved by delaying the fade-in.
    updateFn();

    // Tag after DOM reconciliation via microtasks so the effects are applied
    // before the browser paints the next frame.
    const runTagging = () => {
        if (gen !== generation) return;
        const result = tagAfterUpdate(plan, byName, oldMap, cloneByName, isSecondary);
        const maxAuthored = result.maxAuthored;

        if (isSecondary && plan.stretchSecondary && maxAuthored > 0) {
            const effectiveDur = Math.max(plan.secondaryReflowMs || 1500, maxAuthored);
            document.documentElement.style.setProperty('--secondary-duration', effectiveDur + 'ms');
        }

        if (!result.anyAnimating) {
            cleanupEffects();
            onFinished();
            return;
        }

        const primaryInDelay = plan.primaryInDelay != null ? plan.primaryInDelay : cssMs('--css-fade-in-delay', 400);
        const enterDelay = result.hasBlockIn
            ? (plan.primaryBlockInDelay != null ? plan.primaryBlockInDelay : cssMs('--primary-block-in-delay', 900))
            : primaryInDelay;
        const crossfadeMs = result.hasCrossfades
            ? cssMs('--css-crossfade-delay', 200) + cssMs('--css-crossfade-duration', 500)
            : 0;
        const totalMs = isSecondary
            ? Math.max(plan.secondaryReflowMs || 1300, maxAuthored)
            : Math.max(
                result.hasEnters ? enterDelay + cssMs('--css-fade-in', 400) : 0,
                result.hasLeaves ? cssMs('--css-fade-out', 300) : 0,
                result.hasSlides ? (cssMs('--css-reflow-delay', 200) + cssMs('--css-reflow', 500)) : 0,
                crossfadeMs,
                maxAuthored,
            );

        timers.push(setTimeout(() => {
            if (gen !== generation) return;
            cleanupEffects();
            onFinished();
        }, totalMs + 30));
    };

    queueMicrotask(() => {
        queueMicrotask(runTagging);
    });
}

/// Tags entering/leaving elements for a post-update DOM; returns classification metrics.
function tagAfterUpdate(plan, byName, oldMap, cloneByName, isSecondary) {
    const actions = planActions(plan, byName, oldMap, cloneByName, isSecondary);
    if (actions.anyAnimating) {
        applyActions(actions, isSecondary);
    }
    return actions;
}

/// Phase 1: Pure reads and action classification (no DOM mutations).
function planActions(plan, byName, oldMap, cloneByName, isSecondary) {
    const tolerancePx = plan.tolerancePx || DEFAULT_TOLERANCE_PX;
    const slides = new Map(); // targetElement -> { dx, dy, duration }
    const enters = [];
    const crossfades = [];
    const leaves = [];
    let maxAuthored = 0;

    const liveElements = liveNamed().filter(el => !isHidden(el));
    const liveNames = new Set(liveElements.map(el => el.dataset.vtName));

    for (const el of liveElements) {
        const name = el.dataset.vtName;
        const entry = byName.get(name);
        const kind = el.dataset.vtKind || (entry && entry.kind) || 'span';
        if (kind === 'spacer') continue;

        const old = oldMap.get(name);
        const hash = el.dataset.vtHash || '';
        const hashChanged = !!(old && old.hash && hash && old.hash !== hash);
        const isFade = !old || hashChanged || geometryChanged(old, el, tolerancePx);

        if (isFade) {
            // Case 1: Brand-new entering or content-changed element (Fade-In)
            const cloneEl = cloneByName.get(name) || null;
            const fadeInMs = entry?.fadeInMs || 0;
            const fadeOutMs = entry?.fadeOutMs || 0;
            let inDelay = null;
            let isBlockIn = false;

            if (isSecondary) {
                inDelay = 0;
            } else if (entry?.tokens && entry.tokens.some(t => t.startsWith('in'))) {
                inDelay = entry.timing?.inDelay ?? null;
                const enterMs = (inDelay || 0) + (fadeInMs || cssMs('--css-fade-in', 400));
                maxAuthored = Math.max(maxAuthored, enterMs);
            } else if (entry?.isNewBlock) {
                isBlockIn = true;
            }
            if (fadeInMs) maxAuthored = Math.max(maxAuthored, fadeInMs);
            if (fadeOutMs) maxAuthored = Math.max(maxAuthored, fadeOutMs);

            enters.push({
                el,
                cloneEl,
                fadeInMs,
                fadeOutMs,
                inDelay,
                isBlockIn,
            });
            continue;
        }

        // Case 2: Already-seen element (Reflow)
        if (old) {
            const dy = old.rect.top - el.getBoundingClientRect().top;
            const dx = old.rect.left - el.getBoundingClientRect().left;
            const moved = Math.abs(dy) > tolerancePx || Math.abs(dx) > tolerancePx;

            if (moved) {
                // Case 2A: Moved reflow -> FLIP Slide the block container
                const target = el.closest(CONTAINER_SELECTOR) || el;
                const existing = slides.get(target);
                const fadeInMs = entry?.fadeInMs || 0;
                if (!existing) {
                    slides.set(target, { dx, dy, duration: fadeInMs });
                } else if (!existing.duration && fadeInMs) {
                    existing.duration = fadeInMs;
                }
                if (fadeInMs) maxAuthored = Math.max(maxAuthored, cssMs('--css-reflow-delay', 200) + fadeInMs);
            } else if (!isSecondary) {
                // Case 2B: Stationary reflow -> Cross-fade in place if tag or semantic styling changed
                const styleChanged = el.tagName !== old.tag || cleanClasses(el.className) !== cleanClasses(old.cls);
                if (styleChanged) {
                    const cloneEl = cloneByName.get(name) || null;
                    const fadeInMs = entry?.fadeInMs || 0;
                    const fadeOutMs = entry?.fadeOutMs || 0;
                    const crossfadeDelay = cssMs('--css-crossfade-delay', 200);
                    if (fadeInMs) maxAuthored = Math.max(maxAuthored, crossfadeDelay + fadeInMs);
                    if (fadeOutMs) maxAuthored = Math.max(maxAuthored, crossfadeDelay + fadeOutMs);

                    crossfades.push({
                        el,
                        cloneEl,
                        fadeInMs,
                        fadeOutMs,
                    });
                }
            }
        }
    }

    // Case 3: Old counterparts without a live match fade out (Phase 1 / Exit)
    for (const cloneEl of cloneByName.values()) {
        const name = cloneEl.dataset.vtName;
        if (liveNames.has(name)) continue;
        const kind = cloneEl.dataset.vtKind || 'span';
        if (kind === 'spacer') continue;
        const entry = byName.get(name);
        const fadeOutMs = entry?.fadeOutMs || 0;
        if (fadeOutMs) maxAuthored = Math.max(maxAuthored, fadeOutMs);

        leaves.push({
            cloneEl,
            fadeOutMs,
        });
    }

    return {
        slides,
        enters,
        crossfades,
        leaves,
        maxAuthored,
        hasEnters: enters.length > 0,
        hasLeaves: leaves.length > 0,
        hasSlides: slides.size > 0,
        hasCrossfades: crossfades.length > 0,
        hasBlockIn: enters.some(e => e.isBlockIn),
        anyAnimating: enters.length > 0 || leaves.length > 0 || slides.size > 0 || crossfades.length > 0,
    };
}

/// Phase 2: Pure writes (batch DOM mutations and animations).
function applyActions(actions, isSecondary) {
    // 1. FLIP Slide block containers
    for (const [target, s] of actions.slides) {
        slideElement(target, s.dx, s.dy);
        if (s.duration) {
            target.style.setProperty('--vt-reflow-duration', s.duration + 'ms');
        }
    }

    // 2. Entering elements
    for (const item of actions.enters) {
        item.el.classList.add('vt-enter');
        track(item.el);
        if (item.cloneEl) {
            item.cloneEl.classList.add('vt-leave');
            track(item.cloneEl);
            if (item.fadeOutMs) {
                item.cloneEl.style.setProperty('--vt-fade-out', item.fadeOutMs + 'ms');
            }
        }
        if (item.fadeInMs) {
            item.el.style.setProperty('--vt-fade-in', item.fadeInMs + 'ms');
        }
        if (item.inDelay != null) {
            item.el.style.setProperty('--vt-in-delay', item.inDelay + 'ms');
        }
        if (item.isBlockIn) {
            item.el.classList.add('block-in');
        }
    }

    // 3. Stationary crossfades
    for (const item of actions.crossfades) {
        item.el.classList.add('vt-crossfade-in');
        track(item.el);
        if (item.cloneEl) {
            item.cloneEl.classList.add('vt-crossfade-out');
            track(item.cloneEl);
            if (item.fadeOutMs) {
                item.cloneEl.style.setProperty('--vt-reflow-duration', item.fadeOutMs + 'ms');
            }
        }
        if (item.fadeInMs) {
            item.el.style.setProperty('--vt-reflow-duration', item.fadeInMs + 'ms');
        }
    }

    // 4. Leaving elements
    for (const item of actions.leaves) {
        item.cloneEl.classList.add('vt-leave');
        track(item.cloneEl);
        if (item.fadeOutMs) {
            item.cloneEl.style.setProperty('--vt-fade-out', item.fadeOutMs + 'ms');
        }
    }
}

export function reset_css_transition() {
    generation++;
    cleanupEffects();
}

function liveNamed() {
    return Array.from(document.querySelectorAll('[data-vt-name]'))
        .filter(el => !el.closest('.vt-clone'));
}

function liveNamedByName() {
    const names = new Set();
    for (const el of liveNamed()) {
        names.add(el.dataset.vtName);
    }
    return names;
}

function snapshotNamed() {
    const map = new Map();
    for (const el of liveNamed()) {
        if (isHidden(el)) continue;
        const name = el.dataset.vtName;
        map.set(name, {
            hash: el.dataset.vtHash || '',
            rect: el.getBoundingClientRect(),
            tag: el.tagName,
            cls: el.className,
        });
    }
    return map;
}

function cleanClasses(cls) {
    if (!cls) return '';
    return cls
        .split(/\s+/)
        .filter(c => c && !c.startsWith('vt-') && !c.startsWith('transition-') && c !== 'delayed-hidden' && c !== 'fade-out-removed' && c !== 'block-in')
        .sort()
        .join(' ');
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

function isHidden(el) {
    return !el.dataset.vtName
        || el.closest('.delayed-hidden')
        || el.classList.contains('delayed-hidden')
        || el.classList.contains('fade-out-removed')
        || el.classList.contains('transition-removed');
}

function buildOverlay() {
    const root = document.getElementById('backdrop');
    if (!root) return null;
    const rect = root.getBoundingClientRect();
    const next = document.createElement('div');
    next.className = 'vt-clone-overlay';
    next.setAttribute('aria-hidden', 'true');

    const clone = root.cloneNode(true);
    clone.classList.add('vt-clone');
    clone.setAttribute('inert', '');
    clone.setAttribute('aria-hidden', 'true');
    next.appendChild(clone);

    next.style.left = rect.left + 'px';
    next.style.top = rect.top + 'px';
    next.style.width = rect.width + 'px';
    next.style.height = rect.height + 'px';
    document.body.appendChild(next);
    return next;
}

function slideElement(el, dx, dy) {
    if (Math.abs(dx) < 1 && Math.abs(dy) < 1) return;
    if (el.classList.contains('vt-slide')) return;

    el.style.transition = 'none';
    el.style.transform = `translate(${dx}px, ${dy}px)`;
    void el.offsetWidth; // commit the offset position before animating
    el.classList.add('vt-slide');
    el.style.transition = '';
    el.style.transform = 'translate(0px, 0px)';
    track(el);
}

/// Remembers an element for effect cleanup; also used for the clone copies.
function track(el) {
    if (!cleanupEls.includes(el)) cleanupEls.push(el);
}

function cleanupEffects() {
    for (const timer of timers) clearTimeout(timer);
    timers = [];

    for (const el of cleanupEls) {
        el.classList.remove('vt-enter', 'vt-slide', 'vt-leave', 'vt-crossfade-in', 'vt-crossfade-out', 'block-in');
        el.style.removeProperty('--vt-fade-in');
        el.style.removeProperty('--vt-in-delay');
        el.style.removeProperty('--vt-fade-out');
        el.style.removeProperty('--vt-reflow-duration');
        el.style.removeProperty('--vt-reflow-delay');
        el.style.removeProperty('transform');
        el.style.removeProperty('transition');
    }
    cleanupEls = [];

    if (overlay) {
        overlay.remove();
        overlay = null;
    }
    document.documentElement.style.removeProperty('--secondary-duration');
    document.documentElement.style.removeProperty('--primary-in-delay');
    document.documentElement.style.removeProperty('--primary-block-in-delay');
}

function cssMs(name, fallback) {
    const raw = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    if (raw.endsWith('ms')) return parseFloat(raw) || fallback;
    if (raw.endsWith('s')) return (parseFloat(raw) || fallback / 1000) * 1000;
    return fallback;
}
