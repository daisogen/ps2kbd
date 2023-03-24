#![no_std]
#![no_main]

use core::arch::asm;

const KEYBOARD_IRQ: u8 = 1;
const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;
const COMMAND_PORT: u16 = 0x64;

macro_rules! wait_input_buffer_empty {
    () => {
        while (std::asm::in8(STATUS_PORT) & 2 != 0) {
            unsafe {
                asm!("pause");
            }
        }
    };
}

#[no_mangle]
fn main() {
    // TODO: Check keyboard is present in ACPI

    // Redirect the legacy IRQ
    let (gsi, vec) = std::daisogen::ioapic_redirect_irq(KEYBOARD_IRQ);

    // Enable keyboard
    wait_input_buffer_empty!();
    std::asm::out8(COMMAND_PORT, 0xAE); // COMMAND_ENABLE_FIRST
    wait_input_buffer_empty!();
    std::asm::out8(COMMAND_PORT, 0xA8); // COMMAND_ENABLE_SECOND

    // Set the handler for the vector
    std::daisogen::set_simple_vector(vec, handler as u64);

    // Discard pressed keys
    std::asm::in8(STATUS_PORT);
    std::asm::in8(DATA_PORT);

    // All set
    std::daisogen::unmask(gsi);
    std::daisogen::yld();
}

extern "C" fn handler() {
    // Handle key presses
    loop {
        let status = std::asm::in8(STATUS_PORT);
        if status & 0x01 == 0 {
            // Nothing to do
            break;
        }

        // Key press!
        let keycode = std::asm::in8(DATA_PORT);
        std::daisogen::pd_call1("kbd_buffer_keycode", keycode as u64);
    }

    std::daisogen::eoi();
}
