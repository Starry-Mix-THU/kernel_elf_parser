//! ELF information parsed from the ELF file

use xmas_elf::program::Flags;

use crate::auxv::{AuxEntry, AuxType};

/// ELF Program Header applied to the kernel
///
/// Details can be seen in the [ELF Program Header](https://refspecs.linuxbase.org/elf/gabi4+/ch5.pheader.html)
pub struct ELFPH {
    /// The start offset of the segment in the ELF file
    pub offset: usize,
    /// The destination virtual address of the segment in the kernel memory
    pub vaddr: usize,
    /// Memory size of the segment
    pub memsz: u64,
    /// File size of the segment
    pub filesz: u64,
    /// [`MappingFlags`] of the segment which is used to set the page table
    /// entry
    pub flags: Flags,
}

/// A wrapper for the ELF file data with some useful methods.
pub struct ELFParser<'a> {
    elf: &'a xmas_elf::ElfFile<'a>,
    /// Base address of the ELF file loaded into the memory.
    base: usize,
}

impl<'a> ELFParser<'a> {
    /// Create a new `ELFInfo` instance.
    /// # Arguments
    /// * `elf` - The ELF file data
    /// * `bias` - Bias for the base address of the PIE executable.
    pub fn new(elf: &'a xmas_elf::ElfFile, bias: usize) -> Result<Self, &'static str> {
        if elf.header.pt1.magic.as_slice() != b"\x7fELF" {
            return Err("invalid elf!");
        }
        let base = if elf.header.pt2.type_().as_type() == xmas_elf::header::Type::SharedObject {
            bias
        } else {
            0
        };
        Ok(Self { elf, base })
    }

    /// The entry point of the ELF file.
    pub fn entry(&self) -> usize {
        // TODO: base_load_address_offset?
        self.elf.header.pt2.entry_point() as usize + self.base
    }

    /// The number of program headers in the ELF file.
    pub fn phnum(&self) -> usize {
        self.elf.header.pt2.ph_count() as usize
    }

    /// The size of the program header table entry in the ELF file.
    pub fn phent(&self) -> usize {
        self.elf.header.pt2.ph_entry_size() as usize
    }

    /// The offset of the program header table in the ELF file.
    pub fn phdr(&self) -> usize {
        let ph_offset = self.elf.header.pt2.ph_offset() as usize;
        let header = self
            .elf
            .program_iter()
            .find(|header| {
                (header.offset()..header.offset() + header.file_size())
                    .contains(&(ph_offset as u64))
            })
            .expect("can not find program header table address in elf");
        ph_offset - header.offset() as usize + header.virtual_addr() as usize + self.base
    }

    /// The base address of the ELF file loaded into the memory.
    pub fn base(&self) -> usize {
        self.base
    }

    /// The ref of the ELF file data.
    pub fn elf(&self) -> &xmas_elf::ElfFile {
        self.elf
    }

    /// Part of auxiliary vectors from the ELF file.
    ///
    /// # Arguments
    ///
    /// * `pagesz` - The page size of the system
    /// * `ldso_base` - The base address of the dynamic linker (if exists)
    ///
    /// Details about auxiliary vectors are described in <https://articles.manugarg.com/aboutelfauxiliaryvectors.html>
    pub fn aux_vector(
        &self,
        pagesz: usize,
        ldso_base: Option<usize>,
    ) -> impl Iterator<Item = AuxEntry> {
        [
            (AuxType::PHDR, self.phdr()),
            (AuxType::PHENT, self.phent()),
            (AuxType::PHNUM, self.phnum()),
            (AuxType::PAGESZ, pagesz),
            (AuxType::ENTRY, self.entry()),
        ]
        .into_iter()
        .chain(ldso_base.into_iter().map(|base| (AuxType::BASE, base)))
        .map(|(at, val)| AuxEntry::new(at, val))
    }

    /// Read all [`self::ELFPH`] with `LOAD` type of the elf file.
    pub fn ph_load(&self) -> impl Iterator<Item = ELFPH> + '_ {
        // Load Elf "LOAD" segments at base_addr.
        self.elf
            .program_iter()
            .filter(|ph| ph.get_type() == Ok(xmas_elf::program::Type::Load))
            .map(|ph| {
                let start_va = ph.virtual_addr() as usize + self.base;
                let start_offset = ph.offset() as usize;
                ELFPH {
                    offset: start_offset,
                    vaddr: start_va,
                    memsz: ph.mem_size(),
                    filesz: ph.file_size(),
                    flags: ph.flags(),
                }
            })
    }
}
