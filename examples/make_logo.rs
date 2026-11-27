//! Draws the skills mascot, a pixel game cartridge with a lightning bolt on its label (a skill you load on demand), as SVG.
//!
//!     cargo run --example make_logo
//!
//! writes `assets/icon.svg` and `assets/src/banner.svg`. Render the PNGs with `rsvg-convert -w 512 assets/icon.svg -o assets/icon.png`
//! and `rsvg-convert -w 2000 assets/src/banner.svg -o assets/banner.png`.
use std::collections::BTreeMap;
use std::path::Path;

/// E edge, B body, b shade, L label, l label shade, K bolt, P pin, G pin gap.
const BODY: [&str; 24] = [
    "..EEEEEEEEEEEEEEE...",
    ".EBBBBBBBBBBBBBBBE..",
    ".EBBBBBBBBBBBBBBBBE.",
    ".EBBLLLLLLLLLLLLBBE.",
    ".EBBLLLLLLLLLLLLBBE.",
    ".EBBLLLLLLLLLLLLBBE.",
    ".EBBLLLLLLLLLLLLBBE.",
    ".EBBLLLLLLLLLLLLBBE.",
    ".EBBLLLLLLLLLLLLBBE.",
    ".EBBLLLLLLLLLLLLBBE.",
    ".EBBLLLLLLLLLLLLBBE.",
    ".EBBLLLLLLLLLLLLBBE.",
    ".EBBLLLLLLLLLLLLBBE.",
    ".EBBLllllllllllLBBE.",
    ".EBBBBBBBBBBBBBBBBE.",
    ".EBBBBBBBBBBBBBBBBE.",
    ".EBBBBBBBBBBBBBBBBE.",
    ".EEEEEEEEEEEEEEEEEE.",
    "..EbbbbbbbbbbbbbbE..",
    "..EbPPbPPbPPbPPbPPE.",
    "..EbPPbPPbPPbPPbPPE.",
    "..EbPPbPPbPPbPPbPPE.",
    "..EbPPbPPbPPbPPbPPE.",
    "..EEEEEEEEEEEEEEEE..",
];

/// The bolt, drawn on the label (top-left at BOLT_AT).
const BOLT: [&str; 9] = ["....KK..", "...KK...", "..KK....", ".KKKKKK.", "...KK...", "..KK....", ".KK.....", ".K......", "K......."];
const BOLT_AT: (i32, i32) = (6, 3);

fn color(ch: char) -> Option<&'static str> {
    Some(match ch {
        'E' => "#c9d4ff",
        'B' => "#2b3560",
        'b' => "#1d2445",
        'L' => "#ffc857",
        'l' => "#e0a63a",
        'K' => "#1d2445",
        'P' => "#3dffa0",
        _ => return None,
    })
}

const DEFS: &str = r##"<defs>
    <linearGradient id="tile" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#0a1612"/><stop offset="1" stop-color="#143a2f"/></linearGradient>
    <radialGradient id="aura" cx="0.5" cy="0.45" r="0.55"><stop offset="0" stop-color="#3dffa0" stop-opacity="0.34"/><stop offset="1" stop-color="#3dffa0" stop-opacity="0"/></radialGradient>
    <filter id="glow" x="-30%" y="-30%" width="160%" height="160%"><feGaussianBlur stdDeviation="7"/></filter>
    <pattern id="scan" width="4" height="4" patternUnits="userSpaceOnUse"><rect width="4" height="1" fill="#000" fill-opacity="0.16"/></pattern>
  </defs>"##;

fn mascot(px: i32, ox: i32, oy: i32) -> String {
    let mut cells: BTreeMap<(i32, i32), &str> = BTreeMap::new(); // keyed (row, col): reading order
    for (y, row) in BODY.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if let Some(c) = color(ch) {
                cells.insert((y as i32, x as i32), c);
            }
        }
    }
    for (y, row) in BOLT.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if ch == 'K' {
                cells.insert((BOLT_AT.1 + y as i32, BOLT_AT.0 + x as i32), "#1d2445");
            }
        }
    }
    cells.iter().map(|((y, x), c)| format!(r#"<rect x="{}" y="{}" width="{px}" height="{px}" fill="{c}"/>"#, ox + x * px, oy + y * px)).collect()
}

fn sparkles(spots: &[(i32, i32, i32, f32)]) -> String {
    spots.iter().map(|(x, y, s, o)| format!(r##"<rect x="{x}" y="{y}" width="{s}" height="{s}" fill="#3dffa0" fill-opacity="{o}"/>"##)).collect()
}

fn icon() -> String {
    let m = mascot(9, 38, 20);
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" role="img" aria-label="skills: a pixel game cartridge with a lightning bolt on its label">
  {DEFS}
  <rect x="8" y="8" width="240" height="240" rx="56" fill="url(#tile)"/>
  <rect x="8" y="8" width="240" height="240" rx="56" fill="url(#aura)"/>
  <rect x="8" y="8" width="240" height="240" rx="56" fill="url(#scan)"/>
  <rect x="9" y="9" width="238" height="238" rx="55" fill="none" stroke="#3dffa0" stroke-opacity="0.35" stroke-width="2"/>
  <g filter="url(#glow)" opacity="0.6">{m}</g>
  <g shape-rendering="crispEdges">{m}</g>
  <g shape-rendering="crispEdges">{}</g>
</svg>
"##,
        sparkles(&[(26, 56, 6, 0.9), (214, 80, 6, 0.8), (30, 190, 4, 0.7), (222, 176, 4, 0.9), (206, 40, 4, 0.6)])
    )
}

fn banner() -> String {
    let m = mascot(10, 70, 30);
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1000 300" width="1000" height="300" role="img" aria-label="skills">
  {DEFS}
  <rect width="1000" height="300" rx="28" fill="url(#tile)"/>
  <rect width="1000" height="300" rx="28" fill="url(#scan)"/>
  <rect x="0.5" y="0.5" width="999" height="299" rx="27.5" fill="none" stroke="#3dffa0" stroke-opacity="0.25"/>
  <ellipse cx="170" cy="150" rx="190" ry="150" fill="url(#aura)"/>
  <g filter="url(#glow)" opacity="0.6">{m}</g>
  <g shape-rendering="crispEdges">{m}</g>
  <g shape-rendering="crispEdges">{}</g>
  <text x="372" y="156" font-family="'JetBrains Mono', 'JetBrainsMono Nerd Font Mono', 'Fira Code', Consolas, monospace" font-size="96" font-weight="800" letter-spacing="-4">
    <tspan fill="#e6f5ee">skill</tspan><tspan fill="#3dffa0">s</tspan>
  </text>
  <text x="376" y="208" font-family="Inter, 'Inter Variable', 'Segoe UI', Helvetica, Arial, sans-serif" font-size="24" fill="#9db8ac">Every skill on disk. Only what you need in context.</text>
  <rect x="376" y="232" width="150" height="3" rx="1.5" fill="#ffc857"/>
</svg>
"##,
        sparkles(&[(36, 50, 6, 0.9), (330, 70, 6, 0.8), (40, 250, 4, 0.7), (320, 230, 4, 0.9), (300, 28, 4, 0.6)])
    )
}

fn main() -> std::io::Result<()> {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    std::fs::create_dir_all(assets.join("src"))?;
    std::fs::write(assets.join("icon.svg"), icon())?;
    std::fs::write(assets.join("src/banner.svg"), banner())?;
    println!("wrote assets/icon.svg and assets/src/banner.svg");
    Ok(())
}
