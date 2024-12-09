BUILD_DIR:=target

all: $(BUILD_DIR)/.proto_format_stamp
	cargo test --all-targets
	cargo fmt
	# disallow warnings so they fail CI
	cargo clippy --all-targets -- -D warnings
	# fail for rustdoc warnings
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
	cargo verify-project
	cargo audit

# Ensures proto files are formatted: must be added as a dependency before the generated files
$(BUILD_DIR)/.proto_format_stamp: $(wildcard proto/*.proto) | $(BUILD_DIR)
	clang-format --style=Google -i $<
	touch $@

$(BUILD_DIR):
	mkdir -p $@

clean:
	$(RM) -r $(BUILD_DIR)
