use crate::models::Platform;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Returns the persistent local directory for cached game cover posters.
pub fn get_covers_cache_dir() -> PathBuf {
    static CACHE_DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    CACHE_DIR.get_or_init(|| {
        #[cfg(target_os = "windows")]
        let base = std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::var("USERPROFILE")
                    .map(|p| PathBuf::from(p).join("AppData").join("Local"))
                    .unwrap_or_else(|_| PathBuf::from("."))
            });

        #[cfg(not(target_os = "windows"))]
        let base = std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".cache"))
            .unwrap_or_else(|_| PathBuf::from("."));

        let dir = base.join("gank").join("covers");
        let _ = fs::create_dir_all(&dir);
        dir
    }).clone()
}

/// Sanitizes a string for use as a safe filename.
pub fn sanitize_filename(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect()
}

/// Checks if a cached cover image already exists on disk for the given ID.
pub fn get_cached_cover(id: &str) -> Option<PathBuf> {
    let safe_id = sanitize_filename(id);
    let cache_dir = get_covers_cache_dir();

    // Check for custom user-selected cover first
    if let Ok(entries) = fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                if file_name.starts_with(&format!("{}_custom", safe_id)) && !file_name.contains("_anim") {
                    if path.metadata().map(|m| m.len() > 100).unwrap_or(false) {
                        return Some(path);
                    }
                }
            }
        }
    }

    let file = cache_dir.join(format!("{}.jpg", safe_id));
    if file.exists() && file.metadata().map(|m| m.len() > 1000).unwrap_or(false) {
        Some(file)
    } else {
        None
    }
}

/// Returns the path to the directory containing animated cover frames if it exists and has multiple frames.
pub fn get_animated_cover_frames_dir(id: &str) -> Option<PathBuf> {
    let safe_id = sanitize_filename(id);
    let cache_dir = get_covers_cache_dir();
    let anim_dir = cache_dir.join(format!("{}_anim", safe_id));
    if anim_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&anim_dir) {
            let mut count = 0;
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    if ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("png") {
                        count += 1;
                        if count >= 2 {
                            return Some(anim_dir);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Checks whether an animated cover exists for the given game ID.
pub fn has_animated_cover(id: &str) -> bool {
    get_animated_cover_frames_dir(id).is_some()
}

/// Scans the cache directory once and returns a set of all game IDs that have animated cover directories.
pub fn get_all_animated_cover_ids() -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    let cache_dir = get_covers_cache_dir();
    if let Ok(entries) = fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|f| f.to_str()) {
                    if let Some(id) = name.strip_suffix("_anim") {
                        set.insert(id.to_string());
                    }
                }
            }
        }
    }
    set
}

/// Scans the cache directory once and returns an indexed map of all cached covers for fast O(1) startup lookups.
pub fn get_all_cached_covers() -> std::collections::HashMap<String, PathBuf> {
    let mut map = std::collections::HashMap::new();
    let cache_dir = get_covers_cache_dir();
    if let Ok(entries) = fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                if file_name.ends_with(".jpg") || file_name.ends_with(".png") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if stem.contains("_custom") {
                            if let Some(base_id) = stem.split("_custom").next() {
                                if path.metadata().map(|m| m.len() > 100).unwrap_or(false) {
                                    map.insert(base_id.to_string(), path);
                                }
                            }
                        } else if path.metadata().map(|m| m.len() > 1000).unwrap_or(false) {
                            map.entry(stem.to_string()).or_insert(path);
                        }
                    }
                }
            }
        }
    }
    map
}

/// Checks if a file path is a video or animation format.
pub fn is_video_or_animation_path(path: &std::path::Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(ext.as_str(), "mp4" | "webm" | "mkv" | "mov" | "gif")
}

/// Detects if raw WebP bytes correspond to an animated WebP file (VP8X chunk with animation bit set).
pub fn is_animated_webp_bytes(bytes: &[u8]) -> bool {
    if bytes.len() >= 21 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" && &bytes[12..16] == b"VP8X" {
        return (bytes[20] & 0x02) != 0;
    }
    false
}

