SERVICE_MAIN=konan.service
SERVICE_IOT=konan-iot.service
TIMER=konan_pulse.timer

BINARY_NAME=pi_cli
BUILD_PROFILE=release

# ---- lifecycle ----

deploy: stop update build install reload start

# ---- steps ----

stop:
	@echo "Stopping services and timer..."
	sudo systemctl stop $(SERVICE_MAIN) || true
	sudo systemctl stop $(SERVICE_IOT) || true
	sudo systemctl stop $(TIMER) || true

update:
	@echo "Pulling latest from GitHub..."
	git pull origin main

build:
	@echo "Building $(BINARY_NAME)..."
	cargo build --$(BUILD_PROFILE) --package $(BINARY_NAME)

reload:
	@echo "Reloading systemd..."
	sudo systemctl daemon-reload

start:
	@echo "Starting services and timer..."
	sudo systemctl start $(SERVICE_MAIN)
	sudo systemctl start $(SERVICE_IOT)
	sudo systemctl start $(TIMER)

restart:
	@echo "Restarting everything..."
	sudo systemctl restart $(SERVICE_MAIN)
	sudo systemctl restart $(SERVICE_IOT)
	sudo systemctl restart $(TIMER)
