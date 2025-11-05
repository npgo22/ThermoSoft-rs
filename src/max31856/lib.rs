use crate::max31856::registers::*;
use embedded_hal::digital::InputPin;
use embedded_hal::spi::SpiDevice;

#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct FaultStatus {
    pub cj_range: bool, // Cold-Junction Out-of-Range
    pub tc_range: bool, // Thermocouple Out-of-Range
    pub cj_high: bool,  // Cold-Junction High Fault
    pub cj_low: bool,   // Cold-Junction Low Fault
    pub tc_high: bool,  // Thermocouple Temperature High Fault
    pub tc_low: bool,   // Thermocouple Temperature Low Fault
    pub ovuv: bool,     // Overvoltage or Undervoltage Input Fault
    pub open: bool,     // Thermocouple Open-Circuit Fault
}

impl FaultStatus {
    pub fn from_register(reg: u8) -> Self {
        Self {
            cj_range: (reg & SR_CJ_RANGE) != 0,
            tc_range: (reg & SR_TC_RANGE) != 0,
            cj_high: (reg & SR_CJ_HIGH) != 0,
            cj_low: (reg & SR_CJ_LOW) != 0,
            tc_high: (reg & SR_TC_HIGH) != 0,
            tc_low: (reg & SR_TC_LOW) != 0,
            ovuv: (reg & SR_OVUV) != 0,
            open: (reg & SR_OPEN) != 0,
        }
    }

    pub fn has_fault(&self) -> bool {
        self.cj_range
            || self.tc_range
            || self.cj_high
            || self.cj_low
            || self.tc_high
            || self.tc_low
            || self.ovuv
            || self.open
    }
}

pub fn read_fault_status<SPI>(spi: &mut SPI) -> Result<FaultStatus, SPI::Error>
where
    SPI: SpiDevice,
{
    let mut buffer = [0u8; 2];
    buffer[0] = SR_READ;

    spi.transfer_in_place(&mut buffer)?;

    Ok(FaultStatus::from_register(buffer[1]))
}

pub fn clear_faults<SPI>(spi: &mut SPI) -> Result<(), SPI::Error>
where
    SPI: SpiDevice,
{
    // Read current CR0 register
    let mut buffer = [0u8; 2];
    buffer[0] = 0x00; // CR0 read address
    spi.transfer_in_place(&mut buffer)?;
    let cr0_current = buffer[1];

    // Set the FAULTCLR bit (bit 1) and write back
    let cr0_with_clear = cr0_current | CR0_FAULTCLR;
    spi.write(&[CR0_WRITE, cr0_with_clear])?;

    // Clear the FAULTCLR bit to return to normal operation
    spi.write(&[CR0_WRITE, cr0_current])?;

    Ok(())
}

/// Read multiple registers for debugging
pub fn read_all_config_registers<SPI>(spi: &mut SPI) -> Result<[u8; 16], SPI::Error>
where
    SPI: SpiDevice,
{
    let mut result = [0u8; 16];

    for i in 0..16 {
        let mut buffer = [0u8; 2];
        buffer[0] = i;
        spi.transfer_in_place(&mut buffer)?;
        result[i as usize] = buffer[1];
    }

    Ok(result)
}

/// Set Cold-Junction High Fault Threshold (0x03)
/// Temperature in degrees C (signed 8-bit, resolution 1°C)
/// Default: 0x7F (+127°C)
pub fn set_cj_high_fault_threshold<SPI>(spi: &mut SPI, temp_celsius: i8) -> Result<(), SPI::Error>
where
    SPI: SpiDevice,
{
    spi.write(&[CJHF_WRITE, temp_celsius as u8])
}

/// Set Cold-Junction Low Fault Threshold (0x04)
/// Temperature in degrees C (signed 8-bit, resolution 1°C)
/// Default: 0xC0 (-64°C)
pub fn set_cj_low_fault_threshold<SPI>(spi: &mut SPI, temp_celsius: i8) -> Result<(), SPI::Error>
where
    SPI: SpiDevice,
{
    spi.write(&[CJLF_WRITE, temp_celsius as u8])
}

