fn main() {
    // Only run tauri_build when the feature is enabled to avoid invoking
    // tauri build steps during lightweight runs. Use `--features with_tauri`
    // to enable resource compilation.
    #[cfg(feature = "with_tauri")]
    {
        tauri_build::build();
    }
}
