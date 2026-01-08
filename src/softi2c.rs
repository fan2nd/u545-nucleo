//! Software I2C implementation using embassy-time and embassy-stm32 Flex pins
//!
use embassy_stm32::Peri;
use embassy_stm32::gpio::{Flex, Level, Output, Pin, Pull, Speed};
use embassy_time::{Duration, block_for};
use embedded_hal::i2c::{ErrorType, I2c, Operation};
use heapless::Vec;

/// Software I2C implementation using Flex pins for bidirectional SDA
pub struct SoftI2c<'d> {
    scl: Output<'d>,
    sda: Flex<'d>,
    delay: Duration,
}

impl<'d> SoftI2c<'d> {
    /// Create a new software I2C instance
    pub fn new(scl: Peri<'d, impl Pin>, sda: Peri<'d, impl Pin>, frequency_khz: usize) -> Self {
        let scl = Output::new(scl, Level::High, Speed::High);
        let mut sda = Flex::new(sda);
        sda.set_as_input_output_pull(Speed::High, Pull::Up);

        // Calculate delay from frequency (half period for each edge)
        let period_us = 1000 / frequency_khz; // Convert kHz to period in microseconds
        let half_period_us = period_us / 2;

        Self {
            scl,
            sda,
            delay: Duration::from_micros(half_period_us as u64),
        }
    }

    /// Send start condition
    fn start(&mut self) {
        // SDA high to low while SCL is high
        self.sda.set_high();
        self.scl.set_high();
        block_for(self.delay);

        self.sda.set_low();
        block_for(self.delay);

        self.scl.set_low();
        block_for(self.delay);
    }

    /// Send stop condition
    fn stop(&mut self) {
        // SDA low to high while SCL is high
        self.sda.set_low();
        self.scl.set_low();
        block_for(self.delay);

        self.scl.set_high();
        block_for(self.delay);

        self.sda.set_high();
        block_for(self.delay);

        // Additional delay after stop condition
        block_for(self.delay);
    }

    /// Write a byte and check for ACK
    fn write_byte(&mut self, byte: u8) -> bool {
        // Ensure SDA is in output mode for writing
        self.sda.set_as_output(Speed::Low);

        for i in 0..8 {
            // Set SDA according to bit value
            if (byte & (1 << (7 - i))) != 0 {
                self.sda.set_high();
            } else {
                self.sda.set_low();
            }

            block_for(self.delay);

            // Clock pulse
            self.scl.set_high();
            block_for(self.delay);
            self.scl.set_low();
            block_for(self.delay);
        }

        // Release SDA for ACK and switch to input mode
        self.sda.set_high();
        self.sda.set_as_input_output_pull(Speed::High, Pull::Up);
        block_for(self.delay);

        // Clock for ACK
        self.scl.set_high();
        block_for(self.delay);

        // Read ACK bit
        let ack = !self.sda.is_high();

        self.scl.set_low();
        block_for(self.delay);

        ack
    }

    /// Read a byte and send ACK/NACK
    fn read_byte(&mut self, ack: bool) -> u8 {
        let mut byte = 0u8;

        // Ensure SDA is in input mode for reading
        self.sda.set_as_input_output_pull(Speed::High, Pull::Up);

        for i in 0..8 {
            self.scl.set_high();
            block_for(self.delay);

            // Read bit
            if self.sda.is_high() {
                byte |= 1 << (7 - i);
            }

            self.scl.set_low();
            block_for(self.delay);
        }

        // Switch to output mode for sending ACK/NACK
        self.sda.set_as_output(Speed::Low);

        // Send ACK/NACK
        if ack {
            self.sda.set_low();
        } else {
            self.sda.set_high();
        }

        block_for(self.delay);

        self.scl.set_high();
        block_for(self.delay);
        self.scl.set_low();
        block_for(self.delay);

        // Release SDA
        self.sda.set_high();

        byte
    }