/// Processes an input video or animated format into standardized 2:3 frames at 24fps in the game's anim cache directory.
pub fn process_animated_cover(
    id: &str,
    source_path: &std::path::Path,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let safe_id = sanitize_filename(id);
    let cache_dir = get_covers_cache_dir();
    let anim_dir = cache_dir.join(format!("{}_anim", safe_id));

    if anim_dir.exists() {
        let _ = fs::remove_dir_all(&anim_dir);
    }
    fs::create_dir_all(&anim_dir)?;

    let mut cmd = std::process::Command::new("ffmpeg");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let out_pattern = anim_dir.join("frame_%03d.jpg");
    let out_pattern_str = out_pattern.to_string_lossy().to_string();

    // Standard 2:3 aspect crop with smooth scaling to 240x360 at 18fps (supports up to 10 seconds loop)
    let filter = "crop='min(iw,ih*2/3)':'min(ih,iw*3/2)',scale=240:360:flags=lanczos,fps=18";

    let status = cmd
        .arg("-y")
        .arg("-i")
        .arg(source_path)
        .arg("-t")
        .arg("10") // Extended up to 10 seconds loop
        .arg("-vf")
        .arg(filter)
        .arg("-q:v")
        .arg("3")
        .arg(&out_pattern_str)
        .status();

    if let Ok(st) = status {
        if st.success() {
            let first_frame = anim_dir.join("frame_001.jpg");
            let custom_thumb = cache_dir.join(format!("{}_custom.jpg", safe_id));
            if first_frame.exists() {
                let _ = fs::copy(&first_frame, &custom_thumb);
                return Ok(custom_thumb);
            }
        }
    }

    Err("Falha ao processar animação de capa com ffmpeg".into())
}

/// Loads animated cover frames as compressed byte buffers from disk into memory.
/// Consumes only ~1.5 MB of RAM for a full 10-second animation instead of ~130 MB of raw bitmaps.
pub fn load_animated_cover_compressed_frames(id: &str) -> Vec<Vec<u8>> {
    let mut frames = Vec::new();
    if let Some(anim_dir) = get_animated_cover_frames_dir(id) {
        if let Ok(entries) = fs::read_dir(&anim_dir) {
            let mut paths: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .map(|ext| {
                            ext.eq_ignore_ascii_case("jpg")
                                || ext.eq_ignore_ascii_case("png")
                                || ext.eq_ignore_ascii_case("webp")
                        })
                        .unwrap_or(false)
                })
                .collect();
            paths.sort();

            for path in paths {
                if let Ok(bytes) = fs::read(&path) {
                    frames.push(bytes);
                }
            }
        }
    }
    frames
}

/// Loads animated cover frames as egui ColorImages from disk into memory.
pub fn load_animated_cover_frames(id: &str) -> Vec<eframe::egui::ColorImage> {
    let mut frames = Vec::new();
    if let Some(anim_dir) = get_animated_cover_frames_dir(id) {
        if let Ok(entries) = fs::read_dir(&anim_dir) {
            let mut paths: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("jpg") || ext.eq_ignore_ascii_case("png"))
                        .unwrap_or(false)
                })
                .collect();
            paths.sort();

            for path in paths {
                if let Ok(bytes) = fs::read(&path) {
                    if let Ok(img) = image::load_from_memory(&bytes) {
                        let rgba = img.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let pixels = rgba.into_raw();
                        let color_image = eframe::egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                        frames.push(color_image);
                    }
                }
            }
        }
    }
    frames
}

/// Target standard vertical poster dimensions (Steam / SteamGridDB standard 2:3 ratio).
pub const POSTER_WIDTH: u32 = 600;
pub const POSTER_HEIGHT: u32 = 900;

/// Processes an image buffer, applying a centered "cover" crop and resizing to POSTER_WIDTH x POSTER_HEIGHT (600x900).
pub fn process_cover_image(img: image::DynamicImage) -> image::DynamicImage {
    let (w, h) = (img.width(), img.height());
    if w == 0 || h == 0 {
        return img;
    }

    let target_aspect = POSTER_WIDTH as f64 / POSTER_HEIGHT as f64; // 2.0 / 3.0
    let current_aspect = w as f64 / h as f64;

    let (crop_x, crop_y, crop_w, crop_h) = if current_aspect > target_aspect {
        // Image is wider than 2:3: keep full height, crop horizontal sides
        let new_w = (h as f64 * target_aspect).round() as u32;
        let x = (w.saturating_sub(new_w)) / 2;
        (x, 0, new_w.min(w), h)
    } else {
        // Image is taller than 2:3: keep full width, crop top/bottom
        let new_h = (w as f64 / target_aspect).round() as u32;
        let y = (h.saturating_sub(new_h)) / 2;
        (0, y, w, new_h.min(h))
    };

    let cropped = img.crop_imm(crop_x, crop_y, crop_w, crop_h);
    cropped.resize_exact(POSTER_WIDTH, POSTER_HEIGHT, image::imageops::FilterType::Lanczos3)
}

