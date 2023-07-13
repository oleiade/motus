#Extract the project name from the Cargo.toml file
# PROJECT_NAME = $(shell grep '^name =' Cargo.toml | sed 's/name = "\(.*\)"/\1/')
PROJECT_NAME = $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].name')

# Extract the version number from the Cargo.toml file
# VERSION = $(shell grep '^version =' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
VERSION = $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')

# Map target names to Debian architecture names
DEB_ARCH_x86_64-unknown-linux-gnu := amd64
DEB_ARCH_aarch64-unknown-linux-gnu := arm64

# List of target names for Linux, Windows, and macOS
LINUX_TARGETS = x86_64-unknown-linux-gnu \
				aarch64-unknown-linux-gnu

MACOS_TARGETS = x86_64-apple-darwin \
				aarch64-apple-darwin

WINDOWS_TARGETS = x86_64-pc-windows-gnu \
				  x86_64-pc-windows-msvc



# Rule for building all targets
release: linux macos

# Rule for building Linux targets, Debian, and RPM packages
linux: check_cross check_linux_crosscompilation_on_macos $(LINUX_TARGETS) $(addprefix deb-, $(LINUX_TARGETS))

# Rule for building macOS targets
macos: check_cargo check_toolchain check_cross $(MACOS_TARGETS)

# Rules for building Linux targets and creating tar.gz archives
$(LINUX_TARGETS):
	@echo "building for target $@"
ifeq ($@, x86_64-unknown-linux-gnu)
	CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-unknown-linux-gnu-gcc cross build --target $@ --release
else
	@cross build --release --target=$@
endif
	mkdir -p release/$@
	cp target/$@/release/motus release/$@/
	tar czf release/motus-$@.tar.gz -C release/$@ motus

# Rules for building macOS targets and creating tar.gz archives
x86_64-apple-darwin:
	cross build --target $@ --release
	mkdir -p release/$@
	cp target/$@/release/motus release/$@/
	tar czf release/motus-$@.tar.gz -C release/$@ motus

aarch64-apple-darwin:
	cargo build --target $@ --release
	mkdir -p release/$@
	cp target/$@/release/motus release/$@/
	tar czf release/motus-$@.tar.gz -C release/$@ motus

# Rule for creating a Debian package
deb-%: check_deb
	@echo "Building Debian package for $*"
	cargo deb --no-build -p cli --target $* --output target/$*/$(PROJECT_NAME)-$(VERSION)-$*-$(DEB_ARCH_$*).deb
	mv target/$*/$(PROJECT_NAME)-$(VERSION)-$*-$(DEB_ARCH_$*).deb release/

# Rule for cleaning build artifacts
clean:
	rm -rf target
	rm -rf release

# Rule to check if 'cross' command is installed
check_cross:
	@command -v cross > /dev/null 2>&1 || { echo >&2 "Error: 'cross' command not found. Please install 'cross' by running 'cargo install cross'."; exit 1; }

# Rule to check if 'cargo' command is installed
check_cargo:
	@command -v cargo > /dev/null 2>&1 || { echo >&2 "Error: 'cargo' command not found. Please install Rust using the instructions at https://www.rust-lang.org/tools/install."; exit 1; }

# Rule to check if 'aarch64-apple-darwin' Rustup stable toolchain is installed
check_toolchain:
	@rustup target list --toolchain stable | grep aarch64-apple-darwin > /dev/null 2>&1 || { echo >&2 "Error: The 'aarch64-apple-darwin' Rustup stable toolchain is not installed. Please install it by running 'rustup target add aarch64-apple-darwin --toolchain stable'."; exit 1; }

# Rule to check if 'cargo-deb' tool is installed
check_deb:
	@command -v cargo-deb > /dev/null 2>&1 || { echo >&2 "Error: 'cargo-deb' command not found. Please install 'cargo-deb' by running 'cargo install cargo-deb'."; exit 1; }

# Rule to check if mingw-w64 is installed
check_mingw:
	@command -v x86_64-w64-mingw32-gcc > /dev/null 2>&1 || { echo >&2 "Error: 'mingw-w64' is not installed. Please install it according to your platform (macOS: 'brew install mingw-w64', Debian/Ubuntu: 'sudo apt-get install mingw-w64', Fedora: 'sudo dnf install mingw64-gcc')."; exit 1; }

# Verify Homebrew tap and formula for cross-compiling Linux targets on macOS
check_linux_crosscompilation_on_macos:
ifeq ($(shell uname),Darwin)
	@if ! brew tap | grep -q 'sergiobenitez/osxct'; then \
		echo "Homebrew tap 'sergiobenitez/osxct' is missing. Installing it now..."; \
		brew tap sergiobenitez/osxct; \
	fi
	@if ! brew list --formula | grep -q 'x86_64-unknown-linux-gnu'; then \
		echo "Homebrew formula 'x86_64-unknown-linux-gnu' is missing. Installing it now..."; \
		brew install x86_64-unknown-linux-gnu; \
	fi
	@if ! command -v x86_64-unknown-linux-gnu-gcc >/dev/null; then \
		echo "x86_64-unknown-linux-gnu-gcc not found in PATH. Please add it to your PATH."; \
		exit 1; \
	fi
endif

.PHONY: check_cross check_cargo check_toolchain check_deb check_rpm linux windows macos release $(LINUX_TARGETS) $(WINDOWS_TARGETS) $(MACOS_TARGETS) deb rpm