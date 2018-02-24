use std::io::Cursor;
use byteorder::{ReadBytesExt, LittleEndian};
use std::str;
use super::Section;

pub struct File<'a> {
    pub bin: &'a [u8],
    pub sections: u8,
}

impl<'a> File<'a> {
    const MAGIC: u32 = 0x414d5858;
    const COMPATIBLE_VERSION: u16 = 768;
    const AMXX_HEADER_SIZE: usize = 7;

    pub fn from(bin: &'a [u8]) -> Result<File, &str> {
        let mut reader = Cursor::new(bin);

        // magic
        match reader.read_u32::<LittleEndian>() {
            Ok(magick) => {
                if magick != File::MAGIC {
                    return Err("Invalid file magic");
                }
            }
            Err(_) => return Err("Magic EOF"),
        };

        // version
        match reader.read_u16::<LittleEndian>() {
            Ok(version) => {
                if version != File::COMPATIBLE_VERSION {
                    return Err("Incompatible file version");
                }
            }
            Err(_) => return Err("Version EOF"),
        };

        // sections count
        let sections = match reader.read_u8() {
            Ok(s) => {
                if s < 1 {
                    return Err("Zero sections amount");
                }

                if s > 2 {
                    return Err("More than two sections (malicious file?)");
                }

                s
            }
            Err(_) => return Err("Sections EOF"),
        };

        Ok(File {
            bin: bin,
            sections: sections,
        })
    }

    pub fn sections(&self) -> Result<Vec<Section>, &'a str> {
        let mut sections: Vec<Section> = Vec::new();

        for i in 0..self.sections {
            let section_offset = File::AMXX_HEADER_SIZE + (Section::SIZE * i as usize);
            let section_bin = &self.bin[section_offset..];
            let section = match Section::from(section_bin) {
                Ok(s) => s,
                Err(e) => return Err(e),
            };
            sections.push(section);
        }

        Ok(sections)
    }
}

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use std::fs::File;
    use super::File as AmxmodxFile;
    use super::super::Section;

    fn load_fixture(filename: &str) -> Vec<u8> {
        let mut file_bin: Vec<u8> = Vec::new();
        {
            let mut file = File::open(format!("test/fixtures/{}", filename)).unwrap();
            file.read_to_end(&mut file_bin).unwrap();
        }

        file_bin
    }

    #[test]
    fn it_load_file_when_binary_is_correct() {
        let amxmodx_bin = load_fixture("simple.amxx183");
        assert!(AmxmodxFile::from(&amxmodx_bin).is_ok());
    }

    #[test]
    fn it_return_multiple_sections() {
        let amxmodx_bin = load_fixture("simple.amxx181");
        let amxmodx_file = AmxmodxFile::from(&amxmodx_bin).unwrap();
        let extracted_sections = amxmodx_file.sections().unwrap();
        let expected_sections = [
            Section {
                cellsize: 4,
                disksize: 161,
                imagesize: 288,
                memsize: 16672,
                offset: 41,
            },
            Section {
                cellsize: 8,
                disksize: 177,
                imagesize: 488,
                memsize: 33256,
                offset: 202,
            },
        ];
        assert_eq!(extracted_sections, expected_sections);
    }

    #[test]
    fn it_err_on_empty_file() {
        let amxmodx_bin = vec![];
        let result = AmxmodxFile::from(&amxmodx_bin);
        assert_eq!(result.err().unwrap(), "Magic EOF");
    }

    #[test]
    fn it_err_on_magic_eof() {
        let amxmodx_bin = vec![0, 0, 0];
        let result = AmxmodxFile::from(&amxmodx_bin);
        assert_eq!(result.err().unwrap(), "Magic EOF");
    }

    #[test]
    fn it_err_on_invalid_magic() {
        let amxmodx_bin = vec![0, 0, 0, 0];
        let result = AmxmodxFile::from(&amxmodx_bin);
        assert_eq!(result.err().unwrap(), "Invalid file magic");
    }

    #[test]
    fn it_err_on_version_eof() {
        // Correct magic, incorrect version
        let amxmodx_bin = vec![88, 88, 77, 65, 0];
        let result = AmxmodxFile::from(&amxmodx_bin);
        assert_eq!(result.err().unwrap(), "Version EOF");
    }

    #[test]
    fn it_err_on_incompatible_version() {
        // Correct magic, incorrect version
        let amxmodx_bin = vec![88, 88, 77, 65, 0, 4];
        let result = AmxmodxFile::from(&amxmodx_bin);
        assert_eq!(result.err().unwrap(), "Incompatible file version");
    }

    #[test]
    fn it_err_on_sections_eof() {
        // Correct magic, correct version, no section byte
        let amxmodx_bin = vec![88, 88, 77, 65, 0, 3];
        let result = AmxmodxFile::from(&amxmodx_bin);
        assert_eq!(result.err().unwrap(), "Sections EOF");
    }

    #[test]
    fn it_err_on_zero_sections() {
        // Correct magic, correct version, zero sections
        let amxmodx_bin = vec![88, 88, 77, 65, 0, 3, 0];
        let result = AmxmodxFile::from(&amxmodx_bin);
        assert_eq!(result.err().unwrap(), "Zero sections amount");
    }

    #[test]
    fn it_err_on_more_than_two_sections() {
        // Correct magic, correct version, 3 sections
        let amxmodx_bin = vec![88, 88, 77, 65, 0, 3, 3];
        let result = AmxmodxFile::from(&amxmodx_bin);
        assert_eq!(
            result.err().unwrap(),
            "More than two sections (malicious file?)"
        );
    }

    #[test]
    fn it_err_on_sections_parsing_eof() {
        // Correct magic, correct version, 2 sections, zero section headers
        let amxmodx_bin = vec![88, 88, 77, 65, 0, 3, 2];
        let result = AmxmodxFile::from(&amxmodx_bin).unwrap().sections();
        assert!(result.is_err());
    }
}
