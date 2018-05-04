use super::Native;
use super::Opcode;
use super::Public;
use super::super::util::ReadByteString;
use byteorder::{LittleEndian, ReadBytesExt};
use failure::{Error, ResultExt};
use std::ffi::CString;
use std::io::Cursor;
use std::str;

#[derive(Debug, Fail)]
enum AmxParseError {
    #[fail(display = "Invalid amx magic, expected: 0x{:X}, got: 0x{:X}", _0, _1)]
    InvalidMagic(u16, u16),
    #[fail(display = "Invalid file version, expected: {}, got: {}", _0, _1)]
    InvalidFileVersion(u8, u8),
    #[fail(display = "Invalid amx version, expected: {}, got: {}", _0, _1)]
    InvalidAmxVersion(u8, u8),
}

#[derive(Debug, PartialEq)]
pub struct Plugin {
    flags: u16,
    defsize: u16,
    cod: usize,
    dat: usize,
    hea: usize,
    stp: usize,
    cip: usize,
    publics: usize,
    natives: usize,
    libraries: usize,
    pubvars: usize,
    tags: usize,
    nametable: usize,
    pub bin: Vec<u8>,
}

const AMXMOD_MAGIC: u16 = 0xF1E0;
const FILE_VERSION: u8 = 8;
const AMX_VERSION: u8 = 8;
pub const CELLSIZE: usize = 4;

impl Plugin {
    pub fn from(bin: Vec<u8>) -> Result<Plugin, Error> {
        let mut reader = Cursor::new(&bin);

        {
            let size = reader.read_u32::<LittleEndian>().context("EOF on amx size")?;
            trace!("size:\t{}", size);
        }

        // Magic
        {
            // TODO: test
            let magic = reader.read_u16::<LittleEndian>().context(
                "EOF on amx magic",
            )?;
            if magic != AMXMOD_MAGIC {
                Err(AmxParseError::InvalidMagic(AMXMOD_MAGIC, magic))?;
            }
            trace!("magic:\t0x{:X}", magic);
        }

        // File version
        {
            // TODO: test
            let file_version = reader.read_u8().context("EOF on amx file version")?;
            if file_version != FILE_VERSION {
                Err(AmxParseError::InvalidFileVersion(
                    FILE_VERSION,
                    file_version,
                ))?;
            }
            trace!("file version {}", file_version);
        }

        // Amx version
        {
            // TODO: Test incorrect
            let amx_version = reader.read_u8().context("EOF on amx version")?;
            if amx_version != AMX_VERSION {
                Err(AmxParseError::InvalidAmxVersion(AMX_VERSION, amx_version))?;
            }
            trace!("amx version:\t{}", amx_version);
        }

        // TODO: Parse flags
        let flags = reader.read_u16::<LittleEndian>().context(
            "EOF on amx flags",
        )?;
        trace!("flags:\t0x{:X}", flags);

        let defsize = reader.read_u16::<LittleEndian>().context(
            "EOF on amx defsize",
        )?;
        trace!("defsize:\t{}", defsize);

        let cod = reader.read_u32::<LittleEndian>().context("EOF on amx cod")?;
        trace!("cod:\t0x{:X}", cod);

        let dat = reader.read_u32::<LittleEndian>().context("EOF on amx dat")?;
        trace!("dat:\t0x{:X}", dat);

        let hea = reader.read_u32::<LittleEndian>().context("EOF on amx hea")?;
        trace!("hea:\t0x{:X}", hea);

        let stp = reader.read_u32::<LittleEndian>().context("EOF on amx stp")?;
        trace!("stp:\t0x{:X}", stp);

        let cip = reader.read_u32::<LittleEndian>().context("EOF on amx cip")?;
        trace!("cip:\t0x{:X}", cip);

        let publics = reader.read_u32::<LittleEndian>().context(
            "EOF on amx publics",
        )?;
        trace!("publics:\t0x{:X}", publics);

        let natives = reader.read_u32::<LittleEndian>().context(
            "EOF on amx natives",
        )?;
        trace!("natives:\t0x{:X}", natives);

        let libraries = reader.read_u32::<LittleEndian>().context(
            "EOF on amx libraries",
        )?;
        trace!("libraries:\t0x{:X}", libraries);

        let pubvars = reader.read_u32::<LittleEndian>().context(
            "EOF on amx pubvars",
        )?;
        trace!("pubvars:\t0x{:X}", pubvars);

        let tags = reader.read_u32::<LittleEndian>().context("EOF on amx tags")?;
        trace!("tags:\t0x{:X}", tags);

        let nametable = reader.read_u32::<LittleEndian>().context(
            "EOF on amx nametable",
        )?;
        trace!("nametable:\t0x{:X}", nametable);

        Ok(Plugin {
            flags: flags,
            defsize: defsize,
            cod: cod as usize,
            dat: dat as usize,
            hea: hea as usize,
            stp: stp as usize,
            cip: cip as usize,
            publics: publics as usize,
            natives: natives as usize,
            libraries: libraries as usize,
            pubvars: pubvars as usize,
            tags: tags as usize,
            nametable: nametable as usize,
            bin: bin.clone(),
        })
    }

