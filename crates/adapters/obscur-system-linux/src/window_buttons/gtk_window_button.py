import struct
import sys

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Graphene", "1.0")
gi.require_version("Gsk", "4.0")
from gi.repository import GLib, Graphene, Gsk, Gtk

role, scheme, backdrop, density = sys.argv[1:]
density = int(density)
Gtk.init()
settings = Gtk.Settings.get_default()
settings.set_property("gtk-enable-animations", False)
if settings.find_property("gtk-interface-color-scheme"):
    modes = {
        "dark": Gtk.InterfaceColorScheme.DARK,
        "light": Gtk.InterfaceColorScheme.LIGHT,
    }
    settings.set_property("gtk-interface-color-scheme", modes[scheme])
else:
    settings.set_property("gtk-application-prefer-dark-theme", scheme == "dark")

Adw = None
if settings.get_property("gtk-theme-name") in ("Adwaita", "Adwaita-dark"):
    try:
        gi.require_version("Adw", "1")
        from gi.repository import Adw
    except (ValueError, ImportError):
        pass

if Adw:
    Adw.init()
    style = Adw.StyleManager.get_default()
    style.set_color_scheme(
        Adw.ColorScheme.FORCE_DARK if scheme == "dark" else Adw.ColorScheme.FORCE_LIGHT
    )

def render(renderer, flags):
    window = Gtk.Window()
    try:
        if Adw:
            header = Adw.HeaderBar(show_start_title_buttons=False, show_end_title_buttons=False)
        else:
            header = Gtk.HeaderBar(show_title_buttons=False)

        controls = Gtk.WindowControls.new(Gtk.PackType.END)
        controls.set_decoration_layout(":" + ("maximize" if role == "restore" else role))
        header.pack_end(controls)
        window.set_titlebar(header)
        window.realize()
        window.allocate(300, 100, -1, None)

        button = controls.get_first_child()
        if role == "restore":
            button.get_child().set_from_icon_name("window-restore-symbolic")
        button.set_state_flags(flags, True)
        header.map()
        for _ in range(16):
            if not GLib.MainContext.default().iteration(False):
                break
        window.allocate(300, 100, -1, None)
        valid, bounds = button.compute_bounds(controls)
        if not valid:
            raise RuntimeError("GTK did not allocate the title button")

        snapshot = Gtk.Snapshot.new()
        snapshot.scale(density, density)
        controls.snapshot_child(button, snapshot)
        viewport = Graphene.Rect()
        viewport.init(
            bounds.get_x() * density, bounds.get_y() * density,
            bounds.get_width() * density, bounds.get_height() * density,
        )
        texture = renderer.render_texture(snapshot.to_node(), viewport)
        return int(bounds.get_width()), int(bounds.get_height()), texture.save_to_png_bytes().get_data()
    finally:
        window.destroy()

renderer = Gsk.CairoRenderer.new()
renderer.realize(None)
try:
    states = (
        Gtk.StateFlags.NORMAL,
        Gtk.StateFlags.PRELIGHT,
        Gtk.StateFlags.ACTIVE | Gtk.StateFlags.PRELIGHT,
        Gtk.StateFlags.FOCUS_VISIBLE | Gtk.StateFlags.FOCUSED,
    )
    for state in states:
        if backdrop == "inactive":
            state |= Gtk.StateFlags.BACKDROP
        width, height, png = render(renderer, state)
        sys.stdout.buffer.write(struct.pack("<III", width, height, len(png)))
        sys.stdout.buffer.write(png)
finally:
    renderer.unrealize()
