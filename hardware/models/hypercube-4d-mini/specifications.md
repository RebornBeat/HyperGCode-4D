# HyperCube-4D Mini - Complete Specifications

## Overview

The HyperCube-4D Mini is the entry-level model designed for education, research, and prototyping. This compact system demonstrates the fundamental principles of parallel valve-based deposition while remaining accessible to makers and researchers. The Mini provides a complete working implementation suitable for validating concepts, testing materials, and learning the HyperGCode-4D workflow without the expense and complexity of larger systems.

## Core Specifications

**Build Volume**: 100mm (X) × 100mm (Y) × 150mm (Z)  
**Valve Grid Spacing**: 0.5mm  
**Total Valve Nodes**: 200 × 200 = 40,000 positions  
**Valves Per Node**: 4 (supporting single material or basic dual material)  
**Total Valve Count**: 160,000 valves  
**Layer Resolution**: 0.1mm to 0.4mm  
**Maximum Valve Switching Rate**: 10 Hz per valve  
**Theoretical Print Speed**: Up to 1000× faster than conventional FDM for suitable geometries

## Frame Specifications

**Material**: 2020 Aluminum Extrusion (Type 6, Slot 6)  
**Frame Style**: Cube configuration with vertical posts and horizontal members  
**Dimensions**: 340mm (X) × 340mm (Y) × 500mm (Z) overall external dimensions  
**Weight Capacity**: Designed to support 5kg valve plane assembly

**Required Extrusions**:
- 4 × 500mm vertical posts
- 4 × 300mm base perimeter
- 4 × 300mm top perimeter  
- 4 × 100mm valve plane mounting cross-members
- 2 × 300mm diagonal cross-braces for rigidity

**Corner Connections**: 16 × aluminum corner brackets with M5 hardware  
**Cross-Brace Connections**: 4 × T-slot connectors  
**Leveling**: 4 × adjustable leveling feet with 10mm adjustment range

## Valve Plane Assembly

**Configuration**: Integrated valve plane with embedded routing channels  
**Dimensions**: 150mm × 150mm × 25mm (active area 100mm × 100mm centered)  
**Weight**: Approximately 3kg fully assembled  
**Material**: Aluminum mounting plate with integrated manifold channels

**Valve Array**: 40,000 valve nodes arranged in 200 × 200 grid at 0.5mm spacing. Each node contains four micro-valves for directional material routing. Valves are pneumatically actuated solenoid design with 3mm body size. Response time is under 10ms from command to full actuation.

**Material Distribution**: Single injection point at center of valve plane. Material routes from center injection point through valve network to reach all positions. Embedded channels in mounting plate carry material with 2mm internal diameter. Heated manifold maintains material temperature throughout distribution.

**Thermal System**: Single heating zone covering entire valve plane. Silicone heater pad bonded to underside of mounting plate provides 200W heating capacity. Single thermistor mounted centrally monitors temperature. Thermal insulation on underside reduces heat loss. Operating temperature range is 180°C to 280°C suitable for PLA, PETG, and ABS.

## Z-Axis Mechanism

**Drive System**: Single 8mm lead screw with 2mm pitch  
**Motor**: NEMA 17 stepper motor with 1.8° step angle  
**Linear Motion**: Two 12mm linear rails with LM12LUU bearings  
**Travel Range**: 150mm usable Z height  
**Resolution**: 0.0025mm per full step (200 steps/mm)  
**Maximum Speed**: 10mm/s travel speed  
**Homing**: Mechanical endstop at Z-minimum position

**Mounting**: Z-axis motor mounts to frame base. Lead screw couples directly to motor shaft with rigid coupler. Linear rails mount to vertical frame posts on opposite sides of build volume. Valve plane mounts to carriage plate that travels on linear bearings. Lead screw nut attaches to carriage plate providing vertical drive.

**Leveling**: Manual tramming using adjustment screws at three corners of valve plane mounting. Procedure involves measuring distance from build plate to valve plane at multiple points and adjusting screws to achieve plane parallel to build plate within 0.1mm across entire area.

## Material System

**Extruder**: Single direct drive extruder mounted stationary on frame  
**Motor**: NEMA 17 with 5:1 gearing for adequate torque  
**Filament Path**: 1.75mm filament diameter standard. Filament feeds from spool through guide tube to extruder. Extruder feeds into heated manifold through Bowden tube. Manifold distributes to valve plane injection point.

