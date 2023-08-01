Title: Tesla Coil: Driver Circuit
Date: 2022-02-24

Driver circuit is based off [Zach Armstrong's](https://hackaday.io/project/182391-the-easy-powerful-solid-state-tesla-coil) DRSSTC driver (which itself is based off [Loneoceans's SSTC 2](https://www.loneoceans.com/labs/sstc2/sstc2schematicv10.jpg)).  I've redrawn the schematic in KiCAD.  I altered the switch to turn off the internal interrupter when the external interrupter is enabled.  I reused the GDT footprint off Zach's design.  I changed the PCB design to remove all direct wire-to-board connectors from the original PCB.  I added room for barrier blocks and terminal blocks.  The PCB also removes the fly wire from the original design

<model-viewer alt="driver render" src="{attach}models/tesla-coil-driver-model.glb" poster="{attach}images/tesla-coil-driver-model-poster.png" camera-controls></model-viewer>

Next step is to complete the coil calculations