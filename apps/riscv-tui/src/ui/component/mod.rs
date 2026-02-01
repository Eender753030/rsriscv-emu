mod instruction;
mod exception;
mod mid;

use ratatui::{
    Frame, 
    layout::Rect,
    style::{Color}
};

use crate::state::EmuState;

pub use instruction::Instruction;
pub use exception::Exception;
#[cfg(feature = "zicsr")]
pub use mid::csr::Csr;
pub use mid::register::Register;

pub(super) const ANTI_FLASH_WHITE: Color = Color::Rgb(242, 242, 242);
pub(super) const BERKELEY_BLUE: Color = Color::Rgb(0, 50, 98);
pub(super) const CALIFORNIA_GOLD: Color = Color::Rgb(253, 181, 21);

pub trait Component {
    fn render(f: &mut Frame, area: Rect, emu: &mut EmuState);
}
