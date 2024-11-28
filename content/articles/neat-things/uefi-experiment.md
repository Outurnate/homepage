Title: Experiments in Building UEFI Programs
Date: 2022-12-26

Thanks to the work of [Rust OSDev](https://rust-osdev.com/), building UEFI bootloaders is surprisingly accessible.  Just for fun, I built a small toy EFI executable.  The executable won't do anything meaningful, such as boot an operating system.  In this article, I will go through some of the basics of the EFI environment and how I used them.

# Graphics Output Setup

To do anything meaningful in an EFI program, we must use one or more EFI protocols.  Protocols can be thought of as analogous to syscalls in a traditional userspace program.  There are two protocols we can use to put things on the screen: console I/O, and graphics output.  Console I/O is a simple protocol that allows us to just print text to the screen.  This protocol is perfect for simple boot messages or interactive command lines - but we want more.  This is what the alternative protocol, graphics output, is for.  This protocol allows for full control of the output pixels, at the expense of not having builtin niceties like fonts, text positioning, and keyboard input.  To give us some drawing primitives, we're going to use the `embedded-graphics` family of crates.  To start off, we'll create a type for holding the graphics output handle and owning the in-progress buffer.

    :::rust
    struct Graphics<'a>
    {
      gop: ScopedProtocol<'a, GraphicsOutput>,
      width: usize,
      height: usize,
      buffer: Vec<BltPixel>
    }

    impl<'a> Graphics<'a>
    {
      fn new(bt: &'a BootServices) -> Graphics<'a>
      {
        // Grab the handle for the graphics output protocol
        let gop_handle = bt.get_handle_for_protocol::<GraphicsOutput>().unwrap();

        // Signal that we are going to be the exclusive users of it
        let gop = bt.open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();

        // Fetch the resolution - we'll need it later
        let (width, height) = gop.current_mode_info().resolution();

        // Initialize the struct, and initialize the buffer with all black pixels
        Self { gop, width, height, buffer: vec![BltPixel::new(0, 0, 0); width * height] }
      }
    }

To make our `Graphics` struct usable as a target for `embedded-graphics` functions, we need to implement two traits.  The first is trivial - `Dimensions`, which contains a single function to report the bounding box we must draw within.  The second, `DrawTarget`, is more complex.  The primary function we must implement takes an iterator of pixels that correspond to whatever is current being drawn to this target.  A smart implementation would batch all these together, then expose a method to bitblit the combined buffer to the screen all at once.  In practice, I found that performing a bitblit on every draw call didn't lead to too much flicker, so I opted to leave this as is.

    ::rust
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
      where I: IntoIterator<Item = Pixel<Self::Color>>
    {
      for pixel in pixels
      {
        // ensure the pixel is within the screen coordinates
        if pixel.0.x >= 0 && pixel.0.x < self.width as i32 && pixel.0.y >= 0 && pixel.0.y < self.height as i32
        {
          // open the target pixel for writing
          let target_pixel = self.buffer.get_mut(pixel.0.y as usize * self.width + pixel.0.x as usize).unwrap();

          // copy each channel
          target_pixel.red = pixel.1.r();
          target_pixel.green = pixel.1.g();
          target_pixel.blue = pixel.1.b();
        }
      }

      // bitblit the entire screen
      self.gop.blt(BltOp::BufferToVideo
        {
          buffer: &self.buffer,
          src: BltRegion::Full,
          dest: (0, 0),
          dims: (self.width, self.height),
        })
    }

We then create a main function with standard boilerplate for initializing an EFI program.  Most of this is handled by the `#[entry]` macro, but we are responsible for the lifetime of the boot services.  We can use the boot services object to initialize our graphics handling struct.

    :::rust
    #[entry]
    fn main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status
    {
      uefi_services::init(&mut system_table).unwrap();
      let bt = system_table.boot_services();
      let mut display = Graphics::new(&bt);

      Status::SUCCESS
    }

At this point, all our EFI program does is setup and then immediate exit.  There is nothing interesting to see at this point.

# Simple Text & Images

To start off, let's draw some text.  I wrote a fake bootup message in `bios.txt`.  We'll render it on the screen in the top right corner.  To do this, we first need a simple bitmap font.  I used `FONT_12X16` from the [embedded-vintage-fonts](https://crates.io/crates/embedded-vintage-fonts) crate.

    :::rust
    Text::new(
      include_str!("bios.txt"),
      Point::new(8, 24),
      MonoTextStyle::new(&embedded_vintage_fonts::FONT_12X16, Rgb888::WHITE)
    ).draw(&mut display).unwrap();

Since our drawing target never returns a `Err` variant, we can safely unwrap without worry of panic.  If you are trying this at home, you will want to add a `bt.stall(500000)` call to the end of your program.  This prevents the system from immediately rebooting after drawing to the screen.

Let's put a logo on the screen.  We'll use [tinybmp](https://crates.io/crates/tinybmp) as a lightweight bitmap loader.  We can bake the logo into our binary using the `include_bytes` macro.  From there, it's a simple matter of calculating a position and sending the buffer over to our drawing target

    :::rust
    let bmp_data = include_bytes!("encom.bmp");
    let bmp = Bmp::from_slice(bmp_data).unwrap();
    let location = Point::new((display.bounding_box().size.width - bmp.bounding_box().size.width - 16) as i32, 16);
    Image::new(&bmp, location).draw(&mut display).unwrap();

We've got text, images, and drawing primitives.  We're all set to put together a small animation.

# Final Product

While we could use a long stream of `.draw` and `.stall` calls to animate something frame by frame, this isn't exactly an ergonomic process.  Enter asciinema: a tool for recording and playing back terminal interactions.  I added a `build.rs` script that would parse the output of asciicast into rust code for rendering it.  I only implemented a small subset of terminal control codes, but now I could record myself interacting with simple CLI programs and play them back within my EFI program.  I combined this with the static rendering techniques from the last section to produce a take on one of my favorite scenes from Tron: Legacy.

<video muted autoplay loop><source src="{attach}videos/encom.webm" type="video/webm"></video>

If you want to see the full code, or download the program for you own ESP, you can check it out [here on my GitHub](https://github.com/Outurnate/encom-uefi).  This was a weekend project, and most of the code does reflect that.  I may revisit this someday and maybe implement some simple games as EFI programs.