**Pressure System**: Single pneumatic pressure regulator providing 0-100 PSI adjustable pressure. Compressed air supply required (typical shop compressor adequate). Pressure regulated to 40-60 PSI for typical thermoplastics. Pressure sensor at injection point monitors supply pressure.

**Heating**: Extruder uses standard E3D V6 style hotend capable of 280°C maximum temperature. Manifold heating maintains temperature from extruder to injection point. Valve plane heating maintains material temperature throughout distribution network.

**Material Compatibility**: Compatible with standard 1.75mm thermoplastic filaments including PLA (190-220°C extrusion temperature), PETG (220-250°C), ABS (230-260°C), and TPU (220-240°C with reduced pressure).

## Electronics Architecture

**Main Controller**: Raspberry Pi 4 Model B with 4GB RAM  
**Operating System**: Custom Linux distribution optimized for real-time control  
**Firmware**: HyperGCode-4D firmware running on Raspberry Pi

**Valve Control**: Custom valve driver boards using shift register architecture. Each board controls 4,096 valves (64 × 64 grid section). Ten boards total cover complete 40,000 valve array. Boards connect to Raspberry Pi via SPI interface. Update rate is 100Hz for all valve states.

**Motor Control**: Standard 3D printer stepper driver (A4988 or TMC2209) for Z-axis motor. Driver connects to Raspberry Pi GPIO pins for step/direction control.

**Heater Control**: Solid state relays (SSR) control AC power to heater pads. PWM control from Raspberry Pi through SSR provides temperature regulation. Maximum heater power draw is 250W total.

**Sensor Interface**: Temperature sensors (thermistors) connect to ADC inputs on Raspberry Pi. Pressure sensor outputs analog voltage read by ADC. Endstop switches connect to GPIO pins.

**Power Supply**: 24V DC, 10A switching power supply provides main system power. 5V DC, 3A supply powers Raspberry Pi (separate from main supply). AC power to heaters switched through SSRs.

**Communication**: Ethernet port provides network connectivity. USB ports support flash drives for G-code file transfer. Optional Wi-Fi dongle enables wireless operation. HDMI output connects to monitor for local control interface.

## Control Interface

**Network Interface**: Web-based control accessed through any browser on local network. Interface displays current print status, valve activation visualization, temperature and pressure monitoring, and print progress. Users can start/stop prints, adjust parameters during printing, and access file management.

**File Management**: Upload .hg4d files through web interface or copy to USB drive. File browser shows available files with metadata including estimated print time and material usage. Queue multiple prints for batch operation.

**Monitoring**: Real-time display shows valve plane heat map indicating which valves are currently active. Temperature graphs track all thermal zones over time. Pressure graphs show supply pressure stability. Z-position indicator shows current layer height and progress.

**Emergency Controls**: Large emergency stop button immediately halts all operations. Pause function suspends printing allowing user intervention. Resume function continues from pause point. Cancel function aborts print and returns to safe state.

## Build Plate and Adhesion

**Build Plate**: Aluminum plate 120mm × 120mm × 6mm provides flat, rigid surface. Heated bed with silicone heater pad bonded to underside provides up to 100°C bed temperature. Thermistor embedded in plate monitors temperature. Surface finished with PEI sheet for excellent adhesion without additional adhesives.

**Leveling**: Build plate attaches to frame base through four adjustment screws with springs. Manual leveling procedure uses feeler gauge or paper to set gap between valve plane and build plate. Target gap is 0.2mm across entire plate surface.

**Adhesion Enhancement**: PEI surface provides good adhesion for most materials. Optional glass plate with adhesive (glue stick, hairspray, or dedicated 3D print adhesive) for materials requiring additional bonding. Heated bed improves adhesion and reduces warping for all materials.

## Calibration Procedures

**Z-Axis Calibration**: Move Z-axis through full travel and measure actual displacement. Adjust firmware steps/mm if actual movement differs from commanded. Typical value is 400 steps/mm for 2mm pitch lead screw and 1.8° stepper.

**Valve Timing Calibration**: Command test pattern to small group of valves while observing with high-speed camera or oscilloscope. Measure time from command to full valve opening. Adjust firmware timing parameters to account for actual valve response. Target consistency is within 2ms across all valves in array.

**Pressure Calibration**: Set regulator to specific pressures (20, 40, 60, 80 PSI) and record sensor readings. Create calibration curve mapping sensor voltage to actual pressure. Firmware uses curve to display accurate pressure values.

