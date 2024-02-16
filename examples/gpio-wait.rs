use std::error::Error;
use std::time::Duration;

use embedded_hal::digital::{InputPin, OutputPin, PinState};
use embedded_hal_async::digital::Wait;
use linux_embedded_hal::CdevPin;
use tokio::time::{sleep, timeout};

// This example assumes that input/output pins are shorted.
const CHIP: &str = "/dev/gpiochip0";
const INPUT_LINE: u32 = 4;
const OUTPUT_LINE: u32 = 17;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut input_pin = CdevPin::new_input(CHIP, INPUT_LINE)?;
    let mut output_pin = CdevPin::new_output(CHIP, OUTPUT_LINE, PinState::Low)?;

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