/// Downloads and saves a cover image or video from a direct URL into the cache directory.
pub fn save_custom_cover_from_url(id: &str, url: &str) -> Option<PathBuf> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(20))
        .build();
    let resp = agent.get(url).call().ok()?;
    if resp.status() != 200 {
        return None;
    }
    let mut bytes = Vec::new();
    if std::io::copy(&mut resp.into_reader(), &mut bytes).is_err() || bytes.len() < 100 {
        return None;
    }

    let url_lower = url.to_lowercase();
    let is_video_url = url_lower.ends_with(".mp4")
        || url_lower.ends_with(".webm")
        || url_lower.ends_with(".gif")
        || url_lower.ends_with(".mov")
        || url_lower.ends_with(".mkv");
    let is_anim_bytes = is_video_url
        || is_animated_webp_bytes(&bytes)
        || bytes.starts_with(b"GIF")
        || (bytes.len() >= 8 && &bytes[4..8] == b"ftyp")
        || (bytes.len() >= 4 && &bytes[0..4] == &[0x1A, 0x45, 0xDF, 0xA3]);

    if is_anim_bytes {
        let cache_dir = get_covers_cache_dir();
        let temp_ext = if url_lower.ends_with(".mp4") {
            "mp4"
        } else if url_lower.ends_with(".webm") {
            "webm"
        } else if url_lower.ends_with(".gif") {
            "gif"
        } else {
            "webp"
        };
        let temp_file = cache_dir.join(format!("temp_anim_{}.{}", sanitize_filename(id), temp_ext));
        if fs::write(&temp_file, &bytes).is_ok() {
            let res = process_animated_cover(id, &temp_file).ok();
            let _ = fs::remove_file(temp_file);
            if let Some(thumb) = res {
                return Some(thumb);
            }
        }
    }

    save_custom_cover_from_bytes(id, &bytes).ok()
}

/// Saves raw image bytes as a custom cover for the given game ID, with centered cover cropping to 600x900.
pub fn save_custom_cover_from_bytes(id: &str, bytes: &[u8]) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let safe_id = sanitize_filename(id);
    let cache_dir = get_covers_cache_dir();

    // 1. Remove previous animated directory if any, since user is setting a static cover
    let anim_dir = cache_dir.join(format!("{}_anim", safe_id));
    if anim_dir.exists() {
        let _ = fs::remove_dir_all(&anim_dir);
    }

    // 2. Decode image from memory (supports WebP, PNG, JPG)
    let img = image::load_from_memory(bytes)?;

    // 3. Center crop and resize to 600x900 (cover mode)
    let processed = process_cover_image(img);

    // 4. Clean up older custom covers
    if let Ok(entries) = fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|f| f.to_str()) {
                if name.starts_with(&format!("{}_custom", safe_id)) && !name.contains("_anim") {
                    let _ = fs::remove_file(path);
                }
            }
        }
    }

    // 5. Save processed image as high-quality JPG
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let dest_file = cache_dir.join(format!("{}_custom_{}.jpg", safe_id, timestamp));
    processed.save(&dest_file)?;
    Ok(dest_file)
}

/// Saves a custom cover image or video selected by the user into the cache directory.
pub fn save_custom_cover(id: &str, source_path: &std::path::Path) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let safe_id = sanitize_filename(id);
    let cache_dir = get_covers_cache_dir();

    // Check if source file is a video, gif, or animated WebP
    let mut is_anim = is_video_or_animation_path(source_path);
    if !is_anim {
        if let Ok(mut file) = fs::File::open(source_path) {
            use std::io::Read;
            let mut header = [0u8; 32];
            if let Ok(n) = file.read(&mut header) {
                if n >= 21 && is_animated_webp_bytes(&header[..n]) {
                    is_anim = true;
                }
            }
        }
    }

    if is_anim {
        if let Ok(thumb) = process_animated_cover(id, source_path) {
            return Ok(thumb);
        }
    }

    // Static image path: clean up previous animated cover directory if existing
    let anim_dir = cache_dir.join(format!("{}_anim", safe_id));
    if anim_dir.exists() {
        let _ = fs::remove_dir_all(&anim_dir);
    }

    // 1. Open and decode image from file path (supports WebP, PNG, JPG)
    let img = image::open(source_path)?;

    // 2. Center crop and resize to 600x900 (cover mode)
    let processed = process_cover_image(img);

    // 3. Clean up older custom covers
    if let Ok(entries) = fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|f| f.to_str()) {
                if name.starts_with(&format!("{}_custom", safe_id)) && !name.contains("_anim") {
                    let _ = fs::remove_file(path);
                }
            }
        }
    }

    // 4. Save processed image as high-quality JPG
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let dest_file = cache_dir.join(format!("{}_custom_{}.jpg", safe_id, timestamp));
    processed.save(&dest_file)?;
    Ok(dest_file)
}