**Thermal Calibration**: Set hotend and bed to specific temperatures using external thermocouple as reference. Compare reference readings to internal sensor readings. Adjust firmware temperature offsets to match reference. Typical accuracy target is ±5°C.

**Valve Plane Flatness**: Measure distance from build plate to valve plane at nine points across area (corners, midpoints, center). Calculate deviations from average. Adjust mounting screws to minimize deviation. Target flatness is ±0.1mm across full area.

## Initial Setup and Testing

**Frame Assembly**: Assemble frame following provided diagrams. Ensure all extrusions are seated fully in corner brackets. Tighten all fasteners. Check frame square by measuring diagonals. Install leveling feet and adjust for stable base.

**Z-Axis Installation**: Mount linear rails to vertical posts ensuring parallel alignment. Install lead screw in motor mount. Attach motor and couple to lead screw. Install carriage on linear bearings. Connect lead screw nut to carriage. Test Z-motion through full range checking for smooth operation without binding.

**Valve Plane Mounting**: Attach valve plane to Z-carriage mounting points. Connect material feed line from extruder to injection point. Connect heated manifold power and sensor wiring. Connect valve driver board communications and power. Verify valve plane moves smoothly with Z-axis.

**Electronics Connection**: Mount Raspberry Pi and driver boards in electronics enclosure. Connect all motors, heaters, sensors, and valve drivers according to wiring diagram. Apply power and verify Raspberry Pi boots successfully. Access web interface and verify all components report correctly.

**Functional Testing**: Command single valve in center of array and verify actuation. Gradually expand test to groups of valves, monitoring pressure stability. Test Z-axis motion through web interface. Test heaters and verify temperature control. Load test file and execute print of simple calibration cube.

## Bill of Materials

### Frame Components
- 12 meters total 2020 aluminum extrusion (various lengths listed above)
- 16 × corner brackets for 2020 extrusion
- 4 × T-slot connectors for cross braces
- 60 × M5×8mm button head screws
- 60 × M5 T-slot nuts
- 4 × leveling feet with M5 threads

### Motion Components  
- 1 × NEMA 17 stepper motor (1.8°, 1.5A or higher)
- 1 × 8mm lead screw, 150mm length, 2mm pitch
- 1 × 5mm to 8mm rigid shaft coupling
- 2 × 12mm linear rail, 200mm length
- 2 × LM12LUU linear bearings
- 1 × aluminum carriage plate 150mm × 150mm × 10mm
- 1 × mechanical endstop switch

