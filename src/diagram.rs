use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::mpsc::{self, Receiver, SyncSender, TrySendError},
};

use image::DynamicImage;
use merman::render::{
    HeadlessRenderer,
    raster::{RasterFitBox, RasterOptions},
};
use ratatui::layout::Size;
use ratatui_image::{
    Resize,
    picker::{Picker, ProtocolType},
    sliced::{SignedPosition, SlicedImage, SlicedProtocol},
};

use crate::markdown::{Block, Document};

pub const MAX_CACHE_BYTES: usize = 32 * 1024 * 1024;
pub const DISPLAY_HEIGHT: u16 = 24;
const RASTER_SCALE: f32 = 2.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisplayMode {
    #[default]
    Rendered,
    Source,
}

pub fn is_mermaid(language: Option<&str>) -> bool {
    language.is_some_and(|language| language.trim().eq_ignore_ascii_case("mermaid"))
}

pub fn is_supported_source(source: &str) -> bool {
    let Some(first) = source.lines().find(|line| !line.trim().is_empty()) else {
        return false;
    };
    let first = first.trim_start();
    first.starts_with("flowchart")
        || first.starts_with("graph")
        || first.starts_with("sequenceDiagram")
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    source: String,
    width: u16,
    height: u16,
}

struct CachedDiagram {
    protocol: SlicedProtocol,
    bytes: usize,
}

struct RenderJob {
    key: CacheKey,
    font_size: ratatui_image::FontSize,
}

struct RenderedDiagram {
    key: CacheKey,
    image: Result<(DynamicImage, usize), String>,
}

pub struct DiagramCache {
    picker: Picker,
    entries: HashMap<CacheKey, CachedDiagram>,
    jobs: SyncSender<RenderJob>,
    results: Receiver<RenderedDiagram>,
    pending: HashSet<CacheKey>,
    failures: HashSet<CacheKey>,
    lru: VecDeque<CacheKey>,
    bytes: usize,
}

