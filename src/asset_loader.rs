use std::collections::HashMap;

use macroquad::{
    experimental::coroutines::start_coroutine,
    texture::{load_texture, Texture2D},
    audio::load_sound,
    window::next_frame,
};

/// Load multiple textures in parallel using coroutines
pub async fn load_textures_parallel(paths: Vec<String>) -> HashMap<String, Texture2D> {
    // Spawn coroutines for each texture load
    let mut loaders = Vec::new();
    for path in paths.clone() {
        let handle =
            start_coroutine(async move { (path.clone(), load_texture(&path).await.unwrap()) });
        loaders.push(handle);
    }

    // Wait for all coroutines to complete
    loop {
        let all_done = loaders.iter().all(|h| h.is_done());
        if all_done {
            break;
        }
        next_frame().await;
    }

    // Collect results into HashMap
    loaders.into_iter().map(|h| h.retrieve().unwrap()).collect()
}

/// Load multiple audio files in parallel using coroutines
pub async fn load_audio_parallel(paths: Vec<String>) -> HashMap<String, macroquad::audio::Sound> {
    // Spawn coroutines for each sound load
    let mut loaders = Vec::new();
    for path in paths.clone() {
        let handle =
            start_coroutine(async move { (path.clone(), load_sound(&path).await.unwrap()) });
        loaders.push(handle);
    }

    // Wait for all coroutines to complete
    loop {
        let all_done = loaders.iter().all(|h| h.is_done());
        if all_done {
            break;
        }
        next_frame().await;
    }

    // Collect results into HashMap
    loaders.into_iter().map(|h| h.retrieve().unwrap()).collect()
}
