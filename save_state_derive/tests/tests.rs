use save_state::{Savable, SaveError};

#[derive(serde::Serialize, serde::Deserialize)]
struct CpuFlags {
    a: u8,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum HaltMode {
    NotHalting,
    HaltRunInterrupt,
    HaltNoRunInterrupt,
    HaltBug,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GameboyConfig {
    pub is_dmg: bool,
}

#[derive(save_state_derive::Savable, serde::Serialize, serde::Deserialize)]
pub struct Cpu {
    reg_a: u8,
    reg_b: u8,
    reg_c: u8,
    reg_d: u8,
    reg_e: u8,
    reg_h: u8,
    reg_l: u8,
    reg_f: CpuFlags,

    reg_sp: u16,

    reg_pc: u16,

    enable_interrupt_next: bool,
    ime: bool,
    halt_mode: HaltMode,

    config: GameboyConfig,
}

//impl Savable for Cpu {
//    fn save<W: ::std::io::Write>(&self, writer: &mut W) -> Result<(), SaveError> {
//        ::bincode::serialize_into(writer, self)?;
//        Ok(())
//    }
//
//    fn load<R: ::std::io::Read>(&mut self, reader: &mut R) -> Result<(), SaveError> {
//        let obj = ::bincode::deserialize_from(reader)?;
//
//        let _ = ::std::mem::replace(self, obj);
//        Ok(())
//    }
//
//    fn object_size() -> u64 {
//        1
//    }
//
//    fn current_save_size(&self) -> Result<u64, SaveError> {
//        ::bincode::serialized_size(self).map_err(|e| e.into())
//    }
//}

#[test]
fn cpu_savable_test() {
    let cpu = Cpu {
        reg_a: 0,
        reg_b: 0,
        reg_c: 0,
        reg_d: 0,
        reg_e: 0,
        reg_h: 0,
        reg_l: 0,
        reg_f: CpuFlags { a: 0 },
        reg_sp: 0,
        reg_pc: 0,

        enable_interrupt_next: false,
        ime: false,
        halt_mode: HaltMode::NotHalting,

        config: GameboyConfig { is_dmg: false },
    };
}
