//! Runtime pixel art sprite generation.
//!
//! Creates 16×16 role-based sprites from const pixel data.
//! No asset files needed — everything generated at startup.

use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::HashMap;

pub struct SpriteArtPlugin;

impl Plugin for SpriteArtPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, generate_sprites);
    }
}

/// Resource holding sprite handles keyed by role name.
#[derive(Resource, Default)]
pub struct RoleSprites {
    pub sprites: HashMap<String, Handle<Image>>,
    pub default_sprite: Option<Handle<Image>>,
}

/// Transparent pixel.
const T: [u8; 4] = [0, 0, 0, 0];

/// Generate all role sprites at startup.
fn generate_sprites(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let mut role_sprites = RoleSprites::default();

    // Researcher — magnifying glass
    let researcher = create_sprite(&mut images, &RESEARCHER_PIXELS);
    role_sprites.sprites.insert("researcher".into(), researcher);

    // Coder — code brackets <>
    let coder = create_sprite(&mut images, &CODER_PIXELS);
    role_sprites.sprites.insert("coder".into(), coder);

    // Reviewer — checkmark / eye
    let reviewer = create_sprite(&mut images, &REVIEWER_PIXELS);
    role_sprites.sprites.insert("reviewer".into(), reviewer);

    // Tester — flask / beaker
    let tester = create_sprite(&mut images, &TESTER_PIXELS);
    role_sprites.sprites.insert("tester".into(), tester);

    // Deployer — rocket
    let deployer = create_sprite(&mut images, &DEPLOYER_PIXELS);
    role_sprites.sprites.insert("deployer".into(), deployer);

    // Planner — compass / star
    let planner = create_sprite(&mut images, &PLANNER_PIXELS);
    role_sprites.sprites.insert("planner".into(), planner);

    // Default — diamond
    let default_sprite = create_sprite(&mut images, &DEFAULT_PIXELS);
    role_sprites.default_sprite = Some(default_sprite);

    commands.insert_resource(role_sprites);
}

