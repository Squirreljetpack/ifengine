use std::sync::{Arc, RwLock};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::storage::{create_save, load_saves};
use ifengine::core::key::IntoPageKey;

/// Resolves a page name or substring against all registered story pages in compile-time inventory.
pub fn find_registered_page(target: &str) -> Option<ifengine::core::PageHandle> {
    let clean = target
        .trim_start_matches('#')
        .trim_start_matches('/')
        .trim();
    if clean.is_empty() {
        return None;
    }

    // 1. Exact match
    for page in ifengine::inventory::iter::<ifengine::core::RegisteredPage> {
        if page.id.eq_ignore_ascii_case(clean) {
            return Some((page.factory)(page.id.into()));
        }
    }

    // 2. Suffix match (e.g. "p_first_place_between" matches "chant::act1::p_first_place_between")
    for page in ifengine::inventory::iter::<ifengine::core::RegisteredPage> {
        if page.id.ends_with(clean) || page.id.split("::").last() == Some(clean) {
            return Some((page.factory)(page.id.into()));
        }
    }

    // 3. Substring match
    let clean_lower = clean.to_lowercase();
    for page in ifengine::inventory::iter::<ifengine::core::RegisteredPage> {
        if page.id.to_lowercase().contains(&clean_lower) {
            return Some((page.factory)(page.id.into()));
        }
    }

    None
}

