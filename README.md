# ThermoBoard DAQ Upgrade Software

This is the software for APRL's Thermocouple DAQ Board.

## Status
Open fault detection is, for whatever reason, not reliable. The offset between TC (Thermocouple) and CJ (cold junction) is not set yet.

## Usage Notes

### Error states
During a sensor fault, the firmware will simply return a reading of 0 for that thermocouple. It will keep trying to clear the state during every supposed reading, which will make the fault LED blink under normal operating conditions instead of holding steady. Because of the high impedance nature of this chip's inputs, you may have to literally tap on the inputs to get the fault led to blink consistently.

RCC is enabled in case of HSE crystal failure.

### LEDs
| LED | Usage |
|-----|-----|
| PC7 | Link status (Blinks when looking) |
| PC8 | Data sent over ethernet |
| PC9 | UDP send failure |

### Lead resistance
Theoretically the per-lead resistance maximum of the MAX31856 is 40k. Currently, it is set in firmware to trigger with a lead resistance less than 5k. If lead resistance is less than 5k, a fault state may be triggered.

### Sample Rate
Currently, the sample rate per sensor is set to ~>5Hz. This achieves the desired overall sample rate of 20Hz. You can calculate it with 1000ms / (90ms + (AVG_TC_SAMPLES - 1) + 33.33). 

### Thermocouple Type
This board can accomodate any type of thermocouple you could ever want.

## TODO
CAN-FD, Decoupling MAX31856 library.

### Additional Notes
Currently the chip itself does some basic supersampling. To improve sample rate, however, it may be a good idea to have the sensor send data at every possible opportunity that it can, then doing an actual true FIR filter on the H5. The FMAC is enabled on this chip just in case, however, the FMAC is only capable of doing fixed-point math, and the MAX31856 returns floating point (which isn't actually too computationally expensive to convert between). With this, we can achieve ~11.11Hz per sensor.