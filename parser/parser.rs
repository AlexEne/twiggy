//! Parses binaries into `twiggy_ir::Items`.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path;

use twiggy_ir as ir;
use twiggy_traits as traits;

#[cfg(feature = "dwarf")]
mod object_parse;
mod wasm_parse;

const WASM_MAGIC_NUMBER: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// Parse the file at the given path into IR items.
pub fn read_and_parse<P: AsRef<path::Path>>(
    path: P,
    mode: traits::ParseMode,
) -> anyhow::Result<ir::Items> {
    let path = path.as_ref();
    let mut file = fs::File::open(path)?;
    let mut data = vec![];
    file.read_to_end(&mut data)?;

    match mode {
        traits::ParseMode::Wasm => parse_wasm(&data),
        #[cfg(feature = "dwarf")]
        traits::ParseMode::Dwarf => parse_other(&data),
        traits::ParseMode::Auto => parse_auto(path.extension(), &data),
    }
}

/// Parse the given data into IR items.
pub fn parse(data: &[u8]) -> anyhow::Result<ir::Items> {
    parse_fallback(data)
}

/// A trait for parsing things into `ir::Item`s.
pub(crate) trait Parse<'a> {
    /// Any extra data needed to parse this type's items.
    type ItemsExtra;

    /// Parse `Self` into one or more `ir::Item`s and add them to the builder.
    fn parse_items(
        self,
        items: &mut ir::ItemsBuilder,
        extra: Self::ItemsExtra,
    ) -> anyhow::Result<()>;

    /// Any extra data needed to parse this type's edges.
    type EdgesExtra;

    /// Parse edges between items. This is only called *after* we have already
    /// parsed items.
    fn parse_edges(
        self,
        items: &mut ir::ItemsBuilder,
        extra: Self::EdgesExtra,
    ) -> anyhow::Result<()>;
}

fn parse_auto(extension: Option<&OsStr>, data: &[u8]) -> anyhow::Result<ir::Items> {
    if sniff_wasm(extension, &data) {
        parse_wasm(&data)
    } else {
        #[cfg(feature = "dwarf")]
        let res = parse_other(&data);
        #[cfg(not(feature = "dwarf"))]
        let res = parse_fallback(&data);
        res
    }
}

fn sniff_wasm(extension: Option<&OsStr>, data: &[u8]) -> bool {
    match extension.and_then(|s| s.to_str()) {
        Some("wasm") => true,
        _ => data.get(0..4) == Some(&WASM_MAGIC_NUMBER),
    }
}

fn parse_wasm(data: &[u8]) -> anyhow::Result<ir::Items> {
    let mut items = ir::ItemsBuilder::new(data.len() as u32);

    let module1 = wasm_parse::ModuleReader::new(data);
    module1.parse_items(&mut items, ())?;
    let module2 = wasm_parse::ModuleReader::new(data);
    module2.parse_edges(&mut items, ())?;

    Ok(items.finish())
}

#[cfg(feature = "dwarf")]
fn parse_other(data: &[u8]) -> anyhow::Result<ir::Items> {
    object_parse::parse(&data)
}

fn parse_fallback(data: &[u8]) -> anyhow::Result<ir::Items> {
    parse_wasm(data)
}
