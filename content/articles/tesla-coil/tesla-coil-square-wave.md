Title: Tesla Coil: Square Wave MIDI Synthesizer (part 1)
Date: 2022-10-01

I'm getting a bit ahead of myself with this, but I'm a software guy and I need to write some software.  At some point, I want an external interrupter to use the coil as a musical instrument.  I need to create a MIDI instrument that produces clean square waves.  I'll be targetting an ATMEGA328P, so I'm using the wonderful [avr-hal](https://github.com/Rahix/avr-hal) project by Rahix to provide hardware abstraction.  The synthesizer/interrupter will use the ATMEGA's PWM feature to generate a square waveform.  The chip expects specific registers to be loaded with the on/off intervals.  First, though, we must decode MIDI from the serial line.  MIDI is simply serial data that is easily handled by the UART.  We set up an infinite loop to drain the serial buffer into a stack-allocated temporary buffer for `midly` to decode.

    :::rust
    midly::stack_buffer! { struct LocalBuffer([u8; 1024]); }

    #[arduino_hal::entry]
    fn main() -> !
    {
      let mut stream = MidiStream::with_buffer(LocalBuffer::new());

      let dp = arduino_hal::Peripherals::take().unwrap();
      let pins = arduino_hal::pins!(dp);
      let mut serial = arduino_hal::default_serial!(dp, pins, 115200);

      loop
      {
        let chunk = nb::block!(serial.read()).void_unwrap();
        stream.feed(&[chunk; 1], |event| /* our  event handler */);
      }
    }

From here, I'm going to define a state machine to handle the events.  Since our synth will be monophonic, we need to track the last pressed note, turn off notes when we receive a note off event, and keep track of changes to the pitch bend and tone wheel values.  `Stack<T, N>` is a basic fixed-size stack allocated stack that we will use to track currently pressed notes.  We akso define a trait for actually implementing the oscillator.  This keeps the logic for setting up and updating PWM pins out of the MIDI state machine.

    :::rust
    pub trait Oscillator
    {
      fn enable(&mut self, frequency: f32, duty_cycle: f32);
      fn disable(&mut self);
    }
    
    pub struct Synth<O: Oscillator, const N: usize>
    {
      notes_pressed: Stack<u7, N>,
      bend: u14,
      oscillator: O
    }

First, I will define four utility functions.  The first will be called whenever the state changes and will update the underlying oscillator with the latest frequency.  To do this, we take the top note on the stack and the current pitch bend value and calculate the note's frequency,  The formula for deriving frequency given MIDI note number `n` and pitch bend value `p`, given A4=440Hz is as follows:

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
     <mrow>
      <mfrac>
       <mrow>
        <mi>n</mi>
        <mo stretchy="false">−</mo>
        <mn>69</mn>
       </mrow>
       <mn>12</mn>
      </mfrac>
      <mo stretchy="false">+</mo>
      <mfrac>
       <mrow>
        <mi>p</mi>
        <mo stretchy="false">−</mo>
        <mn>8192</mn>
       </mrow>
       <mrow>
        <mn>4096</mn>
        <mo stretchy="false">×</mo>
        <mn>12</mn>
       </mrow>
      </mfrac>
     </mrow>
    </msup>
   </mrow>
  </mrow>
 </semantics>
</math>

And, the utility function.  For now, we leave the duty cycle pinned at 75%; a future revision will map this to the tone wheel

    :::rust
    fn frequency_for_note_number(note: f32, pitch_bend: f32) -> f32
    {
      440_f32 * powf(
        2_f32, 
        ((note - 69_f32) / 12_f32) + ((pitch_bend - 8192) / 49152_f32))
    }

    fn update(&mut self)
    {
      if let Some(note) = self.notes_pressed.peek()
      {
        let frequency = frequency_for_note_number(note.as_int() as f32, self.bend.as_int() as f32);
        self.oscillator.enable(frequency, 0.75_f32);
      }
      else
      {
        self.oscillator.disable();
      }
    }

The next three utility functions simply handle note on, off, and pitch bend events.  MIDI sequences often contain note on events for notes already playing.  On other instruments, this would retrigger the envelope, but since we are outputting a simple square wave, we just move the note to the top of the stack.

    :::rust
    fn note_on(&mut self, note: u7)
    {
      self.notes_pressed.remove(note);
      self.notes_pressed.push(note);
      self.update_timer();
    }

    fn note_off(&mut self, note: u7)
    {
      self.notes_pressed.remove(note);
      self.update_timer();
    }

    fn pitch_bend(&mut self, amount: u14)
    {
      self.bend = amount;
      self.update_timer();
    }

Finally, we expose a single event handler for our state machine.  MIDI has two ways of communicating note off: a dedicated note off message, and a note on message with a velocity of zero.  We dispatch both to the same utility function.  For now, we ignore the channel the message was sent on.  A future revision will ignore messages not destined for our channel.

    :::rust
    pub fn handle_event(&mut self, event: LiveEvent<'_>)
    {
      if let LiveEvent::Midi { message, .. } = event
      {
        match message
        {
          MidiMessage::NoteOff { key, .. } => self.note_off(key),
          MidiMessage::NoteOn { key, vel } =>
          {
            if vel != 0
            {
              self.note_on(key);
            }
            else
            {
              self.note_off(key);
            }
          },
          MidiMessage::PitchBend { bend } => self.pitch_bend(bend.0),
          _ => {}
        }
      }
    }

A future post will cover the inner workings of the PWM output.