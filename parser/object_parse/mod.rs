use std::convert::TryInto;

use anyhow::anyhow;
use object::{
    elf, Architecture, BinaryFormat, Endianness, File, Object, ObjectSection, ObjectSegment,
    ObjectSymbol, Relocation, RelocationFlags, RelocationTarget, SectionFlags, Symbol, SymbolIndex,
    SymbolKind,
};
use twiggy_ir as ir;

fn maybe_thumb_real_addr(file: &File, addr: u64) -> u64 {
    match file.architecture() {
        Architecture::Arm => {
            addr & !1 // Clear LSB. LSB is set when the function is a Thumb function.
        }
        _ => addr,
    }
}

pub fn parse(data: &[u8]) -> anyhow::Result<ir::Items> {
    let file: File =
        File::parse(data).map_err(|err| anyhow!("Failed to parse data with err: {:?}", err))?;

    let mut alloc_size = 0;
    for segment in file.segments() {
        alloc_size += segment.size();
    }

    let mut items = ir::ItemsBuilder::new(alloc_size as u32);

    let mut symbols = vec![];
    for symbol in file.symbols() {
        if !symbol.is_definition() {
            continue;
        }

        if symbol.size() == 0 {
            continue;
        }

        // Filter out symbols in non-allocated sections. Their symbol values do not correspond to
        // actual runtime addresses.
        match file
            .section_by_index(symbol.section_index().unwrap())
            .unwrap()
            .flags()
        {
            SectionFlags::Elf { sh_flags } => {
                if sh_flags as u32 & elf::SHF_ALLOC != elf::SHF_ALLOC {
                    continue;
                }
            }
            _ => {}
        }

        if !file.segments().any(|segment| {
            segment
                .data_range(maybe_thumb_real_addr(&file, symbol.address()), 1)
                .ok()
                .flatten()
                .is_some()
        }) {
            // Symbol not part of any loaded segment
            continue;
        }

        symbols.push((symbol.address(), symbol.size(), symbol.index()));

        let id = ir::Id::entry(symbol.section_index().unwrap().0, symbol.index().0);
        let name = symbol.name().unwrap();
        let kind: ir::ItemKind = ir::Code::new(name).into();
        
        let item = ir::Item::new(id, name, symbol.size() as u32, 0..1, kind);
        if maybe_thumb_real_addr(&file, symbol.address())
            == maybe_thumb_real_addr(&file, file.entry())
        {
            items.add_root(item);
        } else {
            items.add_item(item);
        }
    }

    if let BinaryFormat::Elf = file.format() {
        let mut any_relocs = false;
        for section in file.sections() {
            if section.name().unwrap().starts_with(".debug")
                || section.name().unwrap().starts_with(".eh_frame")
            {
                continue;
            }

            for (offset, reloc) in section.relocations() {
                any_relocs = true;
                edge_for_reloc(&file, &mut items, &symbols, offset, reloc);
            }
        }

        if !any_relocs {
            eprintln!(
                "Warning: Couldn't find any relocations. \
                 The dominators, garbage and paths subcommands will not function correctly.\n\
                 Hint: Try recompiling the binary with --emit-relocs.\n"
            );
        }
    } else {
        eprintln!(
            "Warning: Note: The dominators, garbage and paths subcommands currently only support \
                WASM and ELF.\n"
        )
    }

    Ok(items.finish())
}

fn read_at<const N: usize>(file: &File<'_>, offset: u64) -> [u8; N] {
    file.segments()
        .find_map(|segment| segment.data_range(offset, N as u64).unwrap())
        .unwrap()
        .try_into()
        .unwrap()
}

fn edge_for_reloc(
    file: &File<'_>,
    items: &mut twiggy_ir::ItemsBuilder,
    symbols: &Vec<(u64, u64, SymbolIndex)>,
    offset: u64,
    reloc: Relocation,
) {
    let Some(reloc_source) = symbol_for_addr(file, symbols, offset) else {
        return;
    };

    match reloc.target() {
        // If the reloc is relative to a non-section symbol, we can directly use this symbol as target.
        RelocationTarget::Symbol(reloc_target_idx)
            if file.symbol_by_index(reloc_target_idx).unwrap().kind() != SymbolKind::Section =>
        {
            if !symbols.iter().any(|&(_, _, idx)| idx == reloc_target_idx) {
                return;
            }
            let reloc_target = file.symbol_by_index(reloc_target_idx).unwrap();
            add_edge_for_symbol(items, reloc_source, reloc_target);
            return;
        }

        // Otherwise we need to compute the target address and find the symbol covering this address.
        _ => {}
    }

    let implicit_addend = match file.architecture() {
        Architecture::Arm => {
            assert_eq!(file.endianness(), Endianness::Little);
            assert!(reloc.has_implicit_addend());
            match reloc.flags() {
                RelocationFlags::Elf {
                    r_type: elf::R_ARM_ABS32,
                } => u64::from(u32::from_le_bytes(read_at(file, offset))) as i64,
                ty => todo!("{:?}", ty),
            }
        }
        Architecture::X86_64 => {
            assert!(!reloc.has_implicit_addend());
            match reloc.flags() {
                RelocationFlags::Elf {
                    r_type: elf::R_X86_64_PC32 | elf::R_X86_64_PLT32,
                } => 4,
                RelocationFlags::Elf {
                    r_type: elf::R_X86_64_64,
                } => 0,
                ty => todo!("{:?}", ty),
            }
        }
        arch => todo!("relocations for architecture {:?} not yet supported", arch),
    };

    let symbol_addr = match reloc.target() {
        RelocationTarget::Symbol(reloc_target_idx) => {
            file.symbol_by_index(reloc_target_idx).unwrap().address()
        }
        RelocationTarget::Absolute => 0,
        _ => todo!(),
    };

    let target_addr = (symbol_addr as i64 + reloc.addend() + implicit_addend) as u64;
    let Some(reloc_target) = symbol_for_addr(file, symbols, target_addr) else {
        return;
    };
    add_edge_for_symbol(items, reloc_source, reloc_target);
}

fn symbol_for_addr<'data, 'file>(
    file: &'file File<'data>,
    symbols: &Vec<(u64, u64, SymbolIndex)>,
    offset: u64,
) -> Option<Symbol<'data, 'file>> {
    let Some(&(_, _, reloc_source_idx)) = symbols
        .iter()
        .find(|&&(addr, size, _idx)| (addr..addr + size).contains(&offset))
    else {
        return None;
    };

    Some(file.symbol_by_index(reloc_source_idx).unwrap())
}

fn add_edge_for_symbol(
    items: &mut twiggy_ir::ItemsBuilder,
    reloc_source: Symbol<'_, '_>,
    reloc_target: Symbol<'_, '_>,
) {
    items.add_edge(
        ir::Id::entry(
            reloc_source.section_index().unwrap().0,
            reloc_source.index().0,
        ),
        ir::Id::entry(
            reloc_target.section_index().unwrap().0,
            reloc_target.index().0,
        ),
    );
}
