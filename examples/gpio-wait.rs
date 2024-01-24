use std::error::Error;
use std::time::Duration;

use embedded_hal::digital::{InputPin, OutputPin, PinState};
use embedded_hal_async::digital::Wait;
use gpio_cdev::{Chip, LineRequestFlags};
use linux_embedded_hal::CdevPin;
use tokio::time::{sleep, timeout};

// This example assumes that input/output pins are shorted.
const INPUT_LINE: u32 = 4;
const OUTPUT_LINE: u32 = 17;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut chip = Chip::new("/dev/gpiochip0")?;
    let input = chip.get_line(INPUT_LINE)?;
    let output = chip.get_line(OUTPUT_LINE)?;

    let mut input_pin =
        CdevPin::new(input.request(LineRequestFlags::INPUT, 0, "")?)?.into_input_pin()?;
    let mut output_pin = CdevPin::new(output.request(LineRequestFlags::OUTPUT, 0, "")?)?
        .into_output_pin(PinState::Low)?;

    timeout(Duration::from_secs(10), async move {
        let set_output = tokio::spawn(async move {
            sleep(Duration::from_secs(5)).await;
            println!("Setting output high.");
            output_pin.set_high()
        });

        println!("Waiting for input to go high.");

        input_pin.wait_for_high().await?;

        assert!(input_pin.is_high()?);
        println!("Input is now high.");

        set_output.await??;

        Ok::<_, Box<dyn Error>>(())
    })
    .await??;

    Ok(())
}
