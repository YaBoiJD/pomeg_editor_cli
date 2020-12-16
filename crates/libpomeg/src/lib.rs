use crate::checksum::is_valid_checksum;
use byteorder::{ByteOrder, LittleEndian};

mod checksum;

#[derive(Debug)]
pub struct Gen3Save {
    save_slot: SaveSlot,
    trainer_id: TrainerID,
}

impl Gen3Save {
    pub fn from_buffer(buffer: &[u8; 0x20000]) -> Self {
        let save_slot = match slot_from_buffer(buffer) {
            Some(s) => s,
            None => panic!("No valid save slot"),
        };

        if !is_valid_checksum(buffer) {
            panic!("Checksum is invalid");
        }

        let trainer_id = TrainerID::from_sector(sector_by_id(save_slot.sector_offset(1), buffer));

        Gen3Save {
            save_slot,
            trainer_id,
        }
    }
}

#[derive(Debug)]
enum SaveSlot {
    A = 0,
    B = 1,
}

impl SaveSlot {
    fn sector_offset(&self, sector_id: u8) -> u8 {
        match self {
            SaveSlot::A => sector_id,
            SaveSlot::B => sector_id + 14,
        }
    }
}

#[derive(Debug)]
struct TrainerID {
    public: u16,
    secret: u16,
}

impl TrainerID {
    fn from_sector(sector: &[u8]) -> Self {
        let public = LittleEndian::read_u16(&sector[0xA..0xC]);
        let secret = LittleEndian::read_u16(&sector[0xD..0xF]);

        TrainerID { public, secret }
    }
}

/// Find out the slot from given buffer
fn slot_from_buffer(buffer: &[u8; 0x20000]) -> Option<SaveSlot> {
    let mut save_index: u32 = get_save_index(sector_by_id(0, buffer));
    let mut save_slot = None;

    for sector_id in 0..27 {
        let retrieved_index = get_save_index(sector_by_id(sector_id, buffer));

        if sector_id == 14 {
            if save_index != std::u32::MAX && retrieved_index < save_index {
                save_slot = Some(SaveSlot::A);
            } else if save_index == std::u32::MAX || retrieved_index > save_index {
                save_slot = Some(SaveSlot::B);
            } else {
                eprintln!("Slot A and B has the same save_index");

                return None;
            }

            save_index = retrieved_index;
        } else if sector_id != 0 && save_index != retrieved_index {
            eprintln!(
                "Sector {} has an invalid save_index, expected \"{}\" but got \"{}\"",
                sector_id, save_index, retrieved_index
            );

            return None;
        }
    }

    save_slot
}

/// Gets the save index from a sector
fn get_save_index(sector: &[u8]) -> u32 {
    LittleEndian::read_u32(&sector[0x0FFC..0x1000])
}

/// Take an buffer and id to return a sector in the form of a slice
///
/// # Panics
///
/// Panics when `sector_id > 31`
fn sector_by_id(sector_id: u8, buffer: &[u8]) -> &[u8] {
    if sector_id > 31 {
        panic!("Sector must be between 0 and 31")
    }

    let offset: usize = (sector_id as usize) << 12;

    &buffer[offset..offset + 0x1000]
}