fn create_sprite(
    images: &mut Assets<Image>,
    pixels: &[[u8; 4]; 256], // 16×16 = 256 pixels
) -> Handle<Image> {
    let data: Vec<u8> = pixels.iter().flat_map(|p| p.iter().copied()).collect();
    let image = Image::new(
        Extent3d { width: 16, height: 16, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    images.add(image)
}

// ── Color palette ──
const W: [u8; 4] = [255, 255, 255, 255]; // white
const L: [u8; 4] = [200, 200, 220, 255]; // light gray
const M: [u8; 4] = [140, 140, 170, 255]; // medium gray
const D: [u8; 4] = [60, 60, 90, 255];    // dark gray/blue
const B: [u8; 4] = [30, 30, 50, 255];    // near-black

// Role accent colors
const R1: [u8; 4] = [100, 180, 255, 255]; // researcher blue
const R2: [u8; 4] = [60, 130, 220, 255];
const C1: [u8; 4] = [120, 230, 120, 255]; // coder green
const C2: [u8; 4] = [70, 180, 70, 255];
const V1: [u8; 4] = [255, 200, 80, 255];  // reviewer gold
const V2: [u8; 4] = [220, 160, 40, 255];
const T1: [u8; 4] = [200, 120, 255, 255]; // tester purple
const T2: [u8; 4] = [150, 80, 220, 255];
const D1: [u8; 4] = [255, 140, 80, 255];  // deployer orange
const D2: [u8; 4] = [220, 100, 40, 255];
const P1: [u8; 4] = [255, 255, 130, 255]; // planner yellow
const P2: [u8; 4] = [220, 220, 80, 255];
const DF: [u8; 4] = [180, 180, 200, 255]; // default silver

// 16×16 pixel art, row by row (top to bottom)
// Researcher: magnifying glass shape
#[rustfmt::skip]
const RESEARCHER_PIXELS: [[u8; 4]; 256] = [
    T,  T,  T,  T,  T,  R2, R2, R2, R2, R2, T,  T,  T,  T,  T,  T,
    T,  T,  T,  R2, R1, R1, W,  W,  R1, R1, R2, T,  T,  T,  T,  T,
    T,  T,  R2, R1, W,  L,  L,  L,  L,  W,  R1, R2, T,  T,  T,  T,
    T,  R2, R1, L,  L,  T,  T,  T,  T,  L,  L,  R1, R2, T,  T,  T,
    T,  R1, W,  L,  T,  T,  T,  T,  T,  T,  L,  W,  R1, T,  T,  T,
    R2, R1, L,  T,  T,  T,  T,  T,  T,  T,  T,  L,  R1, R2, T,  T,
    R2, W,  L,  T,  T,  T,  T,  T,  T,  T,  T,  L,  W,  R2, T,  T,
    R2, W,  L,  T,  T,  T,  T,  T,  T,  T,  T,  L,  W,  R2, T,  T,
    R2, R1, L,  T,  T,  T,  T,  T,  T,  T,  T,  L,  R1, R2, T,  T,
    T,  R1, W,  L,  T,  T,  T,  T,  T,  T,  L,  W,  R1, T,  T,  T,
    T,  R2, R1, L,  L,  T,  T,  T,  T,  L,  L,  R1, R2, T,  T,  T,
    T,  T,  R2, R1, W,  L,  L,  L,  L,  W,  R1, R2, D,  T,  T,  T,
    T,  T,  T,  R2, R1, R1, W,  W,  R1, R1, R2, D,  D,  T,  T,  T,
    T,  T,  T,  T,  T,  R2, R2, R2, R2, R2, T,  D,  D,  D,  T,  T,
    T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  D,  D,  D,  T,
    T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  D,  D,  T,
];

// Coder: code brackets < / >
#[rustfmt::skip]
const CODER_PIXELS: [[u8; 4]; 256] = [
    T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  C1, T,  T,  T,  T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  C1, C2, T,  T,  T,  T,  T,  T,  C1, T,  T,  T,
    T,  T,  T,  C1, C2, T,  T,  T,  T,  T,  M,  T,  C2, C1, T,  T,
    T,  T,  C1, C2, T,  T,  T,  T,  T,  T,  T,  M,  T,  C2, C1, T,
    T,  C1, C2, T,  T,  T,  T,  T,  T,  M,  T,  T,  T,  T,  C2, C1,
    C1, C2, T,  T,  T,  T,  T,  T,  M,  T,  T,  T,  T,  T,  T,  C2,
    C2, T,  T,  T,  T,  T,  T,  M,  T,  T,  T,  T,  T,  T,  T,  C1,
    C2, T,  T,  T,  T,  T,  T,  M,  T,  T,  T,  T,  T,  T,  T,  C1,
    C1, C2, T,  T,  T,  T,  T,  T,  M,  T,  T,  T,  T,  T,  T,  C2,
    T,  C1, C2, T,  T,  T,  T,  T,  T,  M,  T,  T,  T,  T,  C2, C1,
    T,  T,  C1, C2, T,  T,  T,  T,  T,  T,  T,  M,  T,  C2, C1, T,
    T,  T,  T,  C1, C2, T,  T,  T,  T,  T,  M,  T,  C2, C1, T,  T,
    T,  T,  T,  T,  C1, C2, T,  T,  T,  T,  T,  T,  C1, T,  T,  T,
    T,  T,  T,  T,  T,  C1, T,  T,  T,  T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,
];

// Reviewer: eye / shield with checkmark
#[rustfmt::skip]
const REVIEWER_PIXELS: [[u8; 4]; 256] = [
    T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  V2, V2, V2, V2, V2, V2, V2, V2, V2, V2, T,  T,  T,
    T,  T,  V2, V1, V1, V1, V1, V1, V1, V1, V1, V1, V1, V2, T,  T,
    T,  V2, V1, V1, V1, V1, V1, V1, V1, V1, V1, V1, V1, V1, V2, T,
    T,  V2, V1, V1, V1, V1, V1, V1, V1, V1, V1, V1, W,  V1, V2, T,
    T,  V2, V1, V1, V1, V1, V1, V1, V1, V1, V1, W,  V1, V1, V2, T,
    T,  V2, V1, V1, V1, V1, V1, V1, V1, V1, W,  V1, V1, V1, V2, T,
    T,  V2, V1, W,  V1, V1, V1, V1, V1, W,  V1, V1, V1, V1, V2, T,
    T,  V2, V1, V1, W,  V1, V1, V1, W,  V1, V1, V1, V1, V1, V2, T,
    T,  V2, V1, V1, V1, W,  V1, W,  V1, V1, V1, V1, V1, V1, V2, T,
    T,  V2, V1, V1, V1, V1, W,  V1, V1, V1, V1, V1, V1, V1, V2, T,
    T,  T,  V2, V1, V1, V1, V1, V1, V1, V1, V1, V1, V1, V2, T,  T,
    T,  T,  T,  V2, V1, V1, V1, V1, V1, V1, V1, V1, V2, T,  T,  T,
    T,  T,  T,  T,  V2, V2, V1, V1, V1, V2, V2, T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T,  V2, V2, V2, T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,
];

// Tester: flask / beaker
#[rustfmt::skip]
const TESTER_PIXELS: [[u8; 4]; 256] = [
    T,  T,  T,  T,  T,  T2, T1, T1, T1, T2, T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T2, T1, T1, T1, T2, T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T2, L,  L,  L,  T2, T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T2, L,  L,  L,  T2, T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T2, T1, L,  L,  L,  T1, T2, T,  T,  T,  T,  T,
    T,  T,  T,  T2, T1, L,  L,  L,  L,  L,  T1, T2, T,  T,  T,  T,
    T,  T,  T2, T1, L,  L,  L,  L,  L,  L,  L,  T1, T2, T,  T,  T,
    T,  T2, T1, L,  L,  L,  L,  L,  L,  L,  L,  L,  T1, T2, T,  T,
    T,  T2, T1, L,  L,  L,  W,  W,  L,  L,  L,  L,  T1, T2, T,  T,
    T,  T2, T1, T1, L,  W,  T1, T1, W,  L,  L,  T1, T1, T2, T,  T,
    T,  T2, T1, T1, T1, W,  T1, T1, W,  T1, T1, T1, T1, T2, T,  T,
    T,  T2, T1, T1, T1, T1, W,  W,  T1, T1, T1, T1, T1, T2, T,  T,
    T,  T2, T1, T1, T1, T1, T1, T1, T1, T1, T1, T1, T1, T2, T,  T,
    T,  T,  T2, T2, T1, T1, T1, T1, T1, T1, T1, T2, T2, T,  T,  T,
    T,  T,  T,  T,  T2, T2, T2, T2, T2, T2, T2, T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,
];

// Deployer: rocket
#[rustfmt::skip]
const DEPLOYER_PIXELS: [[u8; 4]; 256] = [
    T,  T,  T,  T,  T,  T,  T,  D1, T,  T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T,  D1, W,  D1, T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  D1, W,  W,  W,  D1, T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  D2, W,  L,  W,  D2, T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  D2, D1, L,  L,  L,  D1, D2, T,  T,  T,  T,  T,
    T,  T,  T,  T,  D2, D1, L,  M,  L,  D1, D2, T,  T,  T,  T,  T,
    T,  T,  T,  D2, D1, L,  M,  M,  M,  L,  D1, D2, T,  T,  T,  T,
    T,  T,  T,  D2, D1, L,  M,  D,  M,  L,  D1, D2, T,  T,  T,  T,
    T,  T,  D2, D1, L,  M,  D,  D,  D,  M,  L,  D1, D2, T,  T,  T,
    T,  T,  D2, D1, L,  M,  D,  B,  D,  M,  L,  D1, D2, T,  T,  T,
    T,  D2, D1, L,  M,  M,  M,  M,  M,  M,  M,  L,  D1, D2, T,  T,
    D2, D1, T,  T,  T,  D2, D1, D1, D1, D2, T,  T,  T,  D1, D2, T,
    D1, T,  T,  T,  T,  D2, D1, D1, D1, D2, T,  T,  T,  T,  D1, T,
    T,  T,  T,  T,  T,  T,  D1, D2, D1, T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  D1, D2, T,  D2, D1, T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  D2, T,  T,  T,  D2, T,  T,  T,  T,  T,  T,
];

// Planner: compass star
#[rustfmt::skip]
const PLANNER_PIXELS: [[u8; 4]; 256] = [
    T,  T,  T,  T,  T,  T,  T,  P1, T,  T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T,  P1, P1, P1, T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T,  P2, P1, P2, T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  P2, P2, P1, P2, P2, T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  P2, P1, P2, P1, P2, P1, P2, T,  T,  T,  T,  T,
    T,  T,  T,  P2, P1, P1, P2, P1, P2, P1, P1, P2, T,  T,  T,  T,
    T,  P1, P2, P2, P2, P2, P1, W,  P1, P2, P2, P2, P2, P1, T,  T,
    P1, P1, P1, P1, P1, P1, W,  W,  W,  P1, P1, P1, P1, P1, P1, T,
    T,  P1, P2, P2, P2, P2, P1, W,  P1, P2, P2, P2, P2, P1, T,  T,
    T,  T,  T,  P2, P1, P1, P2, P1, P2, P1, P1, P2, T,  T,  T,  T,
    T,  T,  T,  T,  P2, P1, P2, P1, P2, P1, P2, T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  P2, P2, P1, P2, P2, T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T,  P2, P1, P2, T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T,  P1, P1, P1, T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T,  T,  P1, T,  T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,
];

// Default: diamond shape
#[rustfmt::skip]
const DEFAULT_PIXELS: [[u8; 4]; 256] = [
    T,  T,  T,  T,  T,  T,  T,  DF, T,  T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T,  DF, W,  DF, T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  DF, W,  W,  W,  DF, T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  DF, W,  L,  L,  L,  W,  DF, T,  T,  T,  T,  T,
    T,  T,  T,  DF, W,  L,  L,  L,  L,  L,  W,  DF, T,  T,  T,  T,
    T,  T,  DF, W,  L,  L,  L,  M,  L,  L,  L,  W,  DF, T,  T,  T,
    T,  DF, W,  L,  L,  L,  M,  M,  M,  L,  L,  L,  W,  DF, T,  T,
    DF, W,  W,  L,  L,  M,  M,  M,  M,  M,  L,  L,  W,  W,  DF, T,
    T,  DF, W,  L,  L,  L,  M,  M,  M,  L,  L,  L,  W,  DF, T,  T,
    T,  T,  DF, W,  L,  L,  L,  M,  L,  L,  L,  W,  DF, T,  T,  T,
    T,  T,  T,  DF, W,  L,  L,  L,  L,  L,  W,  DF, T,  T,  T,  T,
    T,  T,  T,  T,  DF, W,  L,  L,  L,  W,  DF, T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  DF, W,  W,  W,  DF, T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T,  DF, W,  DF, T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T,  T,  DF, T,  T,  T,  T,  T,  T,  T,  T,
    T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,
];