/// Opens a native file dialog allowing the user to select an image or video file (PNG, JPG, WEBP, MP4, WEBM, GIF).
#[cfg(target_os = "windows")]
pub fn pick_image_file() -> Option<PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    #[repr(C)]
    #[allow(non_snake_case)]
    struct OPENFILENAMEW {
        lStructSize: u32,
        hwndOwner: *mut std::ffi::c_void,
        hInstance: *mut std::ffi::c_void,
        lpstrFilter: *const u16,
        lpstrCustomFilter: *mut u16,
        nMaxCustFilter: u32,
        nFilterIndex: u32,
        lpstrFile: *mut u16,
        nMaxFile: u32,
        lpstrFileTitle: *mut u16,
        nMaxFileTitle: u32,
        lpstrInitialDir: *const u16,
        lpstrTitle: *const u16,
        Flags: u32,
        nFileOffset: u16,
        nFileExtension: u16,
        lpstrDefExt: *const u16,
        lCustData: usize,
        lpfnHook: *mut std::ffi::c_void,
        lpTemplateName: *const u16,
        pvReserved: *mut std::ffi::c_void,
        dwReserved: u32,
        FlagsEx: u32,
    }

    #[link(name = "comdlg32")]
    extern "system" {
        fn GetOpenFileNameW(lpofn: *mut OPENFILENAMEW) -> i32;
    }

    let mut file_buf = [0u16; 1024];

    // Filter format: "Description\0*.ext1;*.ext2\0All Files\0*.*\0\0"
    let filter: Vec<u16> = "Todas as Mídias de Capa (*.png;*.jpg;*.jpeg;*.webp;*.mp4;*.webm;*.gif)\0*.png;*.jpg;*.jpeg;*.webp;*.mp4;*.webm;*.gif\0Vídeos (*.mp4;*.webm)\0*.mp4;*.webm\0Animações WebP / GIF (*.webp;*.gif)\0*.webp;*.gif\0Imagens Estáticas (*.png;*.jpg;*.jpeg)\0*.png;*.jpg;*.jpeg\0Todos os arquivos (*.*)\0*.*\0\0"
        .encode_utf16()
        .collect();

    let title: Vec<u16> = "Selecionar Capa ou Vídeo do Jogo\0"
        .encode_utf16()
        .collect();

    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: std::ptr::null_mut(),
        hInstance: std::ptr::null_mut(),
        lpstrFilter: filter.as_ptr(),
        lpstrCustomFilter: std::ptr::null_mut(),
        nMaxCustFilter: 0,
        nFilterIndex: 1,
        lpstrFile: file_buf.as_mut_ptr(),
        nMaxFile: file_buf.len() as u32,
        lpstrFileTitle: std::ptr::null_mut(),
        nMaxFileTitle: 0,
        lpstrInitialDir: std::ptr::null(),
        lpstrTitle: title.as_ptr(),
        Flags: 0x00001000 | 0x00000800 | 0x00080000, // OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_EXPLORER
        nFileOffset: 0,
        nFileExtension: 0,
        lpstrDefExt: std::ptr::null(),
        lCustData: 0,
        lpfnHook: std::ptr::null_mut(),
        lpTemplateName: std::ptr::null_mut(),
        pvReserved: std::ptr::null_mut(),
        dwReserved: 0,
        FlagsEx: 0,
    };

    let result = unsafe { GetOpenFileNameW(&mut ofn) };
    if result != 0 {
        let len = file_buf.iter().position(|&c| c == 0).unwrap_or(file_buf.len());
        let os_str = OsString::from_wide(&file_buf[..len]);
        Some(PathBuf::from(os_str))
    } else {
        None
    }
}

