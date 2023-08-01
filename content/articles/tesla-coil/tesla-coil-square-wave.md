Title: Tesla Coil: Square Wave MIDI Synthesizer
Date: 2022-10-01
Status: draft

I'm getting a bit ahead of myself with this, but I'm a software guy and I need to write some software.  At some point, I want an external interrupter to use the coil as a musical instrument.  I need to create a MIDI instrument that produces clean square waves.  I'll be targetting an Atmel chip in the future, so I'm using the wonderful [avr-hal](https://github.com/Rahix/avr-hal) project by Rahix to provide hardware abstraction.  The formula for deriving frequency given MIDI note number is as follows:

<math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
 <semantics>
  <mrow>
   <mi mathvariant="italic">frequency</mi>
   <mo stretchy="false">=</mo>
   <mrow>
    <mn>440</mn>
    <mo stretchy="false">×</mo>
    <msup>
     <mn>2</mn>
     <mfrac>
      <mrow>
       <mi>n</mi>
       <mo stretchy="false">−</mo>
       <mn>69</mn>
      </mrow>
      <mn>12</mn>
     </mfrac>
    </msup>
   </mrow>
  </mrow>
 </semantics>
</math>