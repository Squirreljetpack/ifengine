use base64::prelude::*;
use serde::{Deserialize, Serialize};

pub const AUTOSAVE_KEY: &str = "ifengine_autosave";
pub const SAVES_KEY: &str = "ifengine_saves";

/// Representation of a stored save game slot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveSlot<C> {
    pub id: String,
    pub name: String,
    pub date: String,
    pub page_id: String,
    pub game: ifengine::Game<C>,
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Formats the current local date/time in the browser.
pub fn current_timestamp() -> String {
    let date = js_sys::Date::new_0();
    date.to_locale_string("en-US", &wasm_bindgen::JsValue::UNDEFINED)
        .as_string()
        .unwrap_or_else(|| "Unknown date".into())
}

/// Loads the autosave slot from `localStorage` if it exists.
pub fn load_autosave<C>() -> Option<SaveSlot<C>>
where
    C: ifengine::core::GameContext + for<'de> Deserialize<'de>,
{
    let storage = local_storage()?;
    let encoded = storage.get_item(AUTOSAVE_KEY).ok()??;
    let bytes = BASE64_STANDARD.decode(encoded).ok()?;
    postcard::from_bytes(&bytes).ok()
}

/// Writes the current game state to the autosave slot in `localStorage`.
pub fn save_autosave<C>(game: &ifengine::Game<C>)
where
    C: ifengine::core::GameContext + Serialize,
{
    let Some(storage) = local_storage() else {
        return;
    };
    let slot = SaveSlot {
        id: "autosave".to_string(),
        name: "Autosave".to_string(),
        date: current_timestamp(),
        page_id: game.last_page_id().to_string(),
        game: game.clone(),
    };
    if let Ok(bytes) = postcard::to_allocvec(&slot) {
        let encoded = BASE64_STANDARD.encode(&bytes);
        let _ = storage.set_item(AUTOSAVE_KEY, &encoded);
    }
}

/// Loads all manual saves from `localStorage`.
pub fn load_saves<C>() -> Vec<SaveSlot<C>>
where
    C: ifengine::core::GameContext + for<'de> Deserialize<'de>,
{
    let Some(storage) = local_storage() else {
        return Vec::new();
    };
    let Some(encoded) = storage.get_item(SAVES_KEY).ok().flatten() else {
        return Vec::new();
    };
    let Ok(bytes) = BASE64_STANDARD.decode(encoded) else {
        return Vec::new();
    };
    postcard::from_bytes::<Vec<SaveSlot<C>>>(&bytes).unwrap_or_default()
}

/// Writes all manual saves to `localStorage`.
pub fn write_saves<C>(saves: &[SaveSlot<C>])
where
    C: ifengine::core::GameContext + Serialize,
{
    let Some(storage) = local_storage() else {
        return;
    };
    if let Ok(bytes) = postcard::to_allocvec(saves) {
        let encoded = BASE64_STANDARD.encode(&bytes);
        let _ = storage.set_item(SAVES_KEY, &encoded);
    }
}

/// Creates a new autonamed save slot with current game state.
pub fn create_save<C>(game: &ifengine::Game<C>, custom_name: Option<String>) -> SaveSlot<C>
where
    C: ifengine::core::GameContext + Serialize + for<'de> Deserialize<'de>,
{
    let mut saves = load_saves::<C>();
    let id = format!("{}_{}", js_sys::Date::now() as u64, saves.len() + 1);
    let name = custom_name.unwrap_or_else(|| format!("Save {}", saves.len() + 1));
    let slot = SaveSlot {
        id,
        name,
        date: current_timestamp(),
        page_id: game.last_page_id().to_string(),
        game: game.clone(),
    };
    saves.push(slot.clone());
    write_saves(&saves);
    slot
}

/// Deletes a save slot by its ID.
pub fn delete_save<C>(id: &str)
where
    C: ifengine::core::GameContext + Serialize + for<'de> Deserialize<'de>,
{
    let mut saves = load_saves::<C>();
    saves.retain(|s| s.id != id);
    write_saves(&saves);
}

/// Renames a save slot by its ID.
pub fn rename_save<C>(id: &str, new_name: &str)
where
    C: ifengine::core::GameContext + Serialize + for<'de> Deserialize<'de>,
{
    let mut saves = load_saves::<C>();
    if let Some(slot) = saves.iter_mut().find(|s| s.id == id) {
        slot.name = new_name.to_string();
        write_saves(&saves);
    }
}

/// Overwrites an existing save slot with current game state.
pub fn overwrite_save<C>(id: &str, game: &ifengine::Game<C>)
where
    C: ifengine::core::GameContext + Serialize + for<'de> Deserialize<'de>,
{
    let mut saves = load_saves::<C>();
    if let Some(slot) = saves.iter_mut().find(|s| s.id == id) {
        slot.date = current_timestamp();
        slot.page_id = game.last_page_id().to_string();
        slot.game = game.clone();
        write_saves(&saves);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_slot_postcard_serde() {
        let mut game = story::new();
        let _ = game.view().expect("view should render");

        let slot = SaveSlot {
            id: "save_1".into(),
            name: "My Save".into(),
            date: "2026-09-09".into(),
            page_id: game.last_page_id().to_string(),
            game,
        };

        let bytes = postcard::to_allocvec(&slot).expect("serialize should succeed");
        let encoded = BASE64_STANDARD.encode(&bytes);

        let decoded = BASE64_STANDARD.decode(&encoded).expect("base64 decode");
        let mut loaded: SaveSlot<story::State> =
            postcard::from_bytes(&decoded).expect("deserialize");

        assert_eq!(loaded.id, "save_1");
        assert_eq!(loaded.name, "My Save");

        let view = loaded
            .game
            .view()
            .expect("view should render from loaded save");
        assert!(view.pageid.0.ends_with("rainy_day") || view.pageid.0.contains("saltwrack"));
    }
}