#[cfg(not(target_os = "windows"))]
pub fn pick_image_file() -> Option<PathBuf> {
    None
}

/// Fetches and caches a cover image for a game using Epic Store Content CDN or Steam Community Search.
pub fn fetch_and_cache_cover(id: &str, title: &str, platform: Platform) -> Option<PathBuf> {
    // 1. Check local cache first
    if let Some(cached) = get_cached_cover(id) {
        return Some(cached);
    }

    let cache_dir = get_covers_cache_dir();
    let safe_id = sanitize_filename(id);
    let target_file = cache_dir.join(format!("{}.jpg", safe_id));

    // 2. If it's Epic Games, try Epic Store Content CDN first
    if platform == Platform::EpicGames {
        if let Some(bytes) = try_fetch_epic_cover(title) {
            if fs::write(&target_file, &bytes).is_ok() {
                return Some(target_file);
            }
        }
    }

    // 3. Fallback to Steam Community Search (matches 99% of PC games)
    if let Some(bytes) = try_fetch_steam_cover_by_search(title) {
        if fs::write(&target_file, &bytes).is_ok() {
            return Some(target_file);
        }
    }

    None
}

/// Attempts to fetch vertical cover art from Epic Games Store static content CDN.
fn try_fetch_epic_cover(title: &str) -> Option<Vec<u8>> {
    let slugs = generate_epic_slugs(title);
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(5))
        .build();

    for slug in slugs {
        let api_url = format!(
            "https://store-content-ipv4.ak.epicgames.com/api/en-US/content/products/{}",
            slug
        );

        if let Ok(resp) = agent.get(&api_url).call() {
            if resp.status() == 200 {
                if let Ok(json_val) = resp.into_json::<serde_json::Value>() {
                    // Try to find portraitBackgroundImageUrl or 1200x1600 images
                    if let Some(image_url) = extract_epic_portrait_url(&json_val) {
                        if let Ok(img_resp) = agent.get(&image_url).call() {
                            if img_resp.status() == 200 {
                                let mut bytes = Vec::new();
                                if std::io::copy(&mut img_resp.into_reader(), &mut bytes).is_ok() {
                                    if bytes.len() > 1000 {
                                        return Some(bytes);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Generates reasonable Epic store product slugs from game display titles.
fn generate_epic_slugs(title: &str) -> Vec<String> {
    let mut slugs = Vec::new();
    let lower = title.to_lowercase();

    // 1. Exact cleaned title
    let clean: String = lower
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { ' ' })
        .collect();
    let base_slug = clean.split_whitespace().collect::<Vec<_>>().join("-");
    if !base_slug.is_empty() {
        slugs.push(base_slug.clone());
    }

    // 2. Strip common editions / suffixes ("Enhanced", "Remastered", "Edition", "Special Edition")
    let stripped = lower
        .replace("enhanced", "")
        .replace("special edition", "")
        .replace("definitive edition", "")
        .replace("complete edition", "")
        .replace("remastered", "")
        .replace("standard edition", "");

    let clean_stripped: String = stripped
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { ' ' })
        .collect();
    let stripped_slug = clean_stripped.split_whitespace().collect::<Vec<_>>().join("-");
    if !stripped_slug.is_empty() && stripped_slug != base_slug {
        slugs.push(stripped_slug);
    }

    slugs
}

/// Extracts a portrait cover URL from Epic store product JSON metadata.
fn extract_epic_portrait_url(json: &serde_json::Value) -> Option<String> {
    // 1. Check pages[0].data.hero.portraitBackgroundImageUrl
    if let Some(pages) = json.get("pages").and_then(|p| p.as_array()) {
        for page in pages {
            if let Some(hero) = page.get("data").and_then(|d| d.get("hero")) {
                if let Some(url) = hero.get("portraitBackgroundImageUrl").and_then(|u| u.as_str()) {
                    if !url.is_empty() {
                        return Some(url.to_string());
                    }
                }
            }
        }
    }

    // 2. Fallback: check _images_ array for any image containing "1200x1600" or "s2"
    if let Some(pages) = json.get("pages").and_then(|p| p.as_array()) {
        for page in pages {
            if let Some(images) = page.get("_images_").and_then(|i| i.as_array()) {
                for img in images {
                    if let Some(url) = img.as_str() {
                        if url.contains("1200x1600") || url.contains("s2") {
                            return Some(url.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

/// Fallback: Searches Steam Community and downloads `library_600x900.jpg` for the top matching appid.
fn try_fetch_steam_cover_by_search(title: &str) -> Option<Vec<u8>> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(5))
        .build();

    // Clean title for search (remove special edition markers if needed)
    let clean_title = title
        .replace("Enhanced", "")
        .replace("Standard Edition", "")
        .trim()
        .to_string();

    let search_url = format!(
        "https://steamcommunity.com/actions/SearchApps/{}",
        urlencoding(&clean_title)
    );

    let resp = agent.get(&search_url).call().ok()?;
    if resp.status() != 200 {
        return None;
    }

    let search_results = resp.into_json::<serde_json::Value>().ok()?;
    let items = search_results.as_array()?;
    let first_item = items.first()?;
    let appid = first_item.get("appid")?.as_str()?;

    // Modern Steam vertical poster URL
    let poster_url = format!(
        "https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/{}/library_600x900.jpg",
        appid
    );

    let img_resp = agent.get(&poster_url).call().ok()?;
    if img_resp.status() != 200 {
        return None;
    }

    let mut bytes = Vec::new();
    if std::io::copy(&mut img_resp.into_reader(), &mut bytes).is_ok() && bytes.len() > 1000 {
        Some(bytes)
    } else {
        None
    }
}

/// Simple percent-encoding for query params.
fn urlencoding(input: &str) -> String {
    let mut out = String::new();
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_cover_image_cover_crop() {
        // Landscape image (1920x1080)
        let landscape = image::DynamicImage::new_rgb8(1920, 1080);
        let processed_landscape = process_cover_image(landscape);
        assert_eq!(processed_landscape.width(), 600);
        assert_eq!(processed_landscape.height(), 900);

        // Portrait image (1000x2000)
        let portrait = image::DynamicImage::new_rgb8(1000, 2000);
        let processed_portrait = process_cover_image(portrait);
        assert_eq!(processed_portrait.width(), 600);
        assert_eq!(processed_portrait.height(), 900);

        // Square image (1000x1000)
        let square = image::DynamicImage::new_rgb8(1000, 1000);
        let processed_square = process_cover_image(square);
        assert_eq!(processed_square.width(), 600);
        assert_eq!(processed_square.height(), 900);
    }

    #[test]
    fn test_fetch_epic_chivalry() {
        let path = fetch_and_cache_cover("Peppermint", "Chivalry 2", Platform::EpicGames);
        assert!(path.is_some(), "Chivalry 2 cover should be fetched and cached");
        let p = path.unwrap();
        assert!(p.exists());
        assert!(p.metadata().unwrap().len() > 1000);
    }

    #[test]
    fn test_fetch_epic_gtav() {
        let path = fetch_and_cache_cover("8769e24080ea413b8ebca3f1b8c50951", "Grand Theft Auto V Enhanced", Platform::EpicGames);
        assert!(path.is_some(), "GTA V cover should be fetched and cached");
        let p = path.unwrap();
        assert!(p.exists());
        assert!(p.metadata().unwrap().len() > 1000);
    }

    #[test]
    fn test_video_and_animation_detection() {
        use std::path::Path;

        assert!(is_video_or_animation_path(Path::new("video.mp4")));
        assert!(is_video_or_animation_path(Path::new("movie.webm")));
        assert!(is_video_or_animation_path(Path::new("anim.gif")));
        assert!(is_video_or_animation_path(Path::new("clip.mkv")));
        assert!(!is_video_or_animation_path(Path::new("image.jpg")));
        assert!(!is_video_or_animation_path(Path::new("photo.png")));

        // Simulated WebP header with VP8X animation flag set (bit 1 of byte 20)
        let mut webp_anim = vec![0u8; 32];
        webp_anim[0..4].copy_from_slice(b"RIFF");
        webp_anim[8..12].copy_from_slice(b"WEBP");
        webp_anim[12..16].copy_from_slice(b"VP8X");
        webp_anim[20] = 0x02; // animation flag
        assert!(is_animated_webp_bytes(&webp_anim));

        let mut webp_static = vec![0u8; 32];
        webp_static[0..4].copy_from_slice(b"RIFF");
        webp_static[8..12].copy_from_slice(b"WEBP");
        webp_static[12..16].copy_from_slice(b"VP8X");
        webp_static[20] = 0x00; // static
        assert!(!is_animated_webp_bytes(&webp_static));
    }
}