### Valve Plane Assembly
- 1 × aluminum mounting plate 150mm × 150mm × 25mm with machined channels
- 40,000 × micro-valves (custom pneumatic solenoid, 3mm body)
- 1 × silicone heater pad, 150mm × 150mm, 200W
- 1 × thermistor, 100K, with thermal epoxy
- 1 × thermal insulation pad, 150mm × 150mm
- Pneumatic fittings and tubing per valve array design
- Material injection fitting (1/8" NPT or push-fit for 4mm tubing)

### Material System
- 1 × E3D V6 hotend assembly with heater and thermistor
- 1 × NEMA 17 stepper motor for extruder
- 1 × direct drive extruder gear assembly (5:1 gearing)
- 1 × Bowden tube, 4mm OD × 2mm ID, 300mm length
- 1 × pneumatic pressure regulator, 0-100 PSI adjustable
- 1 × pressure sensor, 0-100 PSI range, analog output
- Compressed air supply connection (1/4" NPT fitting)

### Build Plate
- 1 × aluminum plate, 120mm × 120mm × 6mm
- 1 × silicone heater pad, 120mm × 120mm, 100W
- 1 × thermistor, 100K
- 1 × PEI sheet, 120mm × 120mm
- 4 × M3×40mm adjustment screws
- 4 × springs for leveling

### Electronics
- 1 × Raspberry Pi 4 Model B, 4GB RAM
- 1 × MicroSD card, 32GB or larger for operating system
- 10 × custom valve driver boards (each controls 64×64 grid)
- 1 × A4988 or TMC2209 stepper driver
- 2 × solid state relay, 25A rating for heater control
- 1 × 24V DC power supply, 10A capacity
- 1 × 5V DC power supply, 3A capacity for Raspberry Pi
- Assorted wiring (24AWG for signals, 18AWG for power)
- Spade connectors, JST connectors, and heat shrink tubing
- 1 × electronics enclosure with ventilation
- 1 × emergency stop button (red mushroom style)
- 1 × power inlet with fuse
- 1 × main power switch

### Miscellaneous
- Assorted M3 and M5 screws for assembly
- Cable management clips and ties
- Thermal paste for heater/sensor mounting
- Threadlocker for critical fasteners
- 1 × spool holder for filament

### Tools Required for Assembly
- Hex key set (2mm, 2.5mm, 3mm, 4mm, 5mm)
- Adjustable wrench for pneumatic fittings
- Wire strippers and crimpers
- Soldering iron for electronics assembly
- Multimeter for electrical testing
- Precision squares for frame alignment
- Feeler gauges for leveling (0.1-1.0mm range)

## Estimated Costs

**Frame and Motion**: $150 (extrusions, hardware, linear components)  
**Valve Array**: $2,000 (valves are the dominant cost component)  
**Electronics**: $400 (Raspberry Pi, custom boards, drivers, power supplies)  
**Material System**: $200 (extruder, hotend, pneumatic components)  
**Build Plate**: $80 (plate, heater, PEI surface)  
**Miscellaneous**: $120 (wiring, fasteners, tools, consumables)

**Total Estimated Cost**: $2,950

This represents component cost for a DIY build. Costs assume valve components can be sourced at reasonable quantities. Actual costs vary based on component availability, bulk purchasing, and whether custom PCBs are manufactured professionally or DIY etched.

## Performance Characteristics

**Typical Layer Time**: 15-30 seconds per layer for normal density objects (compared to 3-10 minutes for conventional FDM). Time per layer is nearly constant regardless of geometry complexity since all points print simultaneously. Only total active area affects layer time, not pattern complexity.

**Material Usage Rate**: Up to 10 grams per minute theoretical maximum across full build area. Practical usage typically 2-5 grams per minute for normal prints. Rate limited by pressure system capacity and valve flow characteristics rather than mechanical motion speed.

**Geometric Accuracy**: ±0.2mm in XY plane determined by valve grid spacing. ±0.1mm in Z axis determined by lead screw precision and layer height settings. Accuracy comparable to conventional FDM printing despite fundamentally different approach.

**Surface Finish**: Quality depends on layer height and valve grid resolution. At 0.2mm layers with 0.5mm grid spacing, surface finish is comparable to conventional FDM at similar settings. Valve-based deposition may show different surface texture characteristics requiring finish optimization.

**Minimum Feature Size**: Limited by valve grid spacing. Single-valve features at 0.5mm represent minimum. Multi-valve features can achieve better effective resolution through partial valve activation strategies.

## Operating Environment

**Temperature Range**: 15°C to 30°C ambient temperature for reliable operation. Electronics and valves rated for this range. Higher temperatures may require additional cooling.

**Humidity**: 20% to 80% relative humidity. Electronics should be protected from condensation. Filaments should be stored dry.

**Power Requirements**: 120V or 240V AC input, 50-60Hz. Maximum power draw 400W with all heaters and valves active. Typical operation 200-300W.

**Compressed Air**: Requires compressed air supply at 80-100 PSI minimum. Typical shop compressor adequate. Air must be clean and dry (filter and desiccant recommended). Consumption approximately 0.5 CFM during active printing.

**Ventilation**: Recommended for materials producing fumes (ABS, Nylon). Enclosure with filtered exhaust preferred for indoor operation with odorous materials. PLA printing typically acceptable without special ventilation.

**Noise Level**: Primary noise sources are valve actuation clicks and air pressure sounds. Typical operation approximately 60-65 dB. Quieter than conventional FDM printers due to elimination of stepper motor whine from X/Y motion.

## Maintenance Schedule

**Daily** (when in use): Check compressed air supply pressure and dry air filter. Verify all temperatures reach targets during heat-up. Check filament path for tangles or jams. Monitor first layer adhesion.

**Weekly**: Clean build plate surface with isopropyl alcohol. Check valve plane for any material accumulation or contamination. Verify Z-axis moves smoothly without unusual sounds. Inspect all visible wiring for damage.

**Monthly**: Lubricate Z-axis lead screw with PTFE grease. Check all frame fasteners for tightness. Clean cooling fans and verify airflow. Back up configuration files and print logs. Test emergency stop function.

**Quarterly**: Replace PTFE tube if showing wear or discoloration. Check valve response time calibration and recalibrate if timing has drifted. Deep clean material path including hotend and manifold. Inspect pneumatic fittings for leaks.

**Annually**: Replace worn valves (identified through health monitoring). Disassemble and inspect hotend for degradation. Check frame alignment and re-tram if necessary. Update firmware to latest stable release. Perform complete calibration sequence.

## Troubleshooting Guide

**Valve not responding**: Check valve driver board connections. Verify air pressure adequate. Test individual valve with diagnostic command. Replace valve if mechanically failed.

**Inconsistent extrusion**: Check material pressure sensor readings. Verify heater temperatures stable. Clean nozzle if partially clogged. Check valve timing calibration.

**Z-axis binding**: Verify linear bearings move freely. Check lead screw alignment. Ensure carriage not catching on frame components. Lubricate as needed.

**Temperature errors**: Check thermistor connections for shorts or opens. Verify heater resistance correct. Ensure adequate cooling for electronics. Recalibrate temperature offsets.

**Layer adhesion problems**: Confirm bed temperature adequate for material. Check bed level (should be flat to 0.1mm). Clean bed surface thoroughly. Adjust Z-offset if first layer too high or too low.

**Pressure instability**: Check compressed air supply capacity. Inspect pneumatic lines for leaks. Verify pressure regulator functioning correctly. Reduce number of simultaneously active valves if exceeding system capacity.

**Software connectivity issues**: Verify network settings and Raspberry Pi network connection. Check SD card not corrupted. Restart printer system and reconnect. Check for firmware crashes in logs.

## Upgrade Paths

The Mini design accommodates several upgrades as users' needs evolve and technology advances:

**Dual Material**: Add second extruder, pressure system, and material channel. Modify valve driver firmware to support material routing. Increases capability for multi-color or multi-material printing.

**Enclosed Build Chamber**: Add acrylic or polycarbonate panels around print area. Add chamber heater for consistent temperature. Improves print quality for temperature-sensitive materials like ABS.

**Higher Resolution Valve Array**: Replace valve plane with 0.25mm spacing model (requires 4× valve count). Improves minimum feature size and surface finish. Requires upgraded electronics to drive additional valves.

**Automatic Bed Leveling**: Add Z-probe and implement automated bed leveling procedure. Improves first layer reliability and consistency.

**Filtration System**: Add HEPA and activated carbon filtration to enclosure exhaust. Enables safer printing of materials with strong odors or potentially harmful fumes.

## Educational Applications

The HyperCube-4D Mini serves as an excellent educational platform for teaching concepts in:

**Manufacturing Technology**: Demonstrates additive manufacturing principles, introduces parallel processing concepts, teaches automation and control systems, and provides hands-on experience with novel manufacturing approaches.

**Control Systems**: Real-time control of thousands of actuators, pressure and temperature regulation loops, sensor fusion and monitoring, and coordination of distributed systems all provide rich examples for control theory education.

**Software Engineering**: Embedded Linux system programming, real-time firmware development, network communication protocols, and user interface design all find practical application in operating the system.

**Mechanical Engineering**: Frame design for rigidity and precision, mechanism design for valve arrays, fluid dynamics in material distribution, and thermal management system design all emerge as practical problems requiring engineering solutions.

## Research Applications

The accessible nature of the Mini makes it suitable for research in:

**Novel Materials**: Testing new thermoplastic formulations, exploring bio-compatible materials for tissue engineering, developing conductive or functional materials, and validating materials with unusual flow or thermal properties.

**Process Optimization**: Studying pressure distribution strategies, exploring valve timing algorithms, investigating thermal management approaches, and developing multi-material interface techniques.

**Design for Parallel Manufacturing**: Creating geometries optimized for parallel deposition, studying how design choices affect print time and quality, and exploring applications that exploit simultaneous layer-wide fabrication.

**Sensor Integration**: Developing in-situ process monitoring, implementing closed-loop control strategies, creating quality validation systems, and exploring machine learning applications in print optimization.

## Conclusion

The HyperCube-4D Mini provides an accessible entry point into valve-based parallel deposition technology. While scaled down from larger models in build volume and valve density, it implements all fundamental principles of the HyperGCode-4D approach. The Mini serves as both a practical tool for printing small objects and a learning platform for understanding this novel manufacturing paradigm.

Successful builds of the Mini provide experience that translates directly to larger systems while requiring modest investment in materials and components. The open design encourages experimentation and customization, enabling users to explore variations in valve technology, control strategies, and applications while contributing to the collective understanding of this emerging field.
