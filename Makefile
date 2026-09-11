APP_NAME := stranger
PREFIX   := /usr/local
BINDIR   := $(PREFIX)/bin
TARGET   := target/release/$(APP_NAME)

build:
	cargo test
	cargo build --release --locked

install: build
	@echo "Installing $(APP_NAME) to $(BINDIR)"
	@sudo install -m 0755 $(TARGET) $(BINDIR)/$(APP_NAME)

uninstall:
	@echo "Removing $(BINDIR)/$(APP_NAME)"
	@sudo rm -f $(BINDIR)/$(APP_NAME)

run:
	$(TARGET)

.PHONY: build install uninstall run