/// Reads the current URL hash, stripped of '#' and leading whitespace.
pub fn get_url_hash() -> Option<String> {
    let window = web_sys::window()?;
    let location = window.location();
    let hash = location.hash().ok()?;
    let trimmed = hash.trim_start_matches('#').trim_start_matches('/').trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Mounts the developer console bridge on `window.__ifengine` and `window.ifengine`.
pub fn mount_dev_bridge<C>(
    game_lock: Arc<RwLock<ifengine::Game<C>>>,
    refresh_view: Arc<dyn Fn(&mut ifengine::Game<C>) + Send + Sync>,
) where
    C: ifengine::core::GameContext
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + Send
        + Sync
        + 'static,
{
    let Some(window) = web_sys::window() else {
        return;
    };

    let dev_obj = js_sys::Object::new();

    // 1. goto(target: String)
    {
        let game_lock = Arc::clone(&game_lock);
        let refresh = Arc::clone(&refresh_view);
        let cb = Closure::wrap(Box::new(move |target: String| {
            if let Some(handle) = find_registered_page(&target) {
                let id = handle.id.clone();
                let mut game = game_lock.write().unwrap();
                if let Err(err) = game.handle_action(ifengine::Action::Next(handle)) {
                    web_sys::console::error_1(
                        &format!("[ifengine] Failed to navigate to '{id}': {err:?}").into(),
                    );
                    return;
                }
                refresh(&mut game);
                web_sys::console::log_1(&format!("[ifengine] Navigated to '{id}'").into());
            } else {
                web_sys::console::warn_1(
                    &format!(
                        "[ifengine] Page '{target}' not found. Run __ifengine.pages() to see all registered pages."
                    )
                    .into(),
                );
            }
        }) as Box<dyn Fn(String)>);
        let _ = js_sys::Reflect::set(&dev_obj, &"goto".into(), cb.as_ref().unchecked_ref());
        cb.forget();
    }

    // 2. pages() -> Array
    {
        let cb = Closure::wrap(Box::new(move || {
            let arr = js_sys::Array::new();
            for page in ifengine::inventory::iter::<ifengine::core::RegisteredPage> {
                arr.push(&page.id.into());
            }
            web_sys::console::log_1(&arr);
            arr
        }) as Box<dyn Fn() -> js_sys::Array>);
        let _ = js_sys::Reflect::set(&dev_obj, &"pages".into(), cb.as_ref().unchecked_ref());
        cb.forget();
    }

    // 3. page() -> String
    {
        let game_lock = Arc::clone(&game_lock);
        let cb = Closure::wrap(Box::new(move || {
            let game = game_lock.read().unwrap();
            let id = game.pageid().to_string();
            web_sys::console::log_1(&format!("[ifengine] Current page: {id}").into());
            id
        }) as Box<dyn Fn() -> String>);
        let _ = js_sys::Reflect::set(&dev_obj, &"page".into(), cb.as_ref().unchecked_ref());
        cb.forget();
    }

    // 4. setBit(key: String, bit: u8)
    {
        let game_lock = Arc::clone(&game_lock);
        let refresh = Arc::clone(&refresh_view);
        let cb = Closure::wrap(Box::new(move |key: String, bit: u8| {
            let mut game = game_lock.write().unwrap();
            let page_id = game.pageid().clone();
            let page_key = key
                .parse::<u64>()
                .unwrap_or_else(|_| key.as_str().into_page_key());
            game.handle_choice(page_key, bit);
            refresh(&mut game);
            web_sys::console::log_1(
                &format!("[ifengine] Set bit for key '{key}' to {bit} on '{page_id}'").into(),
            );
        }) as Box<dyn Fn(String, u8)>);
        let _ = js_sys::Reflect::set(&dev_obj, &"setBit".into(), cb.as_ref().unchecked_ref());
        cb.forget();
    }

    // 5. exportJSON() -> String
    {
        let game_lock = Arc::clone(&game_lock);
        let cb = Closure::wrap(Box::new(move || {
            match serde_json::to_string_pretty(&*game_lock.read().unwrap()) {
                Ok(json) => {
                    web_sys::console::log_1(&json.clone().into());
                    json
                }
                Err(err) => {
                    let msg = format!("[ifengine] Error exporting JSON: {err}");
                    web_sys::console::error_1(&msg.clone().into());
                    msg
                }
            }
        }) as Box<dyn Fn() -> String>);
        let _ = js_sys::Reflect::set(&dev_obj, &"exportJSON".into(), cb.as_ref().unchecked_ref());
        cb.forget();
    }

    // 6. importJSON(json_str: String) -> bool
    {
        let game_lock = Arc::clone(&game_lock);
        let refresh = Arc::clone(&refresh_view);
        let cb = Closure::wrap(Box::new(move |json_str: String| {
            match serde_json::from_str::<ifengine::Game<C>>(&json_str) {
                Ok(loaded) => {
                    let mut game = game_lock.write().unwrap();
                    *game = loaded;
                    refresh(&mut game);
                    web_sys::console::log_1(
                        &"[ifengine] Successfully imported state from JSON!".into(),
                    );
                    true
                }
                Err(err) => {
                    web_sys::console::error_1(
                        &format!("[ifengine] Failed to parse JSON game state: {err}").into(),
                    );
                    false
                }
            }
        }) as Box<dyn Fn(String) -> bool>);
        let _ = js_sys::Reflect::set(&dev_obj, &"importJSON".into(), cb.as_ref().unchecked_ref());
        cb.forget();
    }

    // 7. save(name: Option<String>)
    {
        let game_lock = Arc::clone(&game_lock);
        let cb = Closure::wrap(Box::new(move |name: Option<String>| {
            let game = game_lock.read().unwrap();
            let slot = create_save(&game, name);
            web_sys::console::log_1(
                &format!(
                    "[ifengine] Saved state to slot '{}' ({})",
                    slot.name, slot.id
                )
                .into(),
            );
        }) as Box<dyn Fn(Option<String>)>);
        let _ = js_sys::Reflect::set(&dev_obj, &"save".into(), cb.as_ref().unchecked_ref());
        cb.forget();
    }

    // 8. load(query: String)
    {
        let game_lock = Arc::clone(&game_lock);
        let refresh = Arc::clone(&refresh_view);
        let cb = Closure::wrap(Box::new(move |query: String| {
            let saves = load_saves::<C>();
            if let Some(slot) = saves.iter().find(|s| s.id == query || s.name == query) {
                let mut game = game_lock.write().unwrap();
                *game = slot.game.clone();
                refresh(&mut game);
                web_sys::console::log_1(
                    &format!("[ifengine] Loaded save slot '{}' ({})", slot.name, slot.id).into(),
                );
            } else {
                web_sys::console::warn_1(
                    &format!("[ifengine] Save slot matching '{query}' not found.").into(),
                );
            }
        }) as Box<dyn Fn(String)>);
        let _ = js_sys::Reflect::set(&dev_obj, &"load".into(), cb.as_ref().unchecked_ref());
        cb.forget();
    }

    // 9. help()
    {
        let cb = Closure::wrap(Box::new(move || {
            web_sys::console::log_1(
                &r#"[ifengine] Developer Console API:
  __ifengine.goto("name")     - Jump to page by name or substring (e.g. "p_first_place_between")
  __ifengine.pages()          - List all registered story page IDs
  __ifengine.page()           - Print active page ID
  __ifengine.setBit("key", 1) - Set bit / toggle choice on current page
  __ifengine.exportJSON()     - Export game state as formatted JSON string
  __ifengine.importJSON(str)  - Load game state from JSON string
  __ifengine.save("my_save")  - Save game to named slot
  __ifengine.load("my_save")  - Load game from named slot
  URL Hash shortcut:          - Append #<page_name> to browser URL to boot directly to that page!"#
                    .into(),
            );
        }) as Box<dyn Fn()>);
        let _ = js_sys::Reflect::set(&dev_obj, &"help".into(), cb.as_ref().unchecked_ref());
        cb.forget();
    }

    // Bind to window.__ifengine and window.ifengine
    let _ = js_sys::Reflect::set(&window, &"__ifengine".into(), &dev_obj);
    let _ = js_sys::Reflect::set(&window, &"ifengine".into(), &dev_obj);

    // Setup hashchange listener for real-time URL hash navigation
    {
        let game_lock = Arc::clone(&game_lock);
        let refresh = Arc::clone(&refresh_view);
        let on_hash = Closure::wrap(Box::new(move || {
            if let Some(hash) = get_url_hash()
                && let Some(handle) = find_registered_page(&hash)
            {
                let mut game = game_lock.write().unwrap();
                if game.pageid() != &handle.id {
                    let _ = game.handle_action(ifengine::Action::Next(handle));
                    refresh(&mut game);
                }
            }
        }) as Box<dyn Fn()>);

        let _ =
            window.add_event_listener_with_callback("hashchange", on_hash.as_ref().unchecked_ref());
        on_hash.forget();
    }

    web_sys::console::log_1(
        &"[ifengine] Dev bridge mounted. Type `__ifengine.help()` for console tools or use URL #<page> to jump.".into(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_registered_page() {
        // Any story registered in inventory can be found
        let pages: Vec<_> = ifengine::inventory::iter::<ifengine::core::RegisteredPage>().collect();
        assert!(!pages.is_empty(), "Expected registered pages in inventory");

        let first = &pages[0];
        let full_id = first.id;
        let short_id = full_id.split("::").last().unwrap();

        // Exact match
        assert!(find_registered_page(full_id).is_some());

        // Suffix match
        assert!(find_registered_page(short_id).is_some());

        // With leading hash '#'
        let hashed = format!("#{short_id}");
        assert!(find_registered_page(&hashed).is_some());

        // Non-existent page
        assert!(find_registered_page("definitely_nonexistent_page_xyz").is_none());
    }
}