impl DiagramCache {
    pub fn new(picker: Picker) -> Self {
        let (jobs, job_receiver) = mpsc::sync_channel::<RenderJob>(1);
        let (result_sender, results) = mpsc::channel();
        std::thread::spawn(move || {
            while let Ok(job) = job_receiver.recv() {
                let image = render_image(
                    &job.key.source,
                    job.key.width,
                    job.key.height,
                    job.font_size,
                );
                if result_sender
                    .send(RenderedDiagram {
                        key: job.key,
                        image,
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            picker,
            entries: HashMap::new(),
            jobs,
            results,
            pending: HashSet::new(),
            failures: HashSet::new(),
            lru: VecDeque::new(),
            bytes: 0,
        }
    }

    pub fn supports_graphics(&self) -> bool {
        self.picker.protocol_type() != ProtocolType::Halfblocks
    }

    pub fn request(&mut self, source: &str, width: u16, height: u16) {
        self.collect_finished();
        let key = CacheKey {
            source: source.to_owned(),
            width,
            height,
        };
        if self.entries.contains_key(&key) {
            self.touch(&key);
            return;
        }
        if self.pending.contains(&key) || self.failures.contains(&key) {
            return;
        }
        let job = RenderJob {
            key: key.clone(),
            font_size: self.picker.font_size(),
        };
        match self.jobs.try_send(job) {
            Ok(()) => {
                self.pending.insert(key);
            }
            Err(TrySendError::Disconnected(_)) => {
                self.failures.insert(key);
            }
            Err(TrySendError::Full(_)) => {}
        }
    }

    pub fn protocol(&self, source: &str, width: u16, height: u16) -> Option<&SlicedProtocol> {
        self.entries
            .get(&CacheKey {
                source: source.to_owned(),
                width,
                height,
            })
            .map(|entry| &entry.protocol)
    }

    pub fn is_failed(&self, source: &str, width: u16, height: u16) -> bool {
        self.failures.contains(&CacheKey {
            source: source.to_owned(),
            width,
            height,
        })
    }

    fn collect_finished(&mut self) {
        let finished = self.results.try_iter().collect::<Vec<_>>();
        for RenderedDiagram { key, image } in finished {
            self.pending.remove(&key);
            match image.and_then(|(image, bytes)| {
                SlicedProtocol::new_with_resize(
                    &self.picker,
                    image,
                    Size::new(key.width, key.height),
                    Resize::Fit(None),
                )
                .map(|protocol| (protocol, bytes))
                .map_err(|error| error.to_string())
            }) {
                Ok((protocol, bytes)) => self.insert(key, CachedDiagram { protocol, bytes }),
                Err(_) => {
                    self.failures.insert(key);
                }
            }
        }
    }

    fn touch(&mut self, key: &CacheKey) {
        if let Some(index) = self.lru.iter().position(|candidate| candidate == key) {
            self.lru.remove(index);
        }
        self.lru.push_back(key.clone());
    }

    fn insert(&mut self, key: CacheKey, entry: CachedDiagram) {
        self.bytes = self.bytes.saturating_add(entry.bytes);
        self.entries.insert(key.clone(), entry);
        self.touch(&key);
        while self.bytes > MAX_CACHE_BYTES && self.lru.len() > 1 {
            if let Some(oldest) = self.lru.pop_front()
                && let Some(entry) = self.entries.remove(&oldest)
            {
                self.bytes = self.bytes.saturating_sub(entry.bytes);
            }
        }
    }
}

fn render_image(
    source: &str,
    width: u16,
    height: u16,
    font_size: ratatui_image::FontSize,
) -> Result<(DynamicImage, usize), String> {
    if !is_supported_source(source) {
        return Err("v0.3 supports flowchart and sequenceDiagram".to_owned());
    }
    let width_px = u32::from(width).saturating_mul(u32::from(font_size.width));
    let height_px = u32::from(height).saturating_mul(u32::from(font_size.height));
    let raster = RasterOptions::default()
        .with_fit_to(RasterFitBox::contain(width_px, height_px))
        .with_scale(RASTER_SCALE)
        .with_background("#ffffff");
    let renderer = HeadlessRenderer::new().with_diagram_id("docsail");
    let png = renderer
        .render_png_sync(source, &raster)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Mermaid diagram was not detected".to_owned())?;
    let compressed_bytes = png.len();
    let image = image::load_from_memory(&png).map_err(|error| error.to_string())?;
    let raw_bytes = image.width() as usize * image.height() as usize * 4;
    Ok((image, compressed_bytes.saturating_add(raw_bytes)))
}

pub fn rendered_blocks(
    document: &Document,
    mode: DisplayMode,
) -> impl Iterator<Item = (usize, &str)> {
    document
        .blocks()
        .iter()
        .enumerate()
        .filter_map(move |(index, block)| {
            let Block::CodeBlock { language, content } = block else {
                return None;
            };
            (mode == DisplayMode::Rendered && is_mermaid(language.as_deref()))
                .then_some((index, content.as_str()))
        })
}

pub fn image_widget(protocol: &SlicedProtocol, y: i16) -> SlicedImage<'_> {
    SlicedImage::new(protocol, SignedPosition::from((0, y)))
}

#[cfg(test)]
mod tests {
    use super::{
        DiagramCache, DisplayMode, is_mermaid, is_supported_source, render_image, rendered_blocks,
    };
    use crate::markdown::parse;

    #[test]
    fn identifies_mermaid_fences_case_insensitively() {
        assert!(is_mermaid(Some("mermaid")));
        assert!(is_mermaid(Some(" Mermaid ")));
        assert!(!is_mermaid(Some("mermaid {theme: dark}")));
    }

    #[test]
    fn limits_the_initial_renderer_to_two_diagram_families() {
        assert!(is_supported_source("flowchart TD\nA --> B"));
        assert!(is_supported_source("sequenceDiagram\nA->>B: こんにちは"));
        assert!(!is_supported_source("classDiagram\nA <|-- B"));
    }

    #[test]
    fn exposes_only_mermaid_blocks_in_rendered_mode() {
        let document =
            parse("```mermaid\nflowchart TD\nA --> B\n```\n\n```rust\nfn main() {}\n```");
        assert_eq!(rendered_blocks(&document, DisplayMode::Rendered).count(), 1);
        assert_eq!(rendered_blocks(&document, DisplayMode::Source).count(), 0);
    }

    #[test]
    fn keeps_mermaid_source_on_terminals_without_graphics_protocols() {
        assert!(
            !DiagramCache::new(ratatui_image::picker::Picker::halfblocks()).supports_graphics()
        );
    }

    #[test]
    fn rasterizes_a_flowchart_with_a_japanese_label() {
        let (image, bytes) = render_image(
            "flowchart TD\nA[開始] --> B[完了]",
            48,
            12,
            ratatui_image::FontSize::new(10, 20),
        )
        .unwrap();

        assert!(bytes > 0);
        assert!(image.width() > 0);
        assert!(image.height() > 0);
    }
}
