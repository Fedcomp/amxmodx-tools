use super::{Plugin, AMXMOD_MAGIC, AMX_VERSION, FILE_VERSION};
use crate::util::TryFrom;
use byteorder::{LittleEndian, ReadBytesExt};
use failure::{Error, ResultExt};
use std::io::Cursor;

#[derive(Debug, Fail)]
enum AmxParseError {
    #[fail(display = "Invalid amx magic, expected: 0x{:X}, got: 0x{:X}", _0, _1)]
    InvalidMagic(u16, u16),
    #[fail(display = "Invalid file version, expected: {}, got: {}", _0, _1)]
    InvalidFileVersion(u8, u8),
    #[fail(display = "Invalid amx version, expected: {}, got: {}", _0, _1)]
    InvalidAmxVersion(u8, u8),
}

impl TryFrom<Vec<u8>> for Plugin {
    type Error = Error;

    fn try_from(bin: Vec<u8>) -> Result<Self, Self::Error> {
        let mut reader = Cursor::new(&bin);

        {
            let size = reader
                .read_u32::<LittleEndian>()
                .context("EOF on amx size")?;
            trace!("size:\t{}", size);
        }

        // Magic
        {
            // TODO: test
            let magic = reader
                .read_u16::<LittleEndian>()
                .context("EOF on amx magic")?;
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
        let flags = reader
            .read_u16::<LittleEndian>()
            .context("EOF on amx flags")?;
        trace!("flags:\t0x{:X}", flags);

        let defsize = reader
            .read_u16::<LittleEndian>()
            .context("EOF on amx defsize")?;
        trace!("defsize:\t{}", defsize);

        let cod = reader
            .read_u32::<LittleEndian>()
            .context("EOF on amx cod")?;
        trace!("cod:\t0x{:X}", cod);

        let dat = reader
            .read_u32::<LittleEndian>()
            .context("EOF on amx dat")?;
        trace!("dat:\t0x{:X}", dat);

        let hea = reader
            .read_u32::<LittleEndian>()
            .context("EOF on amx hea")?;
        trace!("hea:\t0x{:X}", hea);

        let stp = reader
            .read_u32::<LittleEndian>()
            .context("EOF on amx stp")?;
        trace!("stp:\t0x{:X}", stp);

        let cip = reader
            .read_u32::<LittleEndian>()
            .context("EOF on amx cip")?;
        trace!("cip:\t0x{:X}", cip);

        let publics = reader
            .read_u32::<LittleEndian>()
            .context("EOF on amx publics")?;
        trace!("publics:\t0x{:X}", publics);

        let natives = reader
            .read_u32::<LittleEndian>()
            .context("EOF on amx natives")?;
        trace!("natives:\t0x{:X}", natives);

        let libraries = reader
            .read_u32::<LittleEndian>()
            .context("EOF on amx libraries")?;
        trace!("libraries:\t0x{:X}", libraries);

        let pubvars = reader
            .read_u32::<LittleEndian>()
            .context("EOF on amx pubvars")?;
        trace!("pubvars:\t0x{:X}", pubvars);

        let tags = reader
            .read_u32::<LittleEndian>()
            .context("EOF on amx tags")?;
        trace!("tags:\t0x{:X}", tags);

        let nametable = reader
            .read_u32::<LittleEndian>()
            .context("EOF on amx nametable")?;
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
}

#[cfg(test)]
mod tests {
    use super::super::Plugin;
    use super::*;
    use crate::util::tests::load_fixture;

    #[test]
    fn it_load_plugins_when_it_is_correct() {
        let amxmod_bin = load_fixture("simple.amx183");
        let extracted_plugin = Plugin::try_from(amxmod_bin.clone()).unwrap();
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
}
