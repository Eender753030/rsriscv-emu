mod instruction;
mod exception;
mod mid;

use ratatui::Frame;
use ratatui::layout::Rect;

use crate::state::EmuState;

pub use instruction::Instruction;
pub use exception::Exception;
#[cfg(feature = "zicsr")]
pub use mid::csr::Csr;
pub use mid::register::Register;

pub trait Component {
    fn render(f: &mut Frame, area: Rect, emu: &mut EmuState);
}