    /// Write data to I2C device
    pub fn write(&mut self, addr: u8, data: &[u8]) -> Result<(), Error> {
        self.start();

        // Send address with write bit (0)
        if !self.write_byte(addr << 1) {
            return Err(Error::NoAck);
        }

        // Send data
        for &byte in data {
            if !self.write_byte(byte) {
                return Err(Error::NoAck);
            }
        }

        self.stop();
        Ok(())
    }

    /// Read data from I2C device
    pub fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Error> {
        self.start();

        // Send address with read bit (1)
        if !self.write_byte((addr << 1) | 1) {
            return Err(Error::NoAck);
        }

        // Read data
        let len = buffer.len();
        for i in 0..len {
            let ack = i < len - 1; // Send ACK for all but last byte
            buffer[i] = self.read_byte(ack);
        }

        self.stop();
        Ok(())
    }

    /// Write then read in one transaction (repeated start)
    pub fn write_read(
        &mut self,
        addr: u8,
        write_data: &[u8],
        read_buffer: &mut [u8],
    ) -> Result<(), Error> {
        // Write phase
        self.start();

        // Send address with write bit (0)
        if !self.write_byte(addr << 1) {
            return Err(Error::NoAck);
        }

        // Send write data
        for &byte in write_data {
            if !self.write_byte(byte) {
                return Err(Error::NoAck);
            }
        }

        // Repeated start for read
        self.start();

        // Send address with read bit (1)
        if !self.write_byte((addr << 1) | 1) {
            return Err(Error::NoAck);
        }

        // Read data
        let len = read_buffer.len();
        for i in 0..len {
            let ack = i < len - 1; // Send ACK for all but last byte
            read_buffer[i] = self.read_byte(ack);
        }

        self.stop();
        Ok(())
    }
}

/// I2C error types
#[derive(Debug, defmt::Format)]
pub enum Error {
    NoAck,
    BusError,
    Timeout,
}

impl embedded_hal::i2c::Error for Error {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        match self {
            Error::NoAck => embedded_hal::i2c::ErrorKind::NoAcknowledge(
                embedded_hal::i2c::NoAcknowledgeSource::Unknown,
            ),
            Error::BusError => embedded_hal::i2c::ErrorKind::Bus,
            Error::Timeout => embedded_hal::i2c::ErrorKind::Other,
        }
    }
}

impl ErrorType for SoftI2c<'_> {
    type Error = Error;
}

impl I2c for SoftI2c<'_> {
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.read(address, buffer)
    }

    fn write(&mut self, address: u8, data: &[u8]) -> Result<(), Self::Error> {
        self.write(address, data)
    }

    fn write_read(
        &mut self,
        address: u8,
        write_data: &[u8],
        read_buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.write_read(address, write_data, read_buffer)
    }

    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.start();

        // Send address with write bit (0)
        if !self.write_byte(address << 1) {
            return Err(Error::NoAck);
        }

        for operation in operations {
            match operation {
                Operation::Write(write_data) => {
                    for &byte in *write_data {
                        if !self.write_byte(byte) {
                            return Err(Error::NoAck);
                        }
                    }
                }
                Operation::Read(read_buffer) => {
                    // Repeated start for read
                    self.start();

                    // Send address with read bit (1)
                    if !self.write_byte((address << 1) | 1) {
                        return Err(Error::NoAck);
                    }

                    // Read data
                    let len = read_buffer.len();
                    for i in 0..len {
                        let ack = i < len - 1; // Send ACK for all but last byte
                        read_buffer[i] = self.read_byte(ack);
                    }
                }
            }
        }

        self.stop();
        Ok(())
    }
}

pub fn i2c_scan<T: I2c>(i2c: &mut T) -> Vec<u8, 128> {
    let mut addr_set = Vec::new();
    for addr in 0..128 {
        // Skip reserved addresses (0x00-0x07 and 0x78-0x7F)
        if (addr <= 0x07) || (0x78..=0x7F).contains(&addr) {
            continue;
        }

        // Try simple write operation first (more reliable than transaction)
        let write_response = i2c.write(addr, &[]);

        // Only add address if write operation succeeds
        if write_response.is_ok() {
            addr_set.push(addr).unwrap();
        }
    }
    addr_set
}
