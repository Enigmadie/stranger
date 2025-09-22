APP_NAME := stranger
PREFIX   := /usr/local
BINDIR   := $(PREFIX)/bin
TARGET   := target/release/$(APP_NAME)

# Сборка релиза
build:
	cargo build --release --locked

# Установка в /usr/local/bin (попросит sudo)
install: build
	@echo "Installing $(APP_NAME) to $(BINDIR)"
	@sudo install -m 0755 $(TARGET) $(BINDIR)/$(APP_NAME)

# Удаление
uninstall:
	@echo "Removing $(BINDIR)/$(APP_NAME)"
	@sudo rm -f $(BINDIR)/$(APP_NAME)

# Тестовый прогон
run:
	$(TARGET)

.PHONY: build install uninstall run