    pub fn cod_slice(&self) -> &[u8] {
        // FIXME: Error handling when cod does not match
        // Calculate from start of next segment
        trace!("---- Slicing cod");
        trace!("cod starts at: {}", self.cod);
        trace!("dat starts at: {}", self.dat);
        let cod_size = self.dat - self.cod;
        trace!("cod size: {}", cod_size);
        trace!("bin size: {}", self.bin.len());
        trace!("final range: {}-{}", self.cod, self.cod + cod_size);
        &self.bin[self.cod..(self.cod + cod_size)]
    }

    pub fn opcodes(&self) -> Result<Vec<Opcode>, &str> {
        let mut cod_reader = Cursor::new(self.cod_slice());
        let mut opcodes: Vec<Opcode> = Vec::new();

        // FIXME: Error handling
        // Skip first two opcodes for some reason
        cod_reader.read_u32::<LittleEndian>().unwrap();
        cod_reader.read_u32::<LittleEndian>().unwrap();

        loop {
            match Opcode::read_from(&mut cod_reader) {
                // FIXME: Test all cases
                Ok(Some(o)) => opcodes.extend(o),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(opcodes)
    }

    pub fn natives(&self) -> Vec<Native> {
        let slice = &self.bin[self.natives..self.libraries];
        slice.chunks(8) // Take natives by native struct
           .map(|n_struct| {
               // FIXME: Error handling
               let mut address = &n_struct[0..4];
               let address = address.read_u32::<LittleEndian>().unwrap() as usize;
               let mut name_offset = &n_struct[4..8];
               let name_offset = name_offset.read_u32::<LittleEndian>().unwrap() as usize;
               let name = self.bin[name_offset..].read_string_zero().unwrap();

               Native {
                   name: name,
                   address: address,
               }
           }).collect()
    }

    pub fn publics(&self) -> Vec<Public> {
        let slice = &self.bin[self.publics..self.natives];
        slice.chunks(8) // Take natives by native struct
           .map(|n_struct| {
               // FIXME: Error handling
               let mut address = &n_struct[0..4];
               let address = address.read_u32::<LittleEndian>().unwrap() as usize;
               let mut name_offset = &n_struct[4..8];
               let name_offset = name_offset.read_u32::<LittleEndian>().unwrap() as usize;
               let name = self.bin[name_offset..].read_string_zero().unwrap();

               Public {
                   name: name,
                   address: address,
               }
           }).collect()
    }

    fn dat_size(&self) -> usize {
        self.hea - self.dat
    }

    fn dat_slice(&self) -> &[u8] {
        &self.bin[self.dat..(self.dat + self.dat_size())]
    }

    fn is_addr_in_dat(&self, addr: usize) -> bool {
        addr <= self.dat_size()
    }

    pub fn read_constant_auto_type(&self, addr: usize) -> Result<CString, &str> {
        if !self.is_addr_in_dat(addr) {
            return Err("Invalid constant addr");
        }

        let byte_slice: Vec<u8> = self.dat_slice()[addr..]
            .chunks(CELLSIZE)
            .map(|x| x[0])
            .take_while(|&x| x != 0)
            .collect();

        Ok(CString::new(byte_slice).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::Native;
    use super::Plugin;
    use super::Public;
    use std::ffi::CString;
    use std::fs::File;
    use std::io::prelude::*;

    // TODO: Support amx extraction in programm itself
    // fn extract_section_to_file(amxmodx_bin: &[u8], section_number: usize) {
    //     use super::super::super::amxmodx::File as AmxxFile;
    //     use std::fs::File;
    //     use std::io::prelude::*;
    //
    //     let amxmodx_plugin = AmxxFile::from(&amxmodx_bin).unwrap();
    //     let sections = amxmodx_plugin.sections().unwrap();
    //     let amxmod_plugin = sections[section_number]
    //         .unpack_section(&amxmodx_bin)
    //         .unwrap();
    //
    //     let mut file = File::create("unpacked.amx").unwrap();
    //     file.write_all(&amxmod_plugin.bin).unwrap();
    // }

    fn load_fixture(filename: &str) -> Vec<u8> {
        let mut file_bin: Vec<u8> = Vec::new();
        {
            let mut file = File::open(format!("test/fixtures/{}", filename)).unwrap();
            file.read_to_end(&mut file_bin).unwrap();
        }

        file_bin
    }

    #[test]
    fn it_load_plugins_when_it_is_correct() {
        let amxmod_bin = load_fixture("simple.amx183");
        let extracted_plugin = Plugin::from(amxmod_bin.clone()).unwrap();
        let expected_plugin = Plugin {
            flags: 2,
            defsize: 8,
            cod: 116,
            dat: 192,
            hea: 296,
            stp: 16680,
            cip: 4294967295,
            publics: 56,
            natives: 64,
            libraries: 72,
            pubvars: 72,
            tags: 72,
            nametable: 80,
            bin: amxmod_bin.to_vec(),
        };
        assert_eq!(extracted_plugin, expected_plugin);
    }

    #[test]
    fn it_read_opcodes() {
        let amxmod_bin = load_fixture("simple.amx183");
        let amxmod_plugin = Plugin::from(amxmod_bin).unwrap();
        amxmod_plugin.opcodes().unwrap();
    }

    #[test]
    fn it_read_natives() {
        let amxmod_bin = load_fixture("two_natives.amx183");
        let amxmod_plugin = Plugin::from(amxmod_bin).unwrap();
        let natives = amxmod_plugin.natives();
        let expected_natives = [
            Native {
                name: CString::new("native_one").unwrap(),
                address: 0,
            },
            Native {
                name: CString::new("native_two").unwrap(),
                address: 0,
            },
        ];

        assert_eq!(natives, expected_natives);
    }

    #[test]
    fn it_read_publics() {
        let amxmod_bin = load_fixture("two_natives.amx183");
        let amxmod_plugin = Plugin::from(amxmod_bin).unwrap();
        let publics = amxmod_plugin.publics();
        let expected_publics = [
            Public {
                name: CString::new("func").unwrap(),
                address: 8,
            },
        ];

        assert_eq!(publics, expected_publics);
    }

    #[test]
    fn it_read_constant_by_addr() {
        let amxmod_bin = load_fixture("cell_constants.amx183");
        let amx_plugin = Plugin::from(amxmod_bin).unwrap();
        let resp = amx_plugin.read_constant_auto_type(0);
        assert_eq!("simple plugin", resp.unwrap().into_string().unwrap());
    }
}
