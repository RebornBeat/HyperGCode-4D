# HyperCube-4D Standard Specifications

## Overview
The Standard model targets serious makers and small businesses requiring practical production capabilities. It scales up the Mini's architecture while adding true dual-material support through separated valve channels.

## Core Specifications
**Build Volume**: 200mm × 200mm × 200mm  
**Valve Grid Spacing**: 0.5mm  
**Total Valve Nodes**: 400 × 400 = 160,000 positions  
**Valves Per Node**: 8 (4 per material channel × 2 materials)  
**Total Valve Count**: 1,280,000 valves  
**Layer Resolution**: 0.1mm to 0.4mm  
**Material Channels**: 2 independent (no cross-contamination)

## Frame: 2040 Aluminum Extrusion with Reinforcement
- Vertical posts: 4 × 600mm
- Base/top perimeter: 8 × 400mm each
- Cross-braces: 6 × 400mm diagonal reinforcement
- Total weight capacity: 15kg valve plane assembly
- Enhanced corner brackets with dual-bolt connections

## Valve Plane Assembly
**Dimensions**: 250mm × 250mm × 40mm  
**Weight**: ~10kg with dual-channel routing  
**Thermal Zones**: 4 independent, each 100mm × 100mm  
**Power**: 400W total heating (100W per zone)  
**Material Injection**: 2 points (one per material)  
**Routing Network**: Separated channels prevent material mixing

## Z-Axis: Dual Lead Screw System
- 2 × lead screws, 8mm diameter, 2mm pitch
- Synchronized via GT2 belt from single motor
- 4 × linear rails (12mm diameter, 300mm length)
- Mechanical leveling through independent screw adjustment
- Maximum speed: 15mm/s with 10kg load

## Material System: Dual Independent
Each material channel includes:
- Dedicated direct drive extruder (NEMA 17)
- Independent pressure regulation (0-100 PSI)
- Separate heated manifold (150W capacity)
- Individual flow sensors and pressure monitoring
- Compatible with 1.75mm filament

## Electronics
- Raspberry Pi 4, 8GB RAM
- 80 × valve driver boards (16,000 valves per board)
- SPI communication at 10MHz
- Distributed control: 4 regional processors
- 400Hz update rate for valve states
- Dedicated power: 48V DC @ 20A

## Estimated Cost: $12,500
- Frame and motion: $400
- Valve arrays: $8,000 (economies of scale reduce per-valve cost)
- Electronics: $2,500 (distributed control architecture)
- Dual material system: $800
- Build systems: $300
- Miscellaneous: $500

---
