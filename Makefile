PREFIX ?= $(HOME)/.local
BINDIR = $(PREFIX)/bin
CONFDIR = $(HOME)/.config/surface-dial
SERVICEDIR = $(HOME)/.config/systemd/user
UDEVDIR = /etc/udev/rules.d

.PHONY: build install uninstall enable disable udev

build:
	cargo build --release

install: build
	install -Dm755 target/release/surface-dial-daemon $(BINDIR)/surface-dial-daemon
	install -Dm644 surface-dial.service $(SERVICEDIR)/surface-dial.service
	@mkdir -p $(CONFDIR)
	@test -f $(CONFDIR)/config.toml || install -Dm644 config.toml $(CONFDIR)/config.toml
	@test -f $(CONFDIR)/theme.css || install -Dm644 theme.css $(CONFDIR)/theme.css
	@echo "Installed. Run 'make enable' to start the service."

uninstall: disable
	rm -f $(BINDIR)/surface-dial-daemon
	rm -f $(SERVICEDIR)/surface-dial.service
	systemctl --user daemon-reload
	@echo "Uninstalled. Config left in $(CONFDIR)"

enable:
	systemctl --user daemon-reload
	systemctl --user enable --now surface-dial.service
	@echo "Service enabled and started."

disable:
	-systemctl --user disable --now surface-dial.service

udev:
	sudo install -Dm644 udev/10-surface-dial.rules $(UDEVDIR)/10-surface-dial.rules
	sudo udevadm control --reload
	@echo "Udev rules installed."
