I have a Microsoft Surface Dial that works. I want to create software that I could run as a daemon in my linux machine that would allow me to have different modes. These modes would be:

- Hyprland horizontal scroll on a scrolling layout (read the latest hyprland documentation on scrolling layouts)
- Volume of the machine
- Scrolling of the focused application

For each of these, when I click the surface dial, these modes are being rotated and activated. Meaning that one click changes the mode to the next mode. This has to be shown visually on my hyprland desktop as a floating indicator in the middle of the screen. Elegant and themeable.

The software for this has to be as portable as possible from distro to distro so my minimum requirements are wayland, hyprland and of course the bluetooth stack and graphics stack or toolkit of your choice that can give the best performance.

I'm going to paste code of a relevant repo for the surface dial:

https://github.com/daniel5151/surface-dial-linux

here there is also some code for it:

https://www.reddit.com/r/SurfaceLinux/comments/eqk22k/surface_dial_on_linux/

I leave it up to you what language to implement it on but, as said, as to be performant and portable.