/// Set Linearized Temperature High Fault Threshold (0x05-0x06)
/// Temperature in degrees C (signed 16-bit, resolution 0.0625°C)
/// Default: 0x7FFF (+2047.9375°C)
pub fn set_tc_high_fault_threshold<SPI>(spi: &mut SPI, temp_celsius: f32) -> Result<(), SPI::Error>
where
    SPI: SpiDevice,
{
    // Convert temperature to 16-bit value (resolution 0.0625°C)
    let temp_raw = (temp_celsius / 0.0625) as i16;
    let msb = (temp_raw >> 8) as u8;
    let lsb = temp_raw as u8;

    spi.write(&[LTHFTH_WRITE, msb])?;
    spi.write(&[LTHFTL_WRITE, lsb])
}

/// Set Linearized Temperature Low Fault Threshold (0x07-0x08)
/// Temperature in degrees C (signed 16-bit, resolution 0.0625°C)
/// Default: 0x8000 (-2048°C)
pub fn set_tc_low_fault_threshold<SPI>(spi: &mut SPI, temp_celsius: f32) -> Result<(), SPI::Error>
where
    SPI: SpiDevice,
{
    // Convert temperature to 16-bit value (resolution 0.0625°C)
    let temp_raw = (temp_celsius / 0.0625) as i16;
    let msb = (temp_raw >> 8) as u8;
    let lsb = temp_raw as u8;

    spi.write(&[LTLFTH_WRITE, msb])?;
    spi.write(&[LTLFTL_WRITE, lsb])
}

/// Set Cold-Junction Temperature Offset (0x09)
/// Offset in degrees C (signed 8-bit, resolution 0.0625°C)
/// Default: 0x00 (0°C offset)
/// This is used to compensate for any temperature gradient between
/// the MAX31856 and the thermocouple cold junction
pub fn set_cj_temp_offset<SPI>(spi: &mut SPI, offset_celsius: f32) -> Result<(), SPI::Error>
where
    SPI: SpiDevice,
{
    // Convert offset to 8-bit value (resolution 0.0625°C, but stored as 4-bit fractional)
    let offset_raw = (offset_celsius * 16.0) as i8;
    spi.write(&[CJTO_WRITE, offset_raw as u8])
}

pub async fn read_thermocouple_with_fault_check<SPI, FAULT, DRDY>(
    spi: &mut SPI,
    _fault_pin: &mut FAULT,
    _drdy_pin: &mut DRDY, // Unused in INTERRUPT mode
) -> (i32, Option<FaultStatus>)
where
    SPI: SpiDevice,
    FAULT: InputPin,
    DRDY: InputPin,
{
    // First check fault status before reading temperature
    let mut fault_buffer = [0u8; 2];
    fault_buffer[0] = SR_READ;
    let fault_status = if spi.transfer_in_place(&mut fault_buffer).is_ok() {
        let status = FaultStatus::from_register(fault_buffer[1]);

        if status.has_fault() {
            // Clear the faults
            let _ = clear_faults(spi);
            Some(status)
        } else {
            None
        }
    } else {
        None
    };

    // If there's a fault, return 0 for temperature
    if fault_status.is_some() {
        return (0, fault_status);
    }

    // Read 3 bytes of temperature data starting from LTCBH
    let mut buffer = [0u8; 4];
    buffer[0] = LTCBH_READ;

    let temp_counts = if spi.transfer_in_place(&mut buffer).is_ok() {
        // The data format is: [raw_val[0] << 16] | [raw_val[1] << 8] | [raw_val[2]]
        // Then shift right by 5 to get the 19-bit value
        let raw_val_signed =
            ((buffer[1] as i32) << 16) | ((buffer[2] as i32) << 8) | (buffer[3] as i32);
        raw_val_signed >> 5
    } else {
        0 // Return 0 if SPI read fails
    };

    (temp_counts, fault_status)
}
