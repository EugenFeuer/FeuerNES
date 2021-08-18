const NES_MAGIC_NUMBER: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;

#[derive(Debug, PartialEq)]
pub enum MirroringType {
    Vertical,
    Horizontal,
    FourScreen,
}

pub struct Cartridge {
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
    pub mapper: u8,
    pub mirroring_type: MirroringType,
}

impl Cartridge {
    pub fn new(raw: &Vec<u8>) -> Result<Self, String> {
        if raw.len() < 8 || &raw[0..4] != NES_MAGIC_NUMBER {
            return Err(String::from("not valid nes cartridge!"));
        }

        let num_of_prg_banks = raw[4] as usize;
        let num_of_chr_banks = raw[5] as usize;
        let ctrl_byte_one = raw[6];
        let ctrl_byte_two = raw[7];

        let size_of_prg_ram_in_8k = raw[8];
        let reserved = raw[9];

        let has_battery_backed_ram = ctrl_byte_one & 0b0000_0010 != 0;
        let has_trainer = ctrl_byte_one & 0b0000_0100 != 0;
        let has_four_scrren_vram_layout = ctrl_byte_one & 0b0000_1000 != 0;

        if ctrl_byte_two & 0b0000_0011 != 0 {
            return Err(String::from("not valid iNES 1.0 cartridge!"));
        }

        if ctrl_byte_two & 0b0000_1100 == 2 {
            return Err(String::from("not support iNES 2.0 cartridge!"));
        }

        let is_vertical_mirroring = ctrl_byte_one & 0b0000_0001 != 0;
        let mirroring_type = match (has_four_scrren_vram_layout, is_vertical_mirroring) {
            (true, _) => MirroringType::FourScreen,
            (false, false) => MirroringType::Horizontal,
            (false, true) => MirroringType::Vertical,
        };

        let mapper = (ctrl_byte_two & 0b1111_0000) | (ctrl_byte_one >> 4);

        let size_of_prg_rom = num_of_prg_banks * PRG_ROM_PAGE_SIZE;
        let size_of_chr_rom = num_of_chr_banks * CHR_ROM_PAGE_SIZE;

        let entry_point_of_prg_rom = 16 + if has_trainer { 512 } else { 0 };
        let entry_point_of_chr_rom = entry_point_of_prg_rom + size_of_prg_rom;

        return Ok(Cartridge {
            prg: raw[entry_point_of_prg_rom..(entry_point_of_prg_rom + size_of_prg_rom)].to_vec(),
            chr: raw[entry_point_of_chr_rom..(entry_point_of_chr_rom + size_of_chr_rom)].to_vec(),
            mapper: mapper,
            mirroring_type: mirroring_type,
        });
    }
}